use crate::commands::current;
use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use crate::tools::python::{self, PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS};
use crate::ui;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct GlobalCliEntry {
    pub tool: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_source_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GlobalsReport {
    pub cwd: String,
    pub entries: Vec<GlobalCliEntry>,
}

#[derive(Debug, Clone)]
struct VersionContext {
    version: String,
    source: String,
    source_path: Option<String>,
}

pub fn show(tool_filter: Option<&str>, output: OutputMode, verbose: bool) -> Result<()> {
    let report = collect(tool_filter)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_text(&report, verbose);
            Ok(())
        }
    }
}

pub fn collect(tool_filter: Option<&str>) -> Result<GlobalsReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let cwd = std::env::current_dir()?;
    let contexts = current_contexts().unwrap_or_default();
    let mut entries = Vec::new();

    collect_node_entries(&vex_dir, &contexts, tool_filter, &mut entries);
    collect_python_entries(&vex_dir, &contexts, tool_filter, &mut entries)?;
    collect_go_entries(&vex_dir, &contexts, tool_filter, &mut entries);
    collect_rust_entries(&vex_dir, &contexts, tool_filter, &mut entries);
    collect_java_entries(&contexts, tool_filter, &mut entries);

    entries.sort_by(|left, right| {
        left.tool
            .cmp(&right.tool)
            .then(left.kind.cmp(&right.kind))
            .then(left.name.cmp(&right.name))
            .then(left.path.cmp(&right.path))
    });

    Ok(GlobalsReport {
        cwd: cwd.display().to_string(),
        entries,
    })
}

fn current_contexts() -> Result<BTreeMap<String, VersionContext>> {
    Ok(current::collect_current()?
        .tools
        .into_iter()
        .map(|entry| {
            (
                entry.tool,
                VersionContext {
                    version: entry.version,
                    source: entry.source,
                    source_path: entry.source_path,
                },
            )
        })
        .collect())
}

fn collect_node_entries(
    vex_dir: &Path,
    contexts: &BTreeMap<String, VersionContext>,
    filter: Option<&str>,
    entries: &mut Vec<GlobalCliEntry>,
) {
    if !matches_filter(filter, "node", "") {
        return;
    }
    let bin_dir = vex_dir.join("npm/prefix/bin");
    push_bin_entries(
        entries,
        "node",
        "npm_global",
        "shared npm globals",
        &bin_dir,
        contexts.get("node"),
        |_| true,
    );
}

fn collect_python_entries(
    vex_dir: &Path,
    contexts: &BTreeMap<String, VersionContext>,
    filter: Option<&str>,
    entries: &mut Vec<GlobalCliEntry>,
) -> Result<()> {
    if !matches_filter(filter, "python", "") {
        return Ok(());
    }

    let base_root = vex_dir.join("python/base");
    if base_root.exists() {
        for version_entry in fs::read_dir(base_root)?.filter_map(|entry| entry.ok()) {
            let Ok(file_type) = version_entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let version = version_entry.file_name().to_string_lossy().to_string();
            let bin_dir = python::base_bin_dir(vex_dir, &version);
            let active_context = contexts
                .get("python")
                .filter(|context| context.version == version);
            push_bin_entries(
                entries,
                "python",
                "python_base",
                "Python base environment (pip)",
                &bin_dir,
                active_context,
                is_user_python_cli,
            );

            for entry in entries.iter_mut().filter(|entry| {
                entry.tool == "python"
                    && entry.kind == "python_base"
                    && entry.path.starts_with(&bin_dir.display().to_string())
            }) {
                entry.tool_version = Some(version.clone());
            }
        }
    }

    push_bin_entries(
        entries,
        "python",
        "python_user_base",
        "Python user base (pip --user)",
        &python::user_bin_dir(vex_dir),
        contexts.get("python"),
        is_user_python_cli,
    );

    Ok(())
}

fn collect_go_entries(
    vex_dir: &Path,
    contexts: &BTreeMap<String, VersionContext>,
    filter: Option<&str>,
    entries: &mut Vec<GlobalCliEntry>,
) {
    if !matches_filter(filter, "go", "") {
        return;
    }
    let bin_dir = vex_dir.join("go/bin");
    push_bin_entries(
        entries,
        "go",
        "go_global",
        "managed GOBIN (go install)",
        &bin_dir,
        contexts.get("go"),
        |_| true,
    );
}

fn collect_rust_entries(
    vex_dir: &Path,
    contexts: &BTreeMap<String, VersionContext>,
    filter: Option<&str>,
    entries: &mut Vec<GlobalCliEntry>,
) {
    if !matches_filter(filter, "rust", "") {
        return;
    }
    let bin_dir = vex_dir.join("cargo/bin");
    push_bin_entries(
        entries,
        "rust",
        "cargo_global",
        "managed CARGO_HOME bin (cargo install)",
        &bin_dir,
        contexts.get("rust"),
        |_| true,
    );
}

fn collect_java_entries(
    contexts: &BTreeMap<String, VersionContext>,
    filter: Option<&str>,
    entries: &mut Vec<GlobalCliEntry>,
) {
    if !matches_filter(filter, "java", "") {
        return;
    }

    let context = contexts.get("java");
    let mut seen_paths = BTreeSet::new();
    for (name, source) in [
        ("mvn", "external Maven CLI on PATH"),
        ("gradle", "external Gradle CLI on PATH"),
    ] {
        if !matches_filter(filter, "java", name) {
            continue;
        }
        if let Some(path) = find_on_path(name) {
            if seen_paths.insert(path.clone()) {
                entries.push(entry_from_path(
                    "java",
                    name,
                    if name == "mvn" {
                        "maven_cli"
                    } else {
                        "gradle_cli"
                    },
                    source,
                    &path,
                    context,
                ));
            }
        }
    }

    let Some(home) = dirs::home_dir() else {
        return;
    };
    for (name, kind, source, path) in [
        (
            "maven-local-repository",
            "maven_state",
            "Maven local repository outside vex",
            home.join(".m2/repository"),
        ),
        (
            "gradle-caches",
            "gradle_state",
            "Gradle caches outside vex",
            home.join(".gradle/caches"),
        ),
        (
            "gradle-wrapper-cache",
            "gradle_state",
            "Gradle wrapper distributions outside vex",
            home.join(".gradle/wrapper"),
        ),
    ] {
        if path.exists() && matches_filter(filter, "java", name) {
            entries.push(entry_from_path("java", name, kind, source, &path, context));
        }
    }
}

fn push_bin_entries(
    entries: &mut Vec<GlobalCliEntry>,
    tool: &str,
    kind: &str,
    source: &str,
    bin_dir: &Path,
    context: Option<&VersionContext>,
    include_name: impl Fn(&str) -> bool,
) {
    if !bin_dir.exists() {
        return;
    }

    let Ok(read_dir) = fs::read_dir(bin_dir) else {
        return;
    };

    for entry in read_dir.filter_map(|entry| entry.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !include_name(&name) {
            continue;
        }
        let path = entry.path();
        if is_executable_file(&path) {
            entries.push(entry_from_path(tool, &name, kind, source, &path, context));
        }
    }
}

fn entry_from_path(
    tool: &str,
    name: &str,
    kind: &str,
    source: &str,
    path: &Path,
    context: Option<&VersionContext>,
) -> GlobalCliEntry {
    GlobalCliEntry {
        tool: tool.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
        path: path.display().to_string(),
        source: source.to_string(),
        tool_version: context.map(|context| context.version.clone()),
        version_source: context.map(|context| context.source.clone()),
        version_source_path: context.and_then(|context| context.source_path.clone()),
    }
}

fn is_user_python_cli(name: &str) -> bool {
    !(name == "activate"
        || name.starts_with("activate.")
        || name.starts_with("Activate.")
        || name == "python"
        || name.starts_with("python3")
        || name == PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS
        || name == "pip"
        || name.starts_with("pip3"))
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.is_dir() {
        return false;
    }
    metadata.permissions().mode() & 0o111 != 0
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    std::env::var("PATH").ok()?.split(':').find_map(|entry| {
        if entry.is_empty() {
            return None;
        }
        let candidate = Path::new(entry).join(name);
        is_executable_file(&candidate).then_some(candidate)
    })
}

fn matches_filter(filter: Option<&str>, tool: &str, name: &str) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let filter = filter.to_ascii_lowercase();
    match filter.as_str() {
        "all" => true,
        "npm" => tool == "node",
        "pip" => tool == "python",
        "cargo" => tool == "rust",
        "maven" | "mvn" => {
            tool == "java" && (name.is_empty() || name.contains("maven") || name == "mvn")
        }
        "gradle" => tool == "java" && (name.is_empty() || name.contains("gradle")),
        _ => filter == tool || (!name.is_empty() && filter == name),
    }
}

fn render_text(report: &GlobalsReport, verbose: bool) {
    if report.entries.is_empty() {
        ui::dimmed("No global CLI entries detected.");
        println!();
        ui::dimmed(
            "Install a global CLI with shared npm globals, Go, Cargo, or 'vex python base pip'.",
        );
        return;
    }

    ui::header("Global CLIs and Build Tool State");
    let mut table = ui::Table::new();
    for entry in &report.entries {
        let version_context = entry
            .tool_version
            .as_ref()
            .map(|version| {
                format!(
                    "{} ({})",
                    version,
                    entry.version_source.as_deref().unwrap_or("unknown source")
                )
            })
            .unwrap_or_else(|| "n/a".to_string());
        table = table.row(vec![
            entry.tool.yellow().to_string(),
            entry.name.cyan().to_string(),
            entry.source.clone(),
            version_context.dimmed().to_string(),
        ]);
        if verbose {
            table = table.row(vec![
                "".to_string(),
                "".to_string(),
                format!("{}: {}", "Path".dimmed(), entry.path.dimmed()),
                entry
                    .version_source_path
                    .as_ref()
                    .map(|path| format!("{}: {}", "Version source".dimmed(), path.dimmed()))
                    .unwrap_or_default(),
            ]);
        }
    }
    table.render();
    if report
        .entries
        .iter()
        .any(|entry| entry.tool == "node" && entry.kind == "npm_global")
    {
        ui::dimmed(
            "Node npm globals are shared across vex-managed Node versions; project node_modules/.bin still wins when present.",
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_executable(path: &Path) {
        fs::write(path, "#!/bin/sh\n").unwrap();
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn collect_reports_node_shared_npm_globals() {
        let _guard = ENV_LOCK.lock().unwrap();
        let home = TempDir::new().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        let bin_dir = home.path().join(".vex/npm/prefix/bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_executable(&bin_dir.join("tsx"));

        let report = collect(Some("node")).unwrap();
        let tsx = report
            .entries
            .iter()
            .find(|entry| entry.tool == "node" && entry.name == "tsx")
            .expect("npm global CLI should be reported");
        assert_eq!(tsx.kind, "npm_global");
        assert_eq!(tsx.source, "shared npm globals");

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn collect_reports_go_and_rust_global_bins() {
        let _guard = ENV_LOCK.lock().unwrap();
        let home = TempDir::new().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        let go_bin = home.path().join(".vex/go/bin");
        let cargo_bin = home.path().join(".vex/cargo/bin");
        fs::create_dir_all(&go_bin).unwrap();
        fs::create_dir_all(&cargo_bin).unwrap();
        write_executable(&go_bin.join("gopls"));
        write_executable(&cargo_bin.join("cargo-audit"));

        let report = collect(None).unwrap();
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.tool == "go" && entry.name == "gopls"));
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.tool == "rust" && entry.name == "cargo-audit"));

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn collect_reports_java_build_state() {
        let _guard = ENV_LOCK.lock().unwrap();
        let home = TempDir::new().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        fs::create_dir_all(home.path().join(".m2/repository")).unwrap();
        fs::create_dir_all(home.path().join(".gradle/caches")).unwrap();

        let report = collect(Some("java")).unwrap();
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.name == "maven-local-repository"));
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.name == "gradle-caches"));

        let mvn_report = collect(Some("mvn")).unwrap();
        assert!(mvn_report
            .entries
            .iter()
            .any(|entry| entry.name == "maven-local-repository"));

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn collect_python_base_reports_user_clis_only() {
        let _guard = ENV_LOCK.lock().unwrap();
        let home = TempDir::new().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        let bin_dir = home.path().join(".vex/python/base/3.14.4/bin");
        let user_bin_dir = home.path().join(".vex/python/user/bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&user_bin_dir).unwrap();
        write_executable(&bin_dir.join("kaggle"));
        write_executable(&bin_dir.join("pip"));
        write_executable(&bin_dir.join("python3.14"));
        write_executable(&bin_dir.join(PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS));
        write_executable(&user_bin_dir.join("black"));

        let report = collect(Some("python")).unwrap();
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.tool == "python" && entry.name == "kaggle"));
        assert!(report.entries.iter().any(|entry| {
            entry.tool == "python"
                && entry.kind == "python_user_base"
                && entry.name == "black"
                && entry.source == "Python user base (pip --user)"
        }));
        assert!(!report.entries.iter().any(|entry| entry.name == "pip"
            || entry.name == "python3.14"
            || entry.name == "\u{1d70b}thon"));

        let pip_report = collect(Some("pip")).unwrap();
        assert!(pip_report
            .entries
            .iter()
            .any(|entry| entry.name == "kaggle"));
        assert!(pip_report.entries.iter().any(|entry| entry.name == "black"));

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn collect_accepts_official_package_manager_filters() {
        let _guard = ENV_LOCK.lock().unwrap();
        let home = TempDir::new().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        let npm_bin = home.path().join(".vex/npm/prefix/bin");
        let cargo_bin = home.path().join(".vex/cargo/bin");
        fs::create_dir_all(&npm_bin).unwrap();
        fs::create_dir_all(&cargo_bin).unwrap();
        write_executable(&npm_bin.join("eslint"));
        write_executable(&cargo_bin.join("cargo-audit"));

        let npm_report = collect(Some("npm")).unwrap();
        assert!(npm_report
            .entries
            .iter()
            .any(|entry| entry.tool == "node" && entry.name == "eslint"));

        let cargo_report = collect(Some("cargo")).unwrap();
        assert!(cargo_report
            .entries
            .iter()
            .any(|entry| entry.tool == "rust" && entry.name == "cargo-audit"));

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
