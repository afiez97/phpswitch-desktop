// macOS backend: Homebrew-based PHP switching. There is no direct macOS
// equivalent of update-alternatives/Apache mod_php/systemd, so this is a
// separate implementation (not a wrapper around `phpswitch`, which is
// Debian/Ubuntu-specific). No root/sudo is needed — Homebrew operates
// entirely as the logged-in user.
//
// NOTE: this backend could not be built or run on macOS during development
// (this project was developed in a Linux container) — it compiles and
// follows documented Homebrew CLI behavior, but treat it as unverified until
// tested on a real Mac. See README.md's Desktop App section.

use std::process::Command;

use crate::status::{PhpStatus, PhpVersion};

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {cmd}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(if stderr.trim().is_empty() { stdout } else { stderr }.trim().to_string());
    }
    Ok(stdout)
}

fn brew(args: &[&str]) -> Result<String, String> {
    run("brew", args)
}

/// Installed `php` / `php@X.Y` formulas, each resolved to an "X.Y" version string.
fn installed_formulas() -> Result<Vec<(String, String)>, String> {
    let listed = brew(&["list", "--formula"])?;
    let mut result = Vec::new();

    for name in listed.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(ver) = name.strip_prefix("php@") {
            result.push((name.to_string(), ver.to_string()));
        } else if name == "php" {
            if let Ok(ver) = formula_version("php") {
                result.push((name.to_string(), ver));
            }
        }
    }
    Ok(result)
}

/// Resolve the "X.Y" version of a formula via `brew list --versions <formula>`.
fn formula_version(formula: &str) -> Result<String, String> {
    let out = brew(&["list", "--versions", formula])?;
    let full = out
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| format!("Could not determine version for {formula}"))?;
    let parts: Vec<&str> = full.split('.').collect();
    if parts.len() >= 2 {
        Ok(format!("{}.{}", parts[0], parts[1]))
    } else {
        Ok(full.to_string())
    }
}

fn current_cli_version() -> String {
    if let Ok(out) = run("php", &["-v"]) {
        if let Some(first_line) = out.lines().next() {
            // "PHP 8.3.9 (cli) ..."
            let words: Vec<&str> = first_line.split_whitespace().collect();
            if words.len() >= 2 {
                let parts: Vec<&str> = words[1].split('.').collect();
                if parts.len() >= 2 {
                    return format!("{}.{}", parts[0], parts[1]);
                }
            }
        }
    }
    "?".to_string()
}

/// Formula names with a "started" Homebrew service, e.g. ["php@8.3"].
fn running_php_services() -> Vec<String> {
    let Ok(out) = brew(&["services", "list"]) else {
        return Vec::new();
    };
    out.lines()
        .skip(1) // header row
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let name = cols.next()?;
            let status = cols.next()?;
            if (name == "php" || name.starts_with("php@")) && status == "started" {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn get_status() -> Result<PhpStatus, String> {
    let formulas = installed_formulas()?;
    let cli = current_cli_version();
    let running = running_php_services();

    let active_fpm: Vec<String> = formulas
        .iter()
        .filter(|(name, _)| running.contains(name))
        .map(|(_, ver)| ver.clone())
        .collect();

    let versions = formulas
        .into_iter()
        .map(|(name, ver)| PhpVersion {
            path: format!("(brew: {name})"),
            version: ver,
            has_apache: false,
            has_fpm: true,
            })
        .collect();

    Ok(PhpStatus {
        cli,
        apache: None, // no Apache/mod_php equivalent via Homebrew — frontend hides the card
        active_fpm,
        apache_running: false,
        nginx_running: !running_service_named("nginx").is_empty(),
        versions,
    })
}

fn running_service_named(target: &str) -> String {
    let Ok(out) = brew(&["services", "list"]) else {
        return String::new();
    };
    out.lines()
        .find(|line| line.trim_start().starts_with(target))
        .map(|line| line.to_string())
        .filter(|line| line.split_whitespace().nth(1) == Some("started"))
        .unwrap_or_default()
}

fn formula_name_for_version(version: &str) -> Result<String, String> {
    let formulas = installed_formulas()?;
    formulas
        .into_iter()
        .find(|(_, ver)| ver == version)
        .map(|(name, _)| name)
        .ok_or_else(|| format!("PHP {version} is not installed via Homebrew."))
}

pub fn set_cli(version: &str) -> Result<String, String> {
    let target = formula_name_for_version(version)?;
    let formulas = installed_formulas()?;

    for (name, _) in &formulas {
        let _ = brew(&["unlink", name]);
    }
    brew(&["link", "--force", "--overwrite", &target])?;
    Ok(format!("CLI → {target} (via Homebrew)"))
}

pub fn set_apache(_version: &str) -> Result<String, String> {
    Err("Apache/mod_php isn't available via Homebrew on macOS.".to_string())
}

pub fn set_fpm(version: &str) -> Result<String, String> {
    let target = formula_name_for_version(version)?;

    for name in running_php_services() {
        if name != target {
            let _ = brew(&["services", "stop", &name]);
        }
    }
    brew(&["services", "start", &target])?;
    Ok(format!("PHP-FPM → {target} (via brew services)"))
}

pub fn restart_services() -> Result<String, String> {
    if !running_service_named("nginx").is_empty() {
        brew(&["services", "restart", "nginx"])?;
        Ok("nginx restarted (via brew services).".to_string())
    } else {
        Ok("nginx is not running via Homebrew services — nothing to restart.".to_string())
    }
}
