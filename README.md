# phpswitch

Easy PHP version switcher for Ubuntu — switches CLI, Apache2 mod_php, PHP-FPM, and Nginx in one command.

![Ubuntu](https://img.shields.io/badge/Ubuntu-24.04+-E95420?logo=ubuntu&logoColor=white)
![Bash](https://img.shields.io/badge/Shell-Bash-4EAA25?logo=gnubash&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue)

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

## Features

- Switches PHP for **all components at once**: CLI, Apache2, PHP-FPM, Nginx
- Auto-detects all installed PHP versions
- Auto-updates Nginx `fastcgi_pass` socket to match the new FPM version
- Shows which versions have Apache module / FPM installed
- Interactive numbered menu or direct version argument
- Warns instead of failing when a component isn't available for the requested version
- Switches companion tools: `phpize`, `php-config`, `phar`, `php-cgi`

## Requirements

| Requirement | Version |
|---|---|
| Ubuntu | 20.04, 22.04, 24.04 |
| Bash | 4.0+ |
| systemd | any |
| PHP versions | installed via `ondrej/php` PPA or `apt` |

> Works with Apache2, Nginx, or both — whichever you have installed.

---

## Installation

### Quick install (one-liner)

```bash
curl -fsSL https://raw.githubusercontent.com/afiez97/phpswitch/main/install.sh | sudo bash
```

### Manual install

```bash
git clone https://github.com/afiez97/phpswitch.git
cd phpswitch
sudo bash install.sh
```

### Verify

```bash
phpswitch --status
```

---

## Usage

### Interactive menu

```bash
sudo phpswitch
```

Displays current status and a numbered list to pick from:

```
  Switch to:
    [1]  PHP 8.0 [apache] [fpm]
    [2]  PHP 8.3 [apache] [fpm]
    [3]  PHP 8.4 [apache] [fpm]
    [4]  PHP 8.5
    [s]  Show status
    [q]  Quit

  Choice:
```

### Switch directly

```bash
sudo phpswitch 8.3
sudo phpswitch 8.4
sudo phpswitch 8.5
```

### Show current status

```bash
phpswitch --status    # no sudo needed
```

### Help

```bash
phpswitch --help
```

---

## What gets switched

| Component | Action |
|---|---|
| **CLI** | `update-alternatives --set php /usr/bin/php<ver>` |
| **CLI tools** | Also switches `phpize`, `php-config`, `phar`, `phar.phar`, `php-cgi` |
| **Apache2** | Disables all active `mod_phpX.Y`, enables `mod_php<ver>`, restarts `apache2` |
| **PHP-FPM** | Stops all other `phpX.Y-fpm` services, enables & starts `php<ver>-fpm` |
| **Nginx socket** | Updates `fastcgi_pass unix:/run/php/php<ver>-fpm.sock` in `sites-enabled/*` |
| **Nginx** | Reloads nginx (after config test) |

---

## Installing PHP versions

If a version you want isn't installed, use the **ondrej/php** PPA:

```bash
sudo add-apt-repository ppa:ondrej/php
sudo apt update

# Install a specific version with common extensions
sudo apt install php8.4 php8.4-fpm php8.4-cli \
    php8.4-mysql php8.4-pgsql php8.4-mbstring \
    php8.4-xml php8.4-curl php8.4-zip php8.4-gd \
    php8.4-intl php8.4-bcmath php8.4-redis

# Apache module (for mod_php)
sudo apt install libapache2-mod-php8.4
```

---

## Uninstall

```bash
sudo bash /path/to/phpswitch/uninstall.sh
```

Or manually:

```bash
sudo rm /usr/local/bin/phpswitch
```

---

## Troubleshooting

### Apache fails to restart
Run `sudo apache2ctl configtest` to find config errors.

### Nginx config test fails
The tool skips Nginx reload if `nginx -t` fails. Run `sudo nginx -t` to see the error.

### PHP-FPM not found for a version
The tool will warn and show the install command. PHP 8.5 CLI may be installed without a `-fpm` package yet.

### Switch works but Composer still shows old version
Composer reads `php` from PATH. Open a new terminal or run `hash -r` to refresh.

### Apache module not available for a version
```bash
sudo apt install libapache2-mod-php8.5
```

---

## Author

**afiez** — [github.com/afiez97](https://github.com/afiez97)

---

## License

[MIT](LICENSE)
