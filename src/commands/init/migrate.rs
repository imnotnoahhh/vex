use owo_colors::OwoColorize;
use std::fs;

pub fn migrate_global_tool_versions() {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return,
    };
    let old_path = home.join(".tool-versions");
    let new_path = home.join(".vex").join("tool-versions");

    if !old_path.exists() || new_path.exists() {
        return;
    }

    if let Ok(content) = fs::read_to_string(&old_path) {
        let vex_dir = home.join(".vex");
        if fs::create_dir_all(&vex_dir).is_ok() && fs::write(&new_path, &content).is_ok() {
            let _ = fs::remove_file(&old_path);
            eprintln!(
                "{} Migrated ~/.tool-versions → ~/.vex/tool-versions",
                "vex:".cyan()
            );
        }
    }
}
