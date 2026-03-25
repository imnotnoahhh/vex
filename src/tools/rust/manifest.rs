use crate::error::{Result, VexError};
use crate::http;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RustManifest {
    pkg: Packages,
}

#[derive(Deserialize, Debug)]
struct Packages {
    rust: RustPackage,
}

#[derive(Deserialize, Debug)]
struct RustPackage {
    version: String,
}

pub(super) fn fetch_stable_version() -> Result<String> {
    let content = http::get_text_in_current_context(
        "https://static.rust-lang.org/dist/channel-rust-stable.toml",
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )?;
    let manifest: RustManifest = toml::from_str(&content)
        .map_err(|err| VexError::Parse(format!("Failed to parse Rust manifest: {}", err)))?;

    let version_str = manifest.pkg.rust.version;
    Ok(version_str
        .split_whitespace()
        .next()
        .unwrap_or(&version_str)
        .to_string())
}
