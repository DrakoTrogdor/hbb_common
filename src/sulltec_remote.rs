//! The fork's own values in `hbb_common`.

pub const ST_SERVER_HOST: &str = match option_env!("ST_SERVER_HOST") {
    Some(h) => h,
    None => "",
};

pub const ST_API_SERVER: &str = match option_env!("ST_API_SERVER") {
    Some(a) => a,
    None => "",
};

pub const SERVER_KEY: &str = match option_env!("ST_SERVER_KEY") {
    Some(k) => k,
    None => "",
};

pub const SITE_HOME: &str = "https://www.sulltec.com/";

pub const DOWNLOAD: &str = "https://www.sulltec.com/SullTecRemote/download/";

pub const PRICING: &str = "https://www.sulltec.com/SullTecRemote/pricing/";

pub const PRIVACY: &str = "https://www.sulltec.com/privacy-policy/";

pub const DOCS_HOME: &str = "https://www.sulltec.com/SullTecRemote/docs/";

pub const DOCS_X11_REQUIRED: &str =
    "https://www.sulltec.com/SullTecRemote/docs/linux/#x11-required";

pub const DOCS_LINUX_LOGIN_SCREEN: &str =
    "https://www.sulltec.com/SullTecRemote/docs/linux/#login-screen";

pub const DOCS_LINUX_PERMISSIONS: &str =
    "https://www.sulltec.com/SullTecRemote/docs/linux/#permissions-issue";

pub const DOCS_MAC_PERMISSION: &str =
    "https://www.sulltec.com/SullTecRemote/docs/mac/#enable-permissions";

#[cfg(windows)]
pub fn machine_config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("ProgramData").map(|pd| {
        let mut p = std::path::PathBuf::from(pd);
        p.push(crate::config::APP_NAME.read().unwrap().clone());
        p.push("config");
        p
    })
}

/// Runs only when the shared dir has no config yet AND the old one carries a real identity. That
/// second condition is what stops an empty or secondary profile clobbering the authoritative one: the
/// SYSTEM service holds the registered identity and starts first, so it migrates its own config and
/// user sessions then simply read it.
#[cfg(windows)]
pub fn migrate_user_config_to_machine(
    machine_dir: &std::path::Path,
    old_dir: impl FnOnce() -> std::path::PathBuf,
) {
    static MIGRATED: std::sync::Once = std::sync::Once::new();
    let mut first = false;
    MIGRATED.call_once(|| first = true);
    if !first {
        return;
    }
    let main = format!("{}.toml", *crate::config::APP_NAME.read().unwrap());
    if machine_dir.join(&main).exists() {
        return;
    }
    let old_dir = old_dir();
    if old_dir.as_os_str().is_empty() {
        return;
    }
    if crate::config::load_path::<crate::config::Config>(old_dir.join(&main)).is_empty() {
        return;
    }
    if std::fs::create_dir_all(machine_dir).is_err() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(&old_dir) {
        for entry in entries.flatten() {
            let src = entry.path();
            if src.is_file() {
                let _ = std::fs::copy(&src, machine_dir.join(entry.file_name()));
            }
        }
    }
}

pub fn overwrite_settings() -> std::collections::HashMap<String, String> {
    let mut m: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if !ST_SERVER_HOST.is_empty() {
        m.insert("custom-rendezvous-server".to_owned(), ST_SERVER_HOST.to_owned());
        m.insert("relay-server".to_owned(), ST_SERVER_HOST.to_owned());
    }
    if !ST_API_SERVER.is_empty() {
        m.insert("api-server".to_owned(), ST_API_SERVER.to_owned());
    }
    m.insert("key".to_owned(), SERVER_KEY.to_owned());
    m
}

/// Upstream's `get_api_server` STRIPS `:21114` off any `https://` api-server URL unless this builtin
/// reads "Y".
pub fn builtin_settings() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::from([(
        crate::config::keys::OPTION_ALLOW_HTTPS_21114.to_owned(),
        "Y".to_owned(),
    )])
}

pub fn hide_cm_managed() -> bool {
    use crate::password_security::{approve_mode, keypair_only, ApproveMode};

    crate::config::option2bool("hide-cm", &crate::config::Config::get_option("hide-cm"))
        && (keypair_only() || approve_mode() != ApproveMode::Click)
}
