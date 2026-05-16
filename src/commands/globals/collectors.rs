use super::helpers::{
    entry_from_path, find_on_path, is_user_python_cli, matches_filter, push_bin_entries,
};
use super::{GlobalCliEntry, VersionContext};
use crate::error::Result;
use crate::tools::python;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(super) fn node(
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

pub(super) fn python(
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

pub(super) fn go(
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

pub(super) fn rust(
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

pub(super) fn java(
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
