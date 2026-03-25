//! Java (Eclipse Temurin JDK) tool implementation
//!
//! Uses Adoptium API v3 to query versions, only supports JDK + HotSpot combination.
//! macOS JDK directory structure is special: `Contents/Home/bin/`.

mod api;
mod resolve;

use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool, Version};
use api::{fetch_available_releases, fetch_temurin_releases};
use resolve::{build_remote_versions, resolve_alias_version};
#[cfg(test)]
mod tests;

/// Java (Eclipse Temurin JDK) tool
pub struct JavaTool;

impl Tool for JavaTool {
    fn name(&self) -> &str {
        "java"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let releases = fetch_available_releases()?;
        Ok(build_remote_versions(&releases))
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        let releases = fetch_temurin_releases(version, arch)?;
        if let Some(release) = releases.first() {
            Ok(release.binary.package.link.clone())
        } else {
            Err(VexError::VersionNotFound {
                tool: "java".to_string(),
                version: version.to_string(),
                suggestions: String::new(),
            })
        }
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // Eclipse Temurin's SHA256 is directly in API
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec![
            "java",
            "javac",
            "jar",
            "javadoc",
            "javap",
            "jcmd",
            "jconsole",
            "jdb",
            "jdeprscan",
            "jdeps",
            "jfr",
            "jhsdb",
            "jimage",
            "jinfo",
            "jlink",
            "jmap",
            "jmod",
            "jnativescan",
            "jpackage",
            "jps",
            "jrunscript",
            "jshell",
            "jstack",
            "jstat",
            "jstatd",
            "jwebserver",
            "keytool",
            "rmiregistry",
            "serialver",
            "jarsigner",
        ]
    }

    fn bin_subpath(&self) -> &str {
        // macOS JDK directory structure is special: jdk-21.0.10+7/Contents/Home/bin
        "Contents/Home/bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let releases = fetch_temurin_releases(version, arch)?;
        if let Some(release) = releases.first() {
            Ok(Some(release.binary.package.checksum.clone()))
        } else {
            Ok(None)
        }
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        let versions = self.list_remote()?;
        resolve_alias_version(self, alias, &versions)
    }
}
