//! Environment variable management for tool-specific variables

#![allow(dead_code)]

use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[cfg(test)]
use std::path::PathBuf;

/// Environment configuration from .vex.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvConfig {
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Load environment configuration from .vex.toml
pub fn load_project_env() -> Result<HashMap<String, String>> {
    let path = Path::new(".vex.toml");
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| VexError::Config(format!("Failed to read .vex.toml: {}", e)))?;

    let config: EnvConfig = toml::from_str(&content)
        .map_err(|e| VexError::Config(format!("Failed to parse .vex.toml: {}", e)))?;

    Ok(config.env)
}

/// Get tool-specific environment variables for an active tool
pub fn get_tool_env_vars(tool_name: &str, install_path: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();

    match tool_name {
        "java" => {
            // JAVA_HOME should point to the JDK home directory
            let java_home = if cfg!(target_os = "macos") {
                install_path.join("Contents").join("Home")
            } else {
                install_path.to_path_buf()
            };

            env.insert("JAVA_HOME".to_string(), java_home.display().to_string());
        }
        "go" => {
            // GOROOT points to the Go installation directory
            env.insert("GOROOT".to_string(), install_path.display().to_string());
        }
        "rust" => {
            // RUSTUP_HOME and CARGO_HOME are already managed by vex
            // We set CARGO_HOME in the shell hooks
        }
        "node" | "python" => {
            // Node.js and Python work primarily through PATH
            // No additional environment variables needed
        }
        _ => {}
    }

    env
}

/// Get all active tool environment variables
pub fn get_active_tools_env(vex_dir: &Path) -> HashMap<String, String> {
    let mut all_env = HashMap::new();
    let current_dir = vex_dir.join("current");

    if !current_dir.exists() {
        return all_env;
    }

    if let Ok(entries) = fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            let tool_name = entry.file_name().to_string_lossy().to_string();

            // Read the symlink to get the actual installation path
            if let Ok(target) = fs::read_link(entry.path()) {
                let tool_env = get_tool_env_vars(&tool_name, &target);
                all_env.extend(tool_env);
            }
        }
    }

    all_env
}

/// Generate shell export statements for environment variables
pub fn generate_env_exports(env: &HashMap<String, String>, shell: &str) -> String {
    let mut exports = Vec::new();

    for (key, value) in env {
        let export = match shell {
            "fish" => format!("set -gx {} \"{}\"", key, value),
            "nu" | "nushell" => format!("$env.{} = \"{}\"", key, value),
            _ => format!("export {}=\"{}\"", key, value), // bash, zsh
        };
        exports.push(export);
    }

    exports.sort();
    exports.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_env_vars_java() {
        let path = PathBuf::from("/test/java/21");
        let env = get_tool_env_vars("java", &path);
        assert!(env.contains_key("JAVA_HOME"));

        if cfg!(target_os = "macos") {
            assert!(env.get("JAVA_HOME").unwrap().contains("Contents/Home"));
        }
    }

    #[test]
    fn test_get_tool_env_vars_go() {
        let path = PathBuf::from("/test/go/1.22");
        let env = get_tool_env_vars("go", &path);
        assert_eq!(env.get("GOROOT"), Some(&"/test/go/1.22".to_string()));
    }

    #[test]
    fn test_get_tool_env_vars_node() {
        let path = PathBuf::from("/test/node/20");
        let env = get_tool_env_vars("node", &path);
        assert!(env.is_empty()); // Node.js doesn't need special env vars
    }

    #[test]
    fn test_generate_env_exports_bash() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let exports = generate_env_exports(&env, "bash");
        assert!(exports.contains("export BAZ=\"qux\""));
        assert!(exports.contains("export FOO=\"bar\""));
    }

    #[test]
    fn test_generate_env_exports_fish() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let exports = generate_env_exports(&env, "fish");
        assert_eq!(exports, "set -gx FOO \"bar\"");
    }

    #[test]
    fn test_generate_env_exports_nushell() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let exports = generate_env_exports(&env, "nu");
        assert_eq!(exports, "$env.FOO = \"bar\"");
    }
}
