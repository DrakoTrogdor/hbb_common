//! The fork's own values in `hbb_common`.
//!
//! Upstream owns everything else in this crate. What lives here is what SullTec Remote adds or
//! substitutes: the deployment's server addresses and key, the docs links, and the two derived
//! forms of the product name used for paths.
//!
//! Several of these are read through names upstream already owns — `RENDEZVOUS_SERVERS`,
//! `RS_PUB_KEY`, `LINK_DOCS_HOME`, `LINK_DOCS_X11_REQUIRED`. Those declarations must stay in
//! `config.rs` where upstream's code expects them, so only the *values* moved here and the
//! declarations became one-line aliases. Anything whose name upstream does not know lives here
//! outright.

/// The rendezvous host, supplied at COMPILE TIME via `ST_SERVER_HOST`.
///
/// Deliberately not a literal, for the same two reasons as [`SERVER_KEY`]. It named a specific
/// deployment inside a submodule of a repo that publishes, and rebranding meant hand-editing source
/// in four places with nothing to warn a build that missed one.
///
/// Unset resolves to empty, which fails CLOSED: the client finds no server and simply never
/// connects, rather than reaching some default that is not ours. `Build-Release.ps1` refuses to
/// produce a release artifact without it; plain `cargo check`/`build` still work for development.
pub const ST_SERVER_HOST: &str = match option_env!("ST_SERVER_HOST") {
    Some(h) => h,
    None => "",
};

/// The client API base URL, supplied at COMPILE TIME via `ST_API_SERVER`. Normally
/// `https://<ST_SERVER_HOST>:21114`, which is what `Build-Release.ps1` derives when it is not
/// configured separately; it is its own variable because the API can legitimately sit on a
/// different port or name than the rendezvous service.
pub const ST_API_SERVER: &str = match option_env!("ST_API_SERVER") {
    Some(a) => a,
    None => "",
};

/// The rendezvous server's public key, supplied at COMPILE TIME via `ST_SERVER_KEY`.
///
/// It is deliberately not a literal. hbbs compares it as a bearer string
/// (`if !key.is_empty() && ph.licence_key != key`), so committing it to a public repo hands anyone
/// who reads the source the ability to use the rendezvous and relay servers. Keeping it out of the
/// tree also removes the rotation footgun this used to carry: the value existed in two places, both
/// inside a submodule, with nothing to warn a build that skipped editing them — a rebuild would
/// silently ship the previous key.
///
/// Unset resolves to empty, which fails CLOSED: hbbs holds a non-empty key, so an empty
/// `licence_key` is refused with LICENSE_MISMATCH rather than connecting to the wrong place.
/// `Build-Release.ps1` refuses to produce a release artifact when it is empty; plain `cargo
/// check`/`build` still work for development.
///
/// Read through `config::RS_PUB_KEY`, which is upstream's name for it.
pub const SERVER_KEY: &str = match option_env!("ST_SERVER_KEY") {
    Some(k) => k,
    None => "",
};

/// Documentation root. Read through `config::LINK_DOCS_HOME`, which is upstream's name for it.
pub const DOCS_HOME: &str = "https://www.sulltec.com/docs/en/";

/// Read through `config::LINK_DOCS_X11_REQUIRED`, which is upstream's name for it.
pub const DOCS_X11_REQUIRED: &str = "https://www.sulltec.com/docs/en/manual/linux/#x11-required";

/// Folder form of the product name: "SullTec Remote" -> "SullTecRemote".
///
/// The display name (`config::APP_NAME`) is what end users read; on disk it splits into two derived
/// forms so no path ever carries the space. Upstream gets away with using the display name directly
/// because "RustDesk" and "rustdesk.exe" differ only by case and NTFS ignores that — a hyphenated
/// rename does not.
///
/// Identifiers that are neither a folder nor a file — the Windows service name, the URI scheme,
/// named pipes — keep their own forms via `get_app_ident()` and are unaffected.
#[inline]
pub fn app_dir_name() -> String {
    crate::config::APP_NAME.read().unwrap().replace(' ', "")
}

/// File-stem form of the product name: "SullTec Remote" -> "sulltec-remote".
///
/// The binary is `sulltec-remote.exe`; config files are `sulltec-remote.toml` / `_ab` / `_group`.
#[inline]
pub fn app_file_base() -> String {
    crate::config::APP_NAME
        .read()
        .unwrap()
        .to_lowercase()
        .replace(' ', "-")
}

/// The machine-wide config directory `%ProgramData%\<app_dir_name>\config`.
///
/// The SYSTEM service and every interactive or RDS user session read the SAME config here, so the box
/// keeps ONE identity instead of a per-user one each. `%ProgramData%` is service- and admin-writable
/// but only user-readable, so a session reads the managed identity and cannot tamper with it.
///
/// `None` when `%ProgramData%` is unset, in which case the caller falls back to the per-user dir.
#[cfg(windows)]
pub fn machine_config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("ProgramData").map(|pd| {
        let mut p = std::path::PathBuf::from(pd);
        p.push(app_dir_name());
        p.push("config");
        p
    })
}

/// Move an existing per-user or service-profile config into the machine-wide dir, once, so a box that
/// was already enrolled keeps its RustDesk ID and keypair when it converts to shared config. This
/// converges on a service RESTART; no reboot is needed.
///
/// `old_dir` is passed as a closure because resolving it needs `config`'s own private path
/// patching, and it must not run on the hot path when the migration has already happened.
///
/// Runs only when the shared dir has no config yet AND the old one carries a real identity. That
/// second condition is what stops an empty or secondary profile clobbering the authoritative one: the
/// SYSTEM service holds the registered identity and starts first, so it migrates its own config and
/// user sessions then simply read it.
#[cfg(windows)]
pub fn migrate_user_config_to_machine(
    machine_dir: &std::path::Path,
    old_dir: impl FnOnce() -> std::path::PathBuf,
) {
    // Once per process, and the source directory is resolved lazily inside — `Config::path` is called
    // constantly, and neither the resolution nor the directory walk below should run on every call.
    static MIGRATED: std::sync::Once = std::sync::Once::new();
    let mut first = false;
    MIGRATED.call_once(|| first = true);
    if !first {
        return;
    }
    let main = format!("{}.toml", app_file_base());
    if machine_dir.join(&main).exists() {
        return; // shared config already populated
    }
    let old_dir = old_dir();
    if old_dir.as_os_str().is_empty() {
        return;
    }
    // Only adopt a config that actually holds an identity (id/enc_id + keypair).
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

/// The initial contents of `config::OVERWRITE_SETTINGS` — the FORCED server configuration.
///
/// This map is the top layer in `Config::get_option` (`get_or` checks it before saved options and
/// before defaults), so these entries win over ANY saved or IP config on a deployed client, including
/// one that already had RustDesk pointed somewhere else.
///
/// An unset compile-time value is OMITTED rather than inserted empty. Because this is the top layer,
/// an empty entry would outrank both a saved config and a policy — turning "this build was not told
/// where its server is" into "this device has no server", which no later policy could repair.
pub fn overwrite_settings() -> std::collections::HashMap<String, String> {
    let mut m: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if !ST_SERVER_HOST.is_empty() {
        m.insert("custom-rendezvous-server".to_owned(), ST_SERVER_HOST.to_owned());
        m.insert("relay-server".to_owned(), ST_SERVER_HOST.to_owned());
    }
    // https, so a FRESH INSTALL is TLS-native before it has ever spoken to the console. This is only
    // ever the value a device uses when it carries no `api-server` policy — every managed device is
    // told explicitly, and a LOCKED policy value overwrites this entry (both live in this same map,
    // and the policy mirror inserts over the seed on load). So changing it moves nobody who is
    // already enrolled; it decides where a device points before policy reaches it.
    //
    // It must land BEFORE plaintext is ever refused on the client port: a device installed after that
    // point would otherwise boot on http, be refused, and never enrol — stranded somewhere the
    // console has never seen it.
    if !ST_API_SERVER.is_empty() {
        m.insert("api-server".to_owned(), ST_API_SERVER.to_owned());
    }
    // The key is inserted unconditionally, unlike the addresses above. This is the EFFECTIVE value
    // (overwrite outranks saved config, and the fallback is only consulted when this is absent), so
    // an empty one has to reach hbbs and be REFUSED with LICENSE_MISMATCH. Omitting it would instead
    // let a saved key from some earlier configuration answer in its place.
    m.insert("key".to_owned(), SERVER_KEY.to_owned());
    m
}

/// The initial contents of `config::BUILTIN_SETTINGS` — pre-seeded rather than empty.
///
/// Upstream's `get_api_server` STRIPS `:21114` off any `https://` api-server URL unless this builtin
/// reads "Y". Without it a client pointed at `https://<host>:21114` silently retargets port 443,
/// which is not a clean failure on a typical network: 443 usually forwards to an unrelated host, so
/// the device talks to somebody else's service and the symptom appears over there rather than here.
///
/// Upstream populates this map only from the signed `custom.txt` path, which this fork does not use —
/// it bakes into [`overwrite_settings`] instead — so seeding the initial value is the way in.
///
/// This enables nothing on its own: the `api-server` default above decides the scheme, and a device
/// only moves to https when a locked policy tells it to.
pub fn builtin_settings() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::from([(
        crate::config::keys::OPTION_ALLOW_HTTPS_21114.to_owned(),
        "Y".to_owned(),
    )])
}

/// Should a console-managed endpoint hide the connection-manager window?
///
/// A console-pushed `hide-cm` hides the CM whenever the endpoint never needs an INTERACTIVE accept
/// click: keypair-only logon (every connection is key-authorized and auto-accepts, bypassing
/// approve-mode), or any approve-mode that is not "click" (password auto-accepts, and console
/// key-pair logon likewise). Gated on exactly that, so a prompt the user is meant to click is never
/// silently hidden out from under them.
pub fn hide_cm_managed() -> bool {
    use crate::password_security::{approve_mode, keypair_only, ApproveMode};

    crate::config::option2bool("hide-cm", &crate::config::Config::get_option("hide-cm"))
        && (keypair_only() || approve_mode() != ApproveMode::Click)
}
