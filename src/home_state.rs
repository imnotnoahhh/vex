use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditKind {
    SafeMigration,
    Advisory,
}

#[derive(Debug, Clone)]
pub struct HomeStateAudit {
    pub tool: &'static str,
    pub summary: &'static str,
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub kind: AuditKind,
    pub destination_exists: bool,
}

pub fn audit(home: &Path, tool_filter: Option<&str>) -> Vec<HomeStateAudit> {
    let filter = tool_filter.unwrap_or("all");
    definitions(home)
        .into_iter()
        .filter(|entry| filter == "all" || entry.tool == filter)
        .filter(|entry| entry.source.exists())
        .map(|mut entry| {
            entry.destination_exists = entry
                .destination
                .as_ref()
                .map(|path| path.exists())
                .unwrap_or(false);
            entry
        })
        .collect()
}

fn definitions(home: &Path) -> Vec<HomeStateAudit> {
    let vex = home.join(".vex");
    let go_root = home.join("go");
    let cache_dir = home.join(".cache");
    let library_caches = home.join("Library/Caches");

    vec![
        migratable(
            "legacy_tool_versions",
            "all",
            "legacy ~/.tool-versions can move into ~/.vex/tool-versions",
            home.join(".tool-versions"),
            vex.join("tool-versions"),
        ),
        migratable(
            "rust_cargo_home",
            "rust",
            "legacy Cargo home can move into ~/.vex/cargo",
            home.join(".cargo"),
            vex.join("cargo"),
        ),
        advisory(
            "rust_rustup_home",
            "rust",
            "rustup home is outside ~/.vex and needs a manual coexistence decision",
            home.join(".rustup"),
        ),
        migratable(
            "go_bin",
            "go",
            "legacy Go bin can move into ~/.vex/go/bin",
            go_root.join("bin"),
            vex.join("go/bin"),
        ),
        migratable(
            "go_mod_cache",
            "go",
            "legacy Go module cache can move into ~/.vex/go/pkg/mod",
            go_root.join("pkg/mod"),
            vex.join("go/pkg/mod"),
        ),
        migratable(
            "go_build_cache_home",
            "go",
            "legacy Go build cache can move into ~/.vex/go/cache",
            cache_dir.join("go-build"),
            vex.join("go/cache"),
        ),
        migratable(
            "go_build_cache_library",
            "go",
            "legacy Go build cache can move into ~/.vex/go/cache",
            library_caches.join("go-build"),
            vex.join("go/cache"),
        ),
        migratable(
            "npm_cache",
            "node",
            "legacy npm cache can move into ~/.vex/npm/cache",
            home.join(".npm"),
            vex.join("npm/cache"),
        ),
        advisory(
            "nvm_home",
            "node",
            "nvm installs are outside ~/.vex and need manual cleanup",
            home.join(".nvm"),
        ),
        advisory(
            "pnpm_store",
            "node",
            "pnpm store is outside ~/.vex and needs manual relocation",
            home.join(".pnpm-store"),
        ),
        advisory(
            "yarn_home",
            "node",
            "yarn state is outside ~/.vex and needs manual relocation",
            home.join(".yarn"),
        ),
        migratable(
            "pip_cache_home",
            "python",
            "legacy pip cache can move into ~/.vex/pip/cache",
            cache_dir.join("pip"),
            vex.join("pip/cache"),
        ),
        migratable(
            "pip_cache_library",
            "python",
            "legacy pip cache can move into ~/.vex/pip/cache",
            library_caches.join("pip"),
            vex.join("pip/cache"),
        ),
        advisory(
            "pyenv_home",
            "python",
            "pyenv installs are outside ~/.vex and need manual cleanup",
            home.join(".pyenv"),
        ),
    ]
}

fn migratable(
    _id: &'static str,
    tool: &'static str,
    summary: &'static str,
    source: PathBuf,
    destination: PathBuf,
) -> HomeStateAudit {
    HomeStateAudit {
        tool,
        summary,
        source,
        destination: Some(destination),
        kind: AuditKind::SafeMigration,
        destination_exists: false,
    }
}

fn advisory(
    _id: &'static str,
    tool: &'static str,
    summary: &'static str,
    source: PathBuf,
) -> HomeStateAudit {
    HomeStateAudit {
        tool,
        summary,
        source,
        destination: None,
        kind: AuditKind::Advisory,
        destination_exists: false,
    }
}
