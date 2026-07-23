use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct PhpVersion {
    pub version: String,
    pub path: String,
    #[serde(rename = "hasApache")]
    pub has_apache: bool,
    #[serde(rename = "hasFpm")]
    pub has_fpm: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct PhpStatus {
    pub cli: String,
    /// `null` on platforms with no Apache/mod_php equivalent (e.g. macOS) —
    /// the frontend hides the Apache card when this is null.
    pub apache: Option<String>,
    #[serde(rename = "activeFpm")]
    pub active_fpm: Vec<String>,
    #[serde(rename = "apacheRunning")]
    pub apache_running: bool,
    #[serde(rename = "nginxRunning")]
    pub nginx_running: bool,
    pub versions: Vec<PhpVersion>,
}
