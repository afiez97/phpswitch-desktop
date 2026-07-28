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

// A dry run is entirely read-only (it only previews what would happen), so
// it's routed through `run()` — no sudo, no password prompt — instead of
// `run_privileged()`.
pub fn set_cli(version: &str, dry_run: bool) -> Result<String, String> {
    check_version(version)?;
    if dry_run {
        run(&["--set-cli", version, "--dry-run"])
    } else {
        run_privileged(&["--set-cli", version])
    }
}

pub fn set_apache(version: &str, dry_run: bool) -> Result<String, String> {
    check_version(version)?;
    if dry_run {
        run(&["--set-apache", version, "--dry-run"])
    } else {
        run_privileged(&["--set-apache", version])
    }
}

pub fn set_fpm(version: &str, dry_run: bool) -> Result<String, String> {
    check_version(version)?;
    if dry_run {
        run(&["--set-fpm", version, "--dry-run"])
    } else {
        run_privileged(&["--set-fpm", version])
    }
}

pub fn restart_services(dry_run: bool) -> Result<String, String> {
    if dry_run {
        run(&["--restart-services", "--dry-run"])
    } else {
        run_privileged(&["--restart-services"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_versions_are_accepted() {
        assert!(is_valid_version("8.3"));
        assert!(is_valid_version("8.10"));
        assert!(is_valid_version("10.0"));
    }

    #[test]
    fn malformed_versions_are_rejected() {
        for bad in ["8", "8.", ".3", "8.3.1", "8.x", "", "8..3", "a.b"] {
            assert!(!is_valid_version(bad), "expected {bad:?} to be invalid");
        }
    }

    #[test]
    fn check_version_matches_is_valid_version() {
        assert!(check_version("8.3").is_ok());
        assert!(check_version("8.3.1").is_err());
    }

    #[test]
    fn raw_status_parses_phpswitch_json_status_output() {
        // Mirrors the exact shape print_json_status emits in the phpswitch
        // bash script (phpswitch:print_json_status).
        let json = r#"{"cli":"8.3","apache":"8.3","activeFpm":["8.3"],"apacheRunning":true,"nginxRunning":false,"versions":[{"version":"8.0","path":"/usr/bin/php8.0","hasApache":true,"hasFpm":true},{"version":"8.3","path":"/usr/bin/php8.3","hasApache":true,"hasFpm":true}]}"#;

        let raw: RawStatus = serde_json::from_str(json).expect("should parse");
        assert_eq!(raw.cli, "8.3");
        assert_eq!(raw.apache, "8.3");
        assert_eq!(raw.active_fpm, vec!["8.3".to_string()]);
        assert!(raw.apache_running);
        assert!(!raw.nginx_running);
        assert_eq!(raw.versions.len(), 2);
        assert_eq!(raw.versions[1].version, "8.3");
        assert_eq!(raw.versions[1].path, "/usr/bin/php8.3");
        assert!(raw.versions[1].has_apache);
        assert!(raw.versions[1].has_fpm);
    }

    #[test]
    fn raw_status_handles_no_apache_and_empty_fpm() {
        let json = r#"{"cli":"8.3","apache":"none","activeFpm":[],"apacheRunning":false,"nginxRunning":false,"versions":[]}"#;
        let raw: RawStatus = serde_json::from_str(json).expect("should parse");
        assert_eq!(raw.apache, "none");
        assert!(raw.active_fpm.is_empty());
        assert!(raw.versions.is_empty());
    }
}
