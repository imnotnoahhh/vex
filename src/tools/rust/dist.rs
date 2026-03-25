use crate::tools::Arch;

pub(super) fn download_url(version: &str, arch: Arch) -> String {
    format!(
        "https://static.rust-lang.org/dist/rust-{}-{}.tar.gz",
        version,
        target_triple(arch)
    )
}

pub(super) fn checksum_url(version: &str, arch: Arch) -> String {
    format!("{}.sha256", download_url(version, arch))
}

pub(super) fn parse_sha256_sidecar(content: &str) -> Option<String> {
    content
        .split_whitespace()
        .next()
        .map(|value| value.to_string())
}

pub(super) fn target_triple(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "aarch64-apple-darwin",
        Arch::X86_64 => "x86_64-apple-darwin",
    }
}
