// Linux backend: a thin wrapper around the existing `phpswitch` bash CLI and
// its sudoers rule (see /etc/sudoers.d/phpswitch), rather than reimplementing
// update-alternatives/Apache/PHP-FPM logic in Rust.

use std::path::PathBuf;
use std::process::Command;

use crate::status::{PhpStatus, PhpVersion};

fn find_binary() -> Result<PathBuf, String> {
    if let Ok(override_path) = std::env::var("PHPSWITCH_BIN") {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Ok(path);
        }
    }

    for candidate in ["/usr/bin/phpswitch", "/usr/local/bin/phpswitch"] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }

    // Fall back to PATH lookup (covers dev checkouts running `phpswitch` via a symlink).
    if let Ok(output) = Command::new("which").arg("phpswitch").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    Err("phpswitch CLI not found. Install it first: see README.md".to_string())
}

fn run(args: &[&str]) -> Result<String, String> {
    let bin = find_binary()?;
    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", bin.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let msg = if !stderr.trim().is_empty() { stderr } else { stdout };
        return Err(msg.trim().to_string());
    }
    Ok(stdout)
}

fn run_privileged(args: &[&str]) -> Result<String, String> {
    let bin = find_binary()?;
    let mut sudo_args: Vec<&str> = vec!["-n", bin.to_str().unwrap_or("phpswitch")];
    sudo_args.extend_from_slice(args);

    let output = Command::new("sudo")
        .args(&sudo_args)
        .output()
        .map_err(|e| format!("Failed to run sudo: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}{stderr}");

    if !output.status.success() {
        if stderr.contains("a password is required") || stderr.contains("sudo:") {
            return Err(
                "Passwordless sudo isn't set up for phpswitch. Reinstall the .deb, which installs /etc/sudoers.d/phpswitch."
                    .to_string(),
            );
        }
        return Err(if combined.trim().is_empty() {
            "Command failed.".to_string()
        } else {
            combined.trim().to_string()
        });
    }

    Ok(if combined.trim().is_empty() {
        "Done.".to_string()
    } else {
        combined.trim().to_string()
    })
}

#[derive(serde::Deserialize)]
struct RawVersion {
    version: String,
    path: String,
    #[serde(rename = "hasApache")]
    has_apache: bool,
    #[serde(rename = "hasFpm")]
    has_fpm: bool,
}

#[derive(serde::Deserialize)]
struct RawStatus {
    cli: String,
    apache: String,
    #[serde(rename = "activeFpm")]
    active_fpm: Vec<String>,
    #[serde(rename = "apacheRunning")]
    apache_running: bool,
    #[serde(rename = "nginxRunning")]
    nginx_running: bool,
    versions: Vec<RawVersion>,
}

pub fn get_status() -> Result<PhpStatus, String> {
    let raw_json = run(&["--json-status"])?;
    let raw: RawStatus = serde_json::from_str(raw_json.trim())
        .map_err(|e| format!("Failed to parse phpswitch --json-status output: {e}"))?;

    Ok(PhpStatus {
        cli: raw.cli,
        apache: Some(raw.apache),
        active_fpm: raw.active_fpm,
        apache_running: raw.apache_running,
        nginx_running: raw.nginx_running,
        versions: raw
            .versions
            .into_iter()
            .map(|v| PhpVersion {
                version: v.version,
                path: v.path,
                has_apache: v.has_apache,
                has_fpm: v.has_fpm,
            })
            .collect(),
    })
}

fn is_valid_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let major = parts.next();
    let minor = parts.next();
    let rest_empty = parts.next().is_none();
    matches!((major, minor), (Some(a), Some(b)) if !a.is_empty() && !b.is_empty()
        && a.chars().all(|c| c.is_ascii_digit())
        && b.chars().all(|c| c.is_ascii_digit()))
        && rest_empty
}

fn check_version(version: &str) -> Result<(), String> {
    if !is_valid_version(version) {
        return Err(format!("Invalid PHP version: {version}"));
    }
    Ok(())
}

pub fn set_cli(version: &str) -> Result<String, String> {
    check_version(version)?;
    run_privileged(&["--set-cli", version])
}

pub fn set_apache(version: &str) -> Result<String, String> {
    check_version(version)?;
    run_privileged(&["--set-apache", version])
}

pub fn set_fpm(version: &str) -> Result<String, String> {
    check_version(version)?;
    run_privileged(&["--set-fpm", version])
}

pub fn restart_services() -> Result<String, String> {
    run_privileged(&["--restart-services"])
}
