use crate::error::{Result, VexError};
use crate::resolver;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(super) fn merge_tool_versions_file(
    path: &Path,
    template_contents: &str,
) -> Result<Option<String>> {
    let existing = fs::read_to_string(path)?;
    let existing_versions: BTreeMap<_, _> = resolver::parse_tool_versions(&existing)
        .into_iter()
        .collect();
    let template_versions = resolver::parse_tool_versions(template_contents);

    let mut additions = Vec::new();
    for (tool, version) in template_versions {
        match existing_versions.get(&tool) {
            Some(existing_version) if existing_version != &version => {
                return Err(VexError::Config(format!(
                    "Template could not be applied because {} already pins {}@{} and the template wants {}@{}.\n\nNo files were written.",
                    path.display(),
                    tool,
                    existing_version,
                    tool,
                    version
                )));
            }
            Some(_) => {}
            None => additions.push(format!("{} {}", tool, version)),
        }
    }

    if additions.is_empty() {
        return Ok(None);
    }

    let mut merged = existing;
    if !merged.ends_with('\n') && !merged.is_empty() {
        merged.push('\n');
    }
    for addition in additions {
        merged.push_str(&addition);
        merged.push('\n');
    }
    Ok(Some(merged))
}

pub(super) fn merge_gitignore_file(path: &Path, template_contents: &str) -> Result<Option<String>> {
    let existing = fs::read_to_string(path)?;
    let existing_lines: BTreeSet<_> = existing.lines().map(str::trim).collect();
    let mut additions = Vec::new();

    for line in template_contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if !existing_lines.contains(line) {
            additions.push(line.to_string());
        }
    }

    if additions.is_empty() {
        return Ok(None);
    }

    let mut merged = existing;
    if !merged.ends_with('\n') && !merged.is_empty() {
        merged.push('\n');
    }
    for addition in additions {
        merged.push_str(&addition);
        merged.push('\n');
    }
    Ok(Some(merged))
}
