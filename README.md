<img src="docs/logo.png" alt="phpswitch logo" width="90" align="left">

# phpswitch-desktop

A native desktop app (and CLI) for switching the active PHP version on Ubuntu and macOS —
CLI, Apache2 mod_php, PHP-FPM, and Nginx, all in one click.

![Ubuntu](https://img.shields.io/badge/Ubuntu-20.04+-E95420?logo=ubuntu&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-Homebrew-000000?logo=apple&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue)
[![Releases](https://img.shields.io/github/v/release/afiez97/phpswitch-desktop?include_prereleases&label=release)](https://github.com/afiez97/phpswitch-desktop/releases)

<br clear="left">

![phpswitch desktop app](docs/desktop-app.png)

> Looking for the terminal-only version? See
> [afiez97/phpswitch](https://github.com/afiez97/phpswitch). This repo builds on that same
> CLI and adds a native GUI on top of it.

---

## Download

Grab the latest build from **[Releases](https://github.com/afiez97/phpswitch-desktop/releases)**:

| Platform | File |
|---|---|
| Ubuntu (Debian package) | `phpswitch-desktop_*_amd64.deb` |
| Ubuntu (portable, no install) | `phpswitch-desktop_*_amd64.AppImage` |
| macOS | `phpswitch-desktop_*.dmg` |

Ubuntu also needs the `phpswitch` CLI installed first — see [Installation](#installation) below.
macOS needs [Homebrew](https://brew.sh) with a `php`/`php@X.Y` formula installed; no CLI package
required.

---

## Features

- One-click switching from a native app window — no browser, no terminal
- Switches PHP for **all components at once**: CLI, Apache2, PHP-FPM, Nginx (Linux) /
  Homebrew services (macOS)
- Auto-detects all installed PHP versions
- Auto-updates the Nginx `fastcgi_pass` socket to match the active FPM version (Linux)
- Shows which versions have Apache module / FPM support available
- Also usable as a plain terminal CLI (`phpswitch`) if you don't want the GUI

---

## Installation

### 1. Install the `phpswitch` CLI (Ubuntu only — the desktop app's Linux backend needs it)

```bash
git clone https://github.com/afiez97/phpswitch-desktop.git
cd phpswitch-desktop
./build-deb.sh
sudo dpkg -i build/phpswitch_*_all.deb
```

This installs `/usr/bin/phpswitch` and a scoped `/etc/sudoers.d/phpswitch` rule — see
[What the sudoers rule grants](#what-the-sudoers-rule-grants).

> macOS doesn't need this step — the desktop app talks to Homebrew directly.

### 2. Install the desktop app

Download the `.deb`/`.AppImage`/`.dmg` from [Releases](https://github.com/afiez97/phpswitch-desktop/releases), or build it yourself:

```bash
cd desktop
npm install
npm run build
```

See [`desktop/README.md`](desktop/README.md) for full build prerequisites and output paths.

### Verify the CLI

```bash
phpswitch --status
```

---

## Usage

### Desktop app

Launch it from your app menu (after installing the `.deb`/`.dmg`) or run the `.AppImage`
directly. Click a version's **CLI**/**Apache**/**Nginx** button to switch that target;
**Restart web servers** applies pending Apache/Nginx changes; **Rescan** re-reads installed
versions.

### CLI

```bash
sudo phpswitch              # interactive menu
sudo phpswitch 8.3          # switch CLI + Apache + PHP-FPM + Nginx to PHP 8.3
phpswitch --status          # show current status (no sudo needed)
phpswitch --json-status     # same, as JSON
phpswitch --help
```

```
  PHP Version Status
──────────────────────────────────────────
  CLI       PHP 8.3
  Apache    mod_php8.3  (active)
  PHP-FPM   php8.3-fpm  (active)
  Nginx     active
──────────────────────────────────────────
  Installed PHP versions:
    PHP 8.0 [apache] [fpm]
    PHP 8.3 [apache] [fpm]  ← CLI  ← Apache
    PHP 8.4 [apache] [fpm]
    PHP 8.5
```

---

## What gets switched

| Component | Ubuntu | macOS |
|---|---|---|
| **CLI** | `update-alternatives --set php /usr/bin/php<ver>` | `brew link --force --overwrite php@<ver>` |
| **CLI tools** | `phpize`, `php-config`, `phar`, `phar.phar`, `php-cgi` | — |
| **Apache2** | Disables active `mod_phpX.Y`, enables `mod_php<ver>`, restarts `apache2` | not applicable (no Homebrew equivalent) |
| **PHP-FPM** | Stops other `phpX.Y-fpm`, starts `php<ver>-fpm` | `brew services stop/start php@<ver>` |
| **Nginx socket** | Updates `fastcgi_pass` in `sites-enabled/*` | not automated yet |
| **Nginx** | Reloads after config test | `brew services restart nginx` |

---

## Requirements

| Requirement | Ubuntu | macOS |
|---|---|---|
| OS | 20.04, 22.04, 24.04 | recent macOS with [Homebrew](https://brew.sh) |
| PHP versions | via `ondrej/php` PPA or `apt` | via `brew install php@X.Y` |
| Web servers | Apache2 and/or Nginx (either, both, or neither) | Nginx via Homebrew (optional) |

Installing more PHP versions on Ubuntu:

```bash
sudo add-apt-repository ppa:ondrej/php
sudo apt update
sudo apt install php8.4 php8.4-fpm php8.4-cli libapache2-mod-php8.4
```

On macOS:

```bash
brew install php@8.4
```

---

## What the sudoers rule grants

The desktop app needs root for `update-alternatives`, `a2enmod`/`a2dismod`, and `systemctl`
on Linux. Rather than running the whole app as root, the `.deb` installs
`/etc/sudoers.d/phpswitch`, which grants passwordless `sudo` for exactly these four commands
and nothing else:

```
/usr/bin/phpswitch --set-cli <version>
/usr/bin/phpswitch --set-apache <version>
/usr/bin/phpswitch --set-fpm <version>
/usr/bin/phpswitch --restart-services
```

Each of these validates its version argument (`\d+\.\d+`) and checks the PHP binary actually
exists before doing anything — see `validate_version_arg` in the `phpswitch` script. Review
`/etc/sudoers.d/phpswitch` yourself before relying on it; remove it
(`sudo rm /etc/sudoers.d/phpswitch`) if you'd rather be prompted for a password on every switch.

macOS needs no such rule — Homebrew operates entirely as the logged-in user.

---

## Uninstall

**Desktop app**: uninstall like any other app (`sudo dpkg --purge phpswitch-desktop` if
installed via `.deb`; drag to Trash on macOS).

**CLI**:

```bash
sudo dpkg --purge phpswitch   # also removes /etc/sudoers.d/phpswitch
```

Or, if installed via `install.sh`:

```bash
sudo bash uninstall.sh
```

---

## Troubleshooting

### Apache fails to restart
Run `sudo apache2ctl configtest` to find config errors.

### Nginx config test fails
The tool skips the Nginx reload if `nginx -t` fails. Run `sudo nginx -t` to see the error.

### PHP-FPM not found for a version
The CLI will warn and show the install command — a version may be installed without its
`-fpm` package yet.

### Switch works but Composer still shows the old version
Composer reads `php` from `PATH`. Open a new terminal or run `hash -r` to refresh it.

### Desktop app: "Passwordless sudo isn't set up for phpswitch"
Reinstall the CLI's `.deb` (`sudo dpkg -i build/phpswitch_*_all.deb`) — it installs the
sudoers rule described above.

### macOS: switching isn't verified yet
The macOS backend (`desktop/src-tauri/src/macos.rs`) was developed without access to a real
Mac. If something doesn't behave as expected, please open an issue.

---

## Building from source

- **CLI + `.deb`**: `./build-deb.sh` at the repo root (see [`build-deb.sh`](build-deb.sh)).
- **Desktop app**: see [`desktop/README.md`](desktop/README.md) for prerequisites and build
  steps on both platforms, plus how the `.github/workflows/desktop-build.yml` CI release
  pipeline works.

---

## Author

**afiez** — [github.com/afiez97](https://github.com/afiez97)

---

## License

[MIT](LICENSE)
