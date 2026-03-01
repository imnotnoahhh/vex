use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::os::unix::fs as unix_fs;
use tempfile::TempDir;

// Import vex modules for benchmarking
// Note: These are internal modules, so we need to access them through the crate

/// Helper to create a temporary directory with version files
fn setup_version_files(depth: usize) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let mut current = tmp.path().to_path_buf();

    // Create nested directories
    for i in 0..depth {
        current = current.join(format!("level{}", i));
        fs::create_dir_all(&current).unwrap();
    }

    // Write .tool-versions at the root
    fs::write(
        tmp.path().join(".tool-versions"),
        "node 20.11.0\ngo 1.23.5\njava 21\nrust 1.93.1\n",
    )
    .unwrap();

    // Write language-specific files
    fs::write(tmp.path().join(".node-version"), "20.11.0\n").unwrap();
    fs::write(tmp.path().join(".go-version"), "1.23.5\n").unwrap();
    fs::write(tmp.path().join(".java-version"), "21\n").unwrap();
    fs::write(tmp.path().join(".rust-toolchain"), "1.93.1\n").unwrap();

    tmp
}

/// Helper to setup a fake toolchain directory for switching benchmarks
fn setup_toolchain_dir() -> TempDir {
    let tmp = TempDir::new().unwrap();

    // Create fake node toolchain
    let node_bin = tmp.path().join("toolchains/node/20.11.0/bin");
    fs::create_dir_all(&node_bin).unwrap();
    for name in &["node", "npm", "npx"] {
        fs::write(node_bin.join(name), "fake binary").unwrap();
    }

    // Create fake go toolchain
    let go_bin = tmp.path().join("toolchains/go/1.23.5/bin");
    fs::create_dir_all(&go_bin).unwrap();
    for name in &["go", "gofmt"] {
        fs::write(go_bin.join(name), "fake binary").unwrap();
    }

    tmp
}

/// Benchmark: Parse .tool-versions file content
fn bench_parse_tool_versions(c: &mut Criterion) {
    let content = "# Project versions\nnode 20.11.0\ngo 1.23.5\njava 21\nrust 1.93.1\n\n# Python coming soon\n";

    c.bench_function("parse_tool_versions", |b| {
        b.iter(|| {
            // Inline parsing logic to benchmark
            let result: Vec<(String, String)> = content
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        return None;
                    }
                    let mut parts = line.split_whitespace();
                    let tool = parts.next()?;
                    let version = parts.next()?;
                    Some((tool.to_string(), version.to_string()))
                })
                .collect();
            black_box(result)
        });
    });
}

/// Benchmark: Resolve version from nested directories
fn bench_resolve_version_deep_tree(c: &mut Criterion) {
    let tmp = setup_version_files(10); // 10 levels deep
    let deepest = tmp.path()
        .join("level0/level1/level2/level3/level4/level5/level6/level7/level8/level9");

    c.bench_function("resolve_version_deep_tree", |b| {
        b.iter(|| {
            // Simulate directory traversal
            let mut dir = deepest.clone();
            let mut found = None;

            loop {
                let tool_versions = dir.join(".tool-versions");
                if tool_versions.is_file() {
                    if let Ok(content) = fs::read_to_string(&tool_versions) {
                        for line in content.lines() {
                            let line = line.trim();
                            if line.starts_with("node ") {
                                found = Some(line.split_whitespace().nth(1).unwrap().to_string());
                                break;
                            }
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }

                if !dir.pop() {
                    break;
                }
            }

            black_box(found)
        });
    });
}

/// Benchmark: Resolve all versions from a directory
fn bench_resolve_all_versions(c: &mut Criterion) {
    let tmp = setup_version_files(5);
    let start_dir = tmp.path().join("level0/level1/level2/level3/level4");

    c.bench_function("resolve_all_versions", |b| {
        b.iter(|| {
            let mut versions = std::collections::HashMap::new();
            let mut dir = start_dir.clone();

            loop {
                // Check .tool-versions
                let tool_versions = dir.join(".tool-versions");
                if tool_versions.is_file() {
                    if let Ok(content) = fs::read_to_string(&tool_versions) {
                        for line in content.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
                                continue;
                            }
                            let mut parts = line.split_whitespace();
                            if let (Some(tool), Some(version)) = (parts.next(), parts.next()) {
                                versions.entry(tool.to_string()).or_insert(version.to_string());
                            }
                        }
                    }
                }

                // Check language-specific files
                for (file, tool) in &[
                    (".node-version", "node"),
                    (".go-version", "go"),
                    (".java-version", "java"),
                    (".rust-toolchain", "rust"),
                ] {
                    let path = dir.join(file);
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let version = content.trim().to_string();
                            if !version.is_empty() {
                                versions.entry(tool.to_string()).or_insert(version);
                            }
                        }
                    }
                }

                if !dir.pop() {
                    break;
                }
            }

            black_box(versions)
        });
    });
}

/// Benchmark: Create and update symlinks (version switching)
fn bench_switch_symlinks(c: &mut Criterion) {
    let tmp = setup_toolchain_dir();
    let base = tmp.path();

    c.bench_function("switch_symlinks", |b| {
        b.iter(|| {
            let toolchain_dir = base.join("toolchains/node/20.11.0");
            let current_dir = base.join("current");
            fs::create_dir_all(&current_dir).unwrap();

            // Update current/ symlink
            let current_link = current_dir.join("node");
            let temp_link = current_link.with_extension("tmp");
            let _ = fs::remove_file(&temp_link);
            unix_fs::symlink(&toolchain_dir, &temp_link).unwrap();
            fs::rename(&temp_link, &current_link).unwrap();

            // Update bin/ symlinks
            let bin_dir = base.join("bin");
            fs::create_dir_all(&bin_dir).unwrap();

            for bin_name in &["node", "npm", "npx"] {
                let bin_link = bin_dir.join(bin_name);
                let target = toolchain_dir.join("bin").join(bin_name);
                let _ = fs::remove_file(&bin_link);
                unix_fs::symlink(&target, &bin_link).unwrap();
            }

            black_box(())
        });
    });
}

/// Benchmark: Multiple symlink operations (simulating multiple tools)
fn bench_switch_multiple_tools(c: &mut Criterion) {
    let tmp = setup_toolchain_dir();
    let base = tmp.path();

    c.bench_function("switch_multiple_tools", |b| {
        b.iter(|| {
            // Switch node
            let node_dir = base.join("toolchains/node/20.11.0");
            let current_dir = base.join("current");
            fs::create_dir_all(&current_dir).unwrap();

            let current_link = current_dir.join("node");
            let temp_link = current_link.with_extension("tmp");
            let _ = fs::remove_file(&temp_link);
            unix_fs::symlink(&node_dir, &temp_link).unwrap();
            fs::rename(&temp_link, &current_link).unwrap();

            let bin_dir = base.join("bin");
            fs::create_dir_all(&bin_dir).unwrap();
            for bin_name in &["node", "npm", "npx"] {
                let bin_link = bin_dir.join(bin_name);
                let target = node_dir.join("bin").join(bin_name);
                let _ = fs::remove_file(&bin_link);
                unix_fs::symlink(&target, &bin_link).unwrap();
            }

            // Switch go
            let go_dir = base.join("toolchains/go/1.23.5");
            let current_link = current_dir.join("go");
            let temp_link = current_link.with_extension("tmp");
            let _ = fs::remove_file(&temp_link);
            unix_fs::symlink(&go_dir, &temp_link).unwrap();
            fs::rename(&temp_link, &current_link).unwrap();

            for bin_name in &["go", "gofmt"] {
                let bin_link = bin_dir.join(bin_name);
                let target = go_dir.join("bin").join(bin_name);
                let _ = fs::remove_file(&bin_link);
                unix_fs::symlink(&target, &bin_link).unwrap();
            }

            black_box(())
        });
    });
}

/// Benchmark: Cache write operations
fn bench_cache_write(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();

    let versions_data = serde_json::json!({
        "versions": [
            {"version": "20.11.0", "lts": "Iron"},
            {"version": "20.10.0", "lts": "Iron"},
            {"version": "22.0.0", "lts": null},
            {"version": "21.7.0", "lts": null},
        ],
        "cached_at": 1234567890u64
    });

    c.bench_function("cache_write", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&versions_data).unwrap();
            fs::write(cache_dir.join("remote-node.json"), &json).unwrap();
            black_box(())
        });
    });
}

/// Benchmark: Cache read operations
fn bench_cache_read(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();

    let versions_data = serde_json::json!({
        "versions": [
            {"version": "20.11.0", "lts": "Iron"},
            {"version": "20.10.0", "lts": "Iron"},
            {"version": "22.0.0", "lts": null},
            {"version": "21.7.0", "lts": null},
        ],
        "cached_at": 1234567890u64
    });

    let json = serde_json::to_string(&versions_data).unwrap();
    fs::write(cache_dir.join("remote-node.json"), &json).unwrap();

    c.bench_function("cache_read", |b| {
        b.iter(|| {
            let content = fs::read_to_string(cache_dir.join("remote-node.json")).unwrap();
            let data: serde_json::Value = serde_json::from_str(&content).unwrap();
            black_box(data)
        });
    });
}

/// Benchmark: Cache read + parse + validate
fn bench_cache_full_cycle(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();

    let versions_data = serde_json::json!({
        "versions": [
            {"version": "20.11.0", "lts": "Iron"},
            {"version": "20.10.0", "lts": "Iron"},
            {"version": "22.0.0", "lts": null},
            {"version": "21.7.0", "lts": null},
        ],
        "cached_at": 1234567890u64
    });

    let json = serde_json::to_string(&versions_data).unwrap();
    fs::write(cache_dir.join("remote-node.json"), &json).unwrap();

    c.bench_function("cache_full_cycle", |b| {
        b.iter(|| {
            // Read
            let content = fs::read_to_string(cache_dir.join("remote-node.json")).unwrap();
            let data: serde_json::Value = serde_json::from_str(&content).unwrap();

            // Validate TTL
            let cached_at = data["cached_at"].as_u64().unwrap();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let elapsed = now.saturating_sub(cached_at);
            let is_valid = elapsed <= 300;

            black_box((data, is_valid))
        });
    });
}

criterion_group!(
    benches,
    bench_parse_tool_versions,
    bench_resolve_version_deep_tree,
    bench_resolve_all_versions,
    bench_switch_symlinks,
    bench_switch_multiple_tools,
    bench_cache_write,
    bench_cache_read,
    bench_cache_full_cycle,
);

criterion_main!(benches);
