# phpswitch desktop app

A native desktop app (built with [Tauri](https://tauri.app)) for switching PHP versions —
no browser, no local web server. Works on Ubuntu and macOS.

- **Ubuntu/Linux**: a thin GUI over the [`phpswitch`](../phpswitch) CLI. Install the `.deb`
  from the repo root first (`../build-deb.sh`) — it provides `/usr/bin/phpswitch` and the
  `/etc/sudoers.d/phpswitch` rule this app uses for privileged actions (CLI/Apache/FPM
  switching, service restarts).
- **macOS**: switches PHP via Homebrew (`brew link`, `brew services`) directly — no `phpswitch`
  CLI or root privileges needed. Apache/mod_php has no Homebrew equivalent, so that card is
  hidden on macOS.

> **macOS support is currently unverified.** This project was developed and tested on Linux;
> the macOS backend (`src-tauri/src/macos.rs`) follows documented Homebrew CLI behavior but
> hasn't been run on a real Mac yet. Please test it and file an issue if something's off.

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org) 18+ and npm
- [Rust](https://www.rust-lang.org/tools/install) (stable) — via `rustup`, not just a distro
  package; Tauri's dependency tree needs a fairly recent toolchain
- Platform WebView dev headers:
  - **Ubuntu**: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)

### Build

```bash
cd desktop
npm install
npm run build
```

Output:
- Ubuntu: `src-tauri/target/release/bundle/deb/*.deb` and `bundle/appimage/*.AppImage`
- macOS: `src-tauri/target/release/bundle/dmg/*.dmg` and `bundle/macos/*.app`

### Dev mode (hot reload)

```bash
cd desktop
npm install
npm run dev
```

## CI builds

`.github/workflows/release.yml` builds both platforms (plus the CLI `.deb` from the repo
root) on a `v*` tag push, and publishes everything to one combined GitHub Release —
useful for producing the macOS build without owning a Mac, and the only supported way to
ship a release others can download without a manual "publish draft" step. A manual
`workflow_dispatch` run builds the same artifacts for a sanity check but uploads them as
workflow artifacts instead of touching Releases.

## Architecture

- `src/` — frontend: static `index.html` + `app.js` (no build step / bundler), calling into
  the Rust backend via `window.__TAURI__.core.invoke(...)`.
- `src-tauri/src/status.rs` — the shared `PhpStatus`/`PhpVersion` shape returned by every
  command.
- `src-tauri/src/linux.rs` — shells out to `phpswitch --json-status` / `sudo -n phpswitch
  --set-cli|--set-apache|--set-fpm|--restart-services`.
- `src-tauri/src/macos.rs` — Homebrew-based equivalent (`brew list`, `brew link`,
  `brew services`).
- `src-tauri/src/lib.rs` — Tauri commands (`get_status`, `set_cli`, `set_apache`, `set_fpm`,
  `restart_services`, `rescan`) dispatching to whichever backend matches the target OS.
