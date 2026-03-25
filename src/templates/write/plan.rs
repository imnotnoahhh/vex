use super::merge::{merge_gitignore_file, merge_tool_versions_file};
use crate::error::{Result, VexError};
use crate::templates::{ConflictMode, MergeStrategy, PlannedWrite, PlannedWriteKind, TemplateFile};
use std::path::Path;

pub(in crate::templates) fn build_write_plan(
    cwd: &Path,
    files: &[TemplateFile],
    conflict_mode: ConflictMode,
) -> Result<Vec<PlannedWrite>> {
    let mut writes = Vec::new();
    let mut conflicts = Vec::new();

    for file in files {
        let path = cwd.join(file.path);
        if !path.exists() {
            writes.push(PlannedWrite {
                path,
                contents: file.contents.clone(),
                kind: PlannedWriteKind::Create,
            });
            continue;
        }

        match conflict_mode {
            ConflictMode::Strict => conflicts.push(path),
            ConflictMode::AddOnly => {
                let Some(strategy) = file.merge_strategy else {
                    conflicts.push(path);
                    continue;
                };
                let merged = match strategy {
                    MergeStrategy::ToolVersions => merge_tool_versions_file(&path, &file.contents)?,
                    MergeStrategy::GitIgnore => merge_gitignore_file(&path, &file.contents)?,
                };
                if let Some(contents) = merged {
                    writes.push(PlannedWrite {
                        path,
                        contents,
                        kind: PlannedWriteKind::Merge,
                    });
                }
            }
        }
    }

    if conflicts.is_empty() {
        return Ok(writes);
    }

    let mut message =
        String::from("Template could not be applied because these files already exist:\n");
    for conflict in &conflicts {
        message.push_str(&format!("  - {}\n", conflict.display()));
    }
    message.push_str("\nNo files were written.");
    Err(VexError::Config(message))
}
