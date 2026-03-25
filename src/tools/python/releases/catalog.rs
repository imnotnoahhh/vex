use super::super::lifecycle::SupportStatus;
use crate::tools::Arch;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(in crate::tools::python::releases) fn asset_filename(
    version: &str,
    tag: &str,
    arch: Arch,
) -> String {
    let arch_str = match arch {
        Arch::Arm64 => "aarch64-apple-darwin",
        Arch::X86_64 => "x86_64-apple-darwin",
    };
    format!(
        "cpython-{}+{}-{}-install_only.tar.gz",
        version, tag, arch_str
    )
}

pub(in crate::tools::python::releases) fn find_matching_checksum(
    content: &str,
    filename: &str,
) -> Option<String> {
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() == 2 && parts[1] == filename {
            return Some(parts[0].to_string());
        }
    }
    None
}

pub(in crate::tools::python::releases) fn extract_python_version(
    asset_name: &str,
) -> Option<String> {
    let without_prefix = asset_name.strip_prefix("cpython-")?;
    let version_part = without_prefix.split('+').next()?;
    Some(version_part.to_string())
}

pub(in crate::tools::python::releases) fn get_major_minor(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

pub(in crate::tools::python::releases) fn collect_available_versions(content: &str) -> Vec<String> {
    let mut versions = BTreeSet::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            continue;
        }
        let filename = parts[1];
        if filename.contains("apple-darwin")
            && filename.ends_with("install_only.tar.gz")
            && !filename.contains("stripped")
        {
            if let Some(version) = extract_python_version(filename) {
                versions.insert(version);
            }
        }
    }

    let mut versions: Vec<String> = versions.into_iter().collect();
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|part| part.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|part| part.parse().ok()).collect();
        b_parts.cmp(&a_parts)
    });
    versions
}

pub(in crate::tools::python::releases) fn lifecycle_status_for(
    version: &str,
    lifecycle_statuses: &BTreeMap<String, String>,
) -> String {
    let major_minor = get_major_minor(version);
    lifecycle_statuses
        .get(&major_minor)
        .cloned()
        .unwrap_or_else(|| {
            SupportStatus::from_version(&major_minor)
                .as_str()
                .to_string()
        })
}
