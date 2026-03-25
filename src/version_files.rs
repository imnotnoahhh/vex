use crate::error::Result;
use std::fs;
use std::path::Path;

enum VersionFileFormat {
    ToolVersions,
    SingleValue,
}

pub fn write_tool_version(file_path: &Path, tool_name: &str, version: &str) -> Result<()> {
    let content = match version_file_format(file_path) {
        VersionFileFormat::ToolVersions => {
            write_tool_versions_content(file_path, tool_name, version)
        }
        VersionFileFormat::SingleValue => format!("{}\n", version),
    };

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

fn write_tool_versions_content(file_path: &Path, tool_name: &str, version: &str) -> String {
    let mut lines = fs::read_to_string(file_path)
        .ok()
        .map(|existing| existing.lines().map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut updated = false;
    let mut rewritten = Vec::with_capacity(lines.len().max(1));

    for line in lines.drain(..) {
        if let Some(parts) = parse_tool_line(&line) {
            if parts.tool == tool_name {
                if !updated {
                    rewritten.push(format!(
                        "{}{} {}{}",
                        parts.leading, tool_name, version, parts.trailing
                    ));
                    updated = true;
                }
                continue;
            }
        }
        rewritten.push(line);
    }

    if !updated {
        let insert_at = rewritten
            .iter()
            .rposition(|line| !line.trim().is_empty())
            .map(|idx| idx + 1)
            .unwrap_or(0);
        rewritten.insert(insert_at, format!("{} {}", tool_name, version));
    }

    if rewritten.is_empty() {
        String::new()
    } else {
        rewritten.join("\n") + "\n"
    }
}

fn version_file_format(file_path: &Path) -> VersionFileFormat {
    match file_path.file_name().and_then(|name| name.to_str()) {
        Some(".tool-versions" | "tool-versions") => VersionFileFormat::ToolVersions,
        _ => VersionFileFormat::SingleValue,
    }
}

struct ToolLineParts<'a> {
    leading: &'a str,
    tool: &'a str,
    trailing: &'a str,
}

fn parse_tool_line(line: &str) -> Option<ToolLineParts<'_>> {
    let leading_len = line.len() - line.trim_start_matches(char::is_whitespace).len();
    let leading = &line[..leading_len];
    let rest = &line[leading_len..];
    if rest.is_empty() || rest.starts_with('#') {
        return None;
    }

    let tool_end = rest.find(char::is_whitespace)?;
    let tool = &rest[..tool_end];
    let after_tool = &rest[tool_end..];
    let value_start = after_tool.find(|c: char| !c.is_whitespace())?;
    let value_and_trailing = &after_tool[value_start..];
    let value_end = value_and_trailing
        .find(char::is_whitespace)
        .unwrap_or(value_and_trailing.len());
    let trailing = &value_and_trailing[value_end..];

    Some(ToolLineParts {
        leading,
        tool,
        trailing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_tool_version_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        write_tool_version(&file_path, "node", "20.11.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "node 20.11.0\n");
    }

    #[test]
    fn test_write_tool_version_update_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "node 20.11.0\ngo 1.23.5\n").unwrap();
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("node 22.0.0"));
        assert!(content.contains("go 1.23.5"));
        assert!(!content.contains("20.11.0"));
    }

    #[test]
    fn test_write_tool_version_add_new_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "node 20.11.0\n").unwrap();
        write_tool_version(&file_path, "go", "1.23.5").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("node 20.11.0"));
        assert!(content.contains("go 1.23.5"));
    }

    #[test]
    fn test_write_tool_version_preserves_existing_order() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "rust 1.93.1\ngo 1.23.5\n").unwrap();
        write_tool_version(&file_path, "node", "20.11.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "rust 1.93.1");
        assert_eq!(lines[1], "go 1.23.5");
        assert_eq!(lines[2], "node 20.11.0");
    }

    #[test]
    fn test_write_tool_version_preserves_comments() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "# Comment\nnode 20.11.0\n# Another comment\n").unwrap();
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "# Comment\nnode 22.0.0\n# Another comment\n");
    }

    #[test]
    fn test_write_tool_version_preserves_empty_lines() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "\n\nnode 20.11.0\n\n# footer\n").unwrap();
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "\n\nnode 22.0.0\n\n# footer\n");
    }

    #[test]
    fn test_write_tool_version_preserves_inline_comment_on_updated_line() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        fs::write(&file_path, "node 20.11.0 # lts\n").unwrap();
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "node 22.0.0 # lts\n");
    }

    #[test]
    fn test_write_tool_version_preserves_single_value_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".python-version");

        write_tool_version(&file_path, "python", "3.14.3").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "3.14.3\n");
    }
}
