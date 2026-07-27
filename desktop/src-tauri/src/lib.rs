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
async fn get_status() -> Result<PhpStatus, String> {
    tauri::async_runtime::spawn_blocking(read_status)
        .await
        .map_err(|e| format!("Internal error: {e}"))?
}

#[derive(serde::Serialize)]
struct ActionResult {
    ok: bool,
    log: String,
    #[serde(rename = "logKind")]
    log_kind: String,
    status: Option<PhpStatus>,
}

// Runs `f` (and the subsequent status refresh) on a blocking thread pool
// thread rather than the main thread — `f` shells out to `sudo phpswitch`,
// which can take several seconds (Apache/FPM restarts). Tauri commands that
// aren't `async` run on the main thread, which also drives the webview, so
// a blocking call there would freeze the UI (no repaint) until it returns.
async fn run_action<F>(f: F) -> ActionResult
where
    F: FnOnce() -> Result<String, String> + Send + 'static,
{
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let action_result = f();
        let status = read_status().ok();
        (action_result, status)
    })
    .await;

    match outcome {
        Ok((Ok(log), status)) => ActionResult { ok: true, log, log_kind: "ok".to_string(), status },
        Ok((Err(log), status)) => ActionResult { ok: false, log, log_kind: "warn".to_string(), status },
        Err(e) => ActionResult {
            ok: false,
            log: format!("Internal error: {e}"),
            log_kind: "warn".to_string(),
            status: None,
        },
    }
}

#[tauri::command]
async fn set_cli(version: String) -> ActionResult {
    run_action(move || {
        #[cfg(target_os = "linux")]
        { linux::set_cli(&version) }
        #[cfg(target_os = "macos")]
        { macos::set_cli(&version) }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("Unsupported platform.".to_string()) }
    })
    .await
}

#[tauri::command]
async fn set_apache(version: String) -> ActionResult {
    run_action(move || {
        #[cfg(target_os = "linux")]
        { linux::set_apache(&version) }
        #[cfg(target_os = "macos")]
        { macos::set_apache(&version) }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("Unsupported platform.".to_string()) }
    })
    .await
}

#[tauri::command]
async fn set_fpm(version: String) -> ActionResult {
    run_action(move || {
        #[cfg(target_os = "linux")]
        { linux::set_fpm(&version) }
        #[cfg(target_os = "macos")]
        { macos::set_fpm(&version) }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("Unsupported platform.".to_string()) }
    })
    .await
}

#[tauri::command]
async fn restart_services() -> ActionResult {
    run_action(|| {
        #[cfg(target_os = "linux")]
        { linux::restart_services() }
        #[cfg(target_os = "macos")]
        { macos::restart_services() }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        { Err("Unsupported platform.".to_string()) }
    })
    .await
}

#[tauri::command]
async fn rescan() -> ActionResult {
    run_action(|| Ok("Rescanned installed PHP versions.".to_string())).await
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
