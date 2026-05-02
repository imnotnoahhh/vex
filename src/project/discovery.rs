use std::path::{Path, PathBuf};

const PROJECT_CONFIG_FILE: &str = ".vex.toml";
const PROJECT_VENV_DIR: &str = ".venv";
const NODE_MODULES_BIN_DIR: &str = "node_modules/.bin";

pub fn find_nearest_project_file(start_dir: &Path) -> Option<PathBuf> {
    find_in_ancestors(start_dir, PROJECT_CONFIG_FILE)
}

pub fn find_nearest_venv(start_dir: &Path) -> Option<PathBuf> {
    find_in_ancestors(start_dir, PROJECT_VENV_DIR)
}

pub fn find_nearest_node_modules_bin(start_dir: &Path) -> Option<PathBuf> {
    find_in_ancestors(start_dir, NODE_MODULES_BIN_DIR)
}

fn find_in_ancestors(start_dir: &Path, file_name: &str) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
