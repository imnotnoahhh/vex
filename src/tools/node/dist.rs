use crate::tools::Arch;

pub(super) fn download_url(version: &str, arch: Arch) -> String {
    let version = prefixed_version(version);
    format!(
        "https://nodejs.org/dist/{}/node-{}-darwin-{}.tar.gz",
        version,
        version,
        arch_suffix(arch)
    )
}

pub(super) fn checksum_url(version: &str) -> String {
    format!(
        "https://nodejs.org/dist/{}/SHASUMS256.txt",
        prefixed_version(version)
    )
}

pub(super) fn find_checksum(content: &str, version: &str, arch: Arch) -> Option<String> {
    let filename = format!(
        "node-{}-darwin-{}.tar.gz",
        prefixed_version(version),
        arch_suffix(arch)
    );

    content
        .lines()
        .find(|line| line.contains(&filename))
        .and_then(|line| line.split_whitespace().next())
        .map(str::to_string)
}

fn prefixed_version(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    }
}

fn arch_suffix(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "arm64",
        Arch::X86_64 => "x64",
    }
}
