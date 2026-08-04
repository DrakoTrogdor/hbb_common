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
