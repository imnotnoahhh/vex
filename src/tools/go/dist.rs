use crate::tools::Arch;

pub(super) fn download_url(version: &str, arch: Arch) -> String {
    format!(
        "https://go.dev/dl/{}.darwin-{}.tar.gz",
        ensure_go_prefix(version),
        arch_suffix(arch)
    )
}

pub(super) fn ensure_go_prefix(version: &str) -> String {
    if version.starts_with("go") {
        version.to_string()
    } else {
        format!("go{}", version)
    }
}

pub(super) fn strip_go_prefix(version: &str) -> String {
    version.strip_prefix("go").unwrap_or(version).to_string()
}

pub(super) fn arch_suffix(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "arm64",
        Arch::X86_64 => "amd64",
    }
}

pub(super) fn is_supported_minor_alias(alias: &str) -> bool {
    if !alias.contains('.') {
        return false;
    }

    let parts: Vec<&str> = alias.split('.').collect();
    parts.len() == 2 && (parts[1] == "x" || parts[1].chars().all(|ch| ch.is_ascii_digit()))
}
