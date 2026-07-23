mod status;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

use status::PhpStatus;

fn read_status() -> Result<PhpStatus, String> {
    #[cfg(target_os = "linux")]
    {
        linux::get_status()
    }
    #[cfg(target_os = "macos")]
    {
        macos::get_status()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform.".to_string())
    }
}

#[tauri::command]
fn get_status() -> Result<PhpStatus, String> {
    read_status()
}

#[derive(serde::Serialize)]
struct ActionResult {
    ok: bool,
    log: String,
    #[serde(rename = "logKind")]
    log_kind: String,
    status: Option<PhpStatus>,
}

fn run_action(action_result: Result<String, String>) -> ActionResult {
    match action_result {
        Ok(log) => {
            let status = read_status().ok();
            ActionResult { ok: true, log, log_kind: "ok".to_string(), status }
        }
        Err(log) => {
            let status = read_status().ok();
            ActionResult { ok: false, log, log_kind: "warn".to_string(), status }
        }
    }
}

#[tauri::command]
fn set_cli(version: String) -> ActionResult {
    #[cfg(target_os = "linux")]
    let result = linux::set_cli(&version);
    #[cfg(target_os = "macos")]
    let result = macos::set_cli(&version);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let result: Result<String, String> = Err("Unsupported platform.".to_string());

    run_action(result)
}

#[tauri::command]
fn set_apache(version: String) -> ActionResult {
    #[cfg(target_os = "linux")]
    let result = linux::set_apache(&version);
    #[cfg(target_os = "macos")]
    let result = macos::set_apache(&version);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let result: Result<String, String> = Err("Unsupported platform.".to_string());

    run_action(result)
}

#[tauri::command]
fn set_fpm(version: String) -> ActionResult {
    #[cfg(target_os = "linux")]
    let result = linux::set_fpm(&version);
    #[cfg(target_os = "macos")]
    let result = macos::set_fpm(&version);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let result: Result<String, String> = Err("Unsupported platform.".to_string());

    run_action(result)
}

#[tauri::command]
fn restart_services() -> ActionResult {
    #[cfg(target_os = "linux")]
    let result = linux::restart_services();
    #[cfg(target_os = "macos")]
    let result = macos::restart_services();
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let result: Result<String, String> = Err("Unsupported platform.".to_string());

    run_action(result)
}

#[tauri::command]
fn rescan() -> ActionResult {
    run_action(Ok("Rescanned installed PHP versions.".to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_cli,
            set_apache,
            set_fpm,
            restart_services,
            rescan
        ])
        .run(tauri::generate_context!())
        .expect("error while running phpswitch desktop app");
}
