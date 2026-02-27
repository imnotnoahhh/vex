use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::PathBuf;

mod downloader;
mod error;
mod installer;
mod resolver;
mod shell;
mod switcher;
mod tools;

use error::Result;

#[derive(Parser)]
#[command(name = "vex", version)]
#[command(about = "A fast version manager for macOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize vex directory structure
    Init,

    /// Install a tool version (or all from .tool-versions)
    Install {
        /// Tool and version (e.g., node@20, node@20.11.0). Omit to install from .tool-versions.
        spec: Option<String>,
    },

    /// Switch to a different version
    Use {
        /// Tool and version (e.g., node@20.11.0). Omit to auto-detect from version files.
        spec: Option<String>,

        /// Auto mode: read version files (.tool-versions, .node-version, etc.)
        #[arg(long)]
        auto: bool,
    },

    /// List installed versions
    List {
        /// Tool name (e.g., node)
        tool: String,
    },

    /// List available remote versions
    ListRemote {
        /// Tool name (e.g., node)
        tool: String,

        /// Show all versions (default: interactive top 20)
        #[arg(long)]
        all: bool,
    },

    /// Show current active versions
    Current,

    /// Uninstall a version
    Uninstall {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },

    /// Output shell hook for auto-switching
    Env {
        /// Shell type (zsh or bash)
        shell: String,
    },

    /// Pin a tool version in the current directory (.tool-versions)
    Local {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },

    /// Pin a tool version globally (~/.tool-versions)
    Global {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },
}

fn vex_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".vex")
}

fn init_vex() -> Result<()> {
    let vex_dir = vex_dir();

    // 创建目录结构
    fs::create_dir_all(vex_dir.join("cache"))?;
    fs::create_dir_all(vex_dir.join("locks"))?;
    fs::create_dir_all(vex_dir.join("toolchains"))?;
    fs::create_dir_all(vex_dir.join("current"))?;
    fs::create_dir_all(vex_dir.join("bin"))?;

    // 创建配置文件
    let config_path = vex_dir.join("config.toml");
    if !config_path.exists() {
        fs::write(&config_path, "# vex configuration\n")?;
    }

    println!("✓ Created directory structure at {}", vex_dir.display());
    println!();
    println!("Run this to activate vex (auto-switching on cd):");
    println!();
    println!("  echo 'eval \"$(vex env zsh)\"' >> ~/.zshrc && source ~/.zshrc");
    println!();

    Ok(())
}

fn parse_spec(spec: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = spec.split('@').collect();
    if parts.len() == 2 {
        // 格式：tool@version
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else if parts.len() == 1 {
        // 格式：tool（只有工具名，需要交互式选择版本）
        Ok((parts[0].to_string(), String::new()))
    } else {
        Err(error::VexError::Parse(format!(
            "Invalid spec format: {}. Expected format: tool@version or tool",
            spec
        )))
    }
}

fn interactive_install(tool_name: &str) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;

    println!("Fetching available versions of {}...", tool_name);
    let versions = tool.list_remote()?;

    println!();
    println!("Select a version to install:");
    println!();

    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            if let Some(lts) = &v.lts {
                format!("{} (LTS: {})", v.version, lts)
            } else {
                v.version.clone()
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .max_length(20)
        .interact_opt()
        .map_err(|e| error::VexError::Dialog(e.to_string()))?;

    if let Some(index) = selection {
        let selected_version = &versions[index].version;
        // 移除 v 前缀（如果有）
        let version = selected_version
            .strip_prefix('v')
            .unwrap_or(selected_version);

        println!();
        installer::install(tool.as_ref(), version)?;
        switcher::switch_version(tool.as_ref(), version)?;
    } else {
        println!("Installation cancelled.");
    }

    Ok(())
}

fn show_current() -> Result<()> {
    let vex_dir = dirs::home_dir().unwrap().join(".vex");
    let current_dir = vex_dir.join("current");

    if !current_dir.exists() {
        println!("No tools activated yet.");
        println!();
        println!("Use 'vex install <tool>' to install a tool.");
        return Ok(());
    }

    let entries = fs::read_dir(&current_dir)?;
    let mut tools: Vec<(String, String)> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let tool_name = entry.file_name().to_string_lossy().to_string();

        if let Ok(target) = fs::read_link(entry.path()) {
            if let Some(version) = target.file_name() {
                let version_str = version.to_string_lossy().to_string();
                tools.push((tool_name, version_str));
            }
        }
    }

    if tools.is_empty() {
        println!("No tools activated yet.");
        println!();
        println!("Use 'vex install <tool>' to install a tool.");
        return Ok(());
    }

    tools.sort_by(|a, b| a.0.cmp(&b.0));

    println!();
    println!("Current active versions:");
    println!();

    for (tool, version) in tools {
        println!("  {} → {}", tool, version);
    }

    println!();

    Ok(())
}

fn uninstall(tool_name: &str, version: &str) -> Result<()> {
    let vex_dir = dirs::home_dir().unwrap().join(".vex");
    let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);

    if !version_dir.exists() {
        return Err(error::VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: version.to_string(),
        });
    }

    println!("Uninstalling {} {}...", tool_name, version);

    // 检查是否是当前激活的版本
    let current_link = vex_dir.join("current").join(tool_name);
    if current_link.exists() {
        if let Ok(target) = fs::read_link(&current_link) {
            if target == version_dir {
                println!("Warning: This is the currently active version.");
                println!("The symlinks in ~/.vex/bin/ will be broken after uninstall.");
                println!(
                    "Use 'vex use {}@<version>' to switch to another version.",
                    tool_name
                );
            }
        }
    }

    // 删除版本目录
    fs::remove_dir_all(&version_dir)?;

    println!("✓ Uninstalled {} {}", tool_name, version);

    Ok(())
}

fn list_installed(tool_name: &str) -> Result<()> {
    let vex_dir = dirs::home_dir().unwrap().join(".vex");
    let toolchains_dir = vex_dir.join("toolchains").join(tool_name);

    if !toolchains_dir.exists() {
        println!("No versions of {} installed.", tool_name);
        return Ok(());
    }

    let entries = fs::read_dir(&toolchains_dir)?;
    let mut versions: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if versions.is_empty() {
        println!("No versions of {} installed.", tool_name);
        return Ok(());
    }

    versions.sort();

    // 检查当前激活的版本
    let current_link = vex_dir.join("current").join(tool_name);
    let current_version = if current_link.exists() {
        fs::read_link(&current_link)
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
    } else {
        None
    };

    println!();
    println!("Installed versions of {}:", tool_name);
    println!();

    for version in versions {
        if Some(&version) == current_version.as_ref() {
            println!("  {} (current)", version);
        } else {
            println!("  {}", version);
        }
    }

    println!();

    Ok(())
}

fn list_remote(tool_name: &str, show_all: bool) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;

    println!("Fetching available versions of {}...", tool_name);
    let versions = tool.list_remote()?;

    if show_all {
        println!();
        for version in versions.iter() {
            if let Some(lts) = &version.lts {
                println!("  {} (LTS: {})", version.version, lts);
            } else {
                println!("  {}", version.version);
            }
        }
        println!();
        println!("Total: {} versions", versions.len());
        return Ok(());
    }

    // 默认只展示最近 20 个，支持上下键滚动
    let recent: Vec<_> = versions.iter().take(20).cloned().collect();
    let items: Vec<String> = recent
        .iter()
        .map(|v| {
            if let Some(lts) = &v.lts {
                format!("{} (LTS: {})", v.version, lts)
            } else {
                v.version.clone()
            }
        })
        .collect();

    println!();
    println!("Use arrow keys to browse, Enter to install, Esc to quit");
    println!(
        "Showing latest {} of {} versions (use --all to show all)",
        items.len(),
        versions.len()
    );
    println!();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Available versions of {}", tool_name))
        .items(&items)
        .default(0)
        .max_length(20)
        .interact_opt()
        .map_err(|e| error::VexError::Dialog(e.to_string()))?;

    if let Some(index) = selection {
        let selected_version = &recent[index].version;
        let version = selected_version
            .strip_prefix('v')
            .unwrap_or(selected_version);
        let resolved = tools::resolve_fuzzy_version(tool.as_ref(), version)?;
        println!();
        println!("Installing {}@{}...", tool_name, resolved);
        installer::install(tool.as_ref(), &resolved)?;
        switcher::switch_version(tool.as_ref(), &resolved)?;
    }

    Ok(())
}

fn install_from_version_files() -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        println!("No version files found (.tool-versions, .node-version, etc.)");
        return Ok(());
    }

    let vex_dir = dirs::home_dir().unwrap().join(".vex");

    for (tool_name, version) in &versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(_) => {
                eprintln!("vex: skipping unsupported tool '{}'", tool_name);
                continue;
            }
        };

        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if version_dir.exists() {
            println!("{}@{} already installed, skipping.", tool_name, version);
            continue;
        }

        installer::install(tool.as_ref(), version)?;
        switcher::switch_version(tool.as_ref(), version)?;
    }

    Ok(())
}

fn write_tool_version(file_path: &std::path::Path, tool_name: &str, version: &str) -> Result<()> {
    let mut entries: Vec<(String, String)> = Vec::new();

    // Read existing file if present
    if file_path.is_file() {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let mut parts = trimmed.split_whitespace();
                if let (Some(t), Some(v)) = (parts.next(), parts.next()) {
                    if t != tool_name {
                        entries.push((t.to_string(), v.to_string()));
                    }
                }
            }
        }
    }

    entries.push((tool_name.to_string(), version.to_string()));
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let content: String = entries
        .iter()
        .map(|(t, v)| format!("{} {}", t, v))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    fs::write(file_path, content)?;
    Ok(())
}

fn auto_switch() -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        return Ok(());
    }

    let vex_dir = dirs::home_dir().unwrap().join(".vex");

    for (tool_name, version) in &versions {
        // 检查工具是否支持
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 检查版本是否已安装
        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if !version_dir.exists() {
            eprintln!(
                "vex: {}@{} not installed. Run 'vex install' to install.",
                tool_name, version
            );
            continue;
        }

        // 检查是否已经是当前版本（避免重复切换）
        let current_link = vex_dir.join("current").join(tool_name);
        if current_link.exists() {
            if let Ok(target) = fs::read_link(&current_link) {
                if let Some(current_ver) = target.file_name() {
                    if current_ver.to_string_lossy() == version.as_str() {
                        continue;
                    }
                }
            }
        }

        // 静默切换
        switcher::switch_version(tool.as_ref(), version)?;
    }

    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init_vex()?;
        }
        Commands::Install { spec } => {
            if let Some(spec) = spec {
                let (tool_name, version) = parse_spec(&spec)?;
                if version.is_empty() {
                    interactive_install(&tool_name)?;
                } else {
                    let tool = tools::get_tool(&tool_name)?;
                    let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
                    installer::install(tool.as_ref(), &resolved)?;
                    switcher::switch_version(tool.as_ref(), &resolved)?;
                }
            } else {
                install_from_version_files()?;
            }
        }
        Commands::Use { spec, auto } => {
            if auto {
                auto_switch()?;
            } else if let Some(spec) = spec {
                let (tool_name, version) = parse_spec(&spec)?;
                let tool = tools::get_tool(&tool_name)?;
                let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
                switcher::switch_version(tool.as_ref(), &resolved)?;
            } else {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0) or use --auto".to_string(),
                ));
            }
        }
        Commands::List { tool } => {
            list_installed(&tool)?;
        }
        Commands::ListRemote { tool, all } => {
            list_remote(&tool, all)?;
        }
        Commands::Current => {
            show_current()?;
        }
        Commands::Uninstall { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version to uninstall (e.g., node@20.11.0)".to_string(),
                ));
            }
            uninstall(&tool_name, &version)?;
        }
        Commands::Env { shell } => match shell::generate_hook(&shell) {
            Ok(hook) => print!("{}", hook),
            Err(e) => return Err(error::VexError::Parse(e)),
        },
        Commands::Local { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0)".to_string(),
                ));
            }
            let tool = tools::get_tool(&tool_name)?;
            let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
            let file_path = resolver::current_dir().join(".tool-versions");
            write_tool_version(&file_path, &tool_name, &resolved)?;
            println!("Set {}@{} in {}", tool_name, resolved, file_path.display());
        }
        Commands::Global { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0)".to_string(),
                ));
            }
            let tool = tools::get_tool(&tool_name)?;
            let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
            let file_path = dirs::home_dir().unwrap().join(".tool-versions");
            write_tool_version(&file_path, &tool_name, &resolved)?;
            println!("Set {}@{} in {}", tool_name, resolved, file_path.display());
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_with_version() {
        let (tool, version) = parse_spec("node@20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "20.11.0");
    }

    #[test]
    fn test_parse_spec_tool_only() {
        let (tool, version) = parse_spec("node").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_java() {
        let (tool, version) = parse_spec("java@21").unwrap();
        assert_eq!(tool, "java");
        assert_eq!(version, "21");
    }

    #[test]
    fn test_parse_spec_rust() {
        let (tool, version) = parse_spec("rust@1.93.1").unwrap();
        assert_eq!(tool, "rust");
        assert_eq!(version, "1.93.1");
    }

    #[test]
    fn test_parse_spec_go() {
        let (tool, version) = parse_spec("go@1.23.5").unwrap();
        assert_eq!(tool, "go");
        assert_eq!(version, "1.23.5");
    }

    #[test]
    fn test_parse_spec_invalid_multiple_at() {
        let result = parse_spec("node@20@11");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_spec_empty_version() {
        let (tool, version) = parse_spec("node@").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_version_with_v_prefix() {
        let (tool, version) = parse_spec("node@v20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "v20.11.0");
    }

    #[test]
    fn test_vex_dir() {
        let dir = vex_dir();
        assert!(dir.ends_with(".vex"));
    }
}
