use crate::tools::Arch;

pub(crate) fn download_url(version: &str, arch: Arch) -> String {
    format!(
        "https://static.rust-lang.org/dist/rust-{}-{}.tar.gz",
        version,
        target_triple(arch)
    )
}

pub(crate) fn checksum_url(version: &str, arch: Arch) -> String {
    format!("{}.sha256", download_url(version, arch))
}

pub(crate) fn parse_sha256_sidecar(content: &str) -> Option<String> {
    content
        .split_whitespace()
        .next()
        .map(|value| value.to_string())
}

pub(crate) fn target_triple(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "aarch64-apple-darwin",
        Arch::X86_64 => "x86_64-apple-darwin",
    }
}
