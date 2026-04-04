//! vex - macOS binary version manager
//!
//! Manages official binary distributions of Node.js, Go, Java, Rust, and other languages.
//! Implements fast version switching via symlinks + PATH prepending.

mod activation;
mod advisories;
mod alias;
mod app;
mod archive_cache;
mod cache;
mod checksum;
mod cli;
mod commands;
mod config;
mod downloader;
mod error;
mod fs_utils;
mod home_state;
mod http;
mod installer;
mod lock;
mod lockfile;
mod logging;
mod output;
mod paths;
mod project;
mod requested_versions;
mod resolver;
mod shell;
mod spec;
mod switcher;
mod team_config;
mod templates;
mod tool_metadata;
mod tools;
mod ui;
mod updater;
mod version_files;
mod version_state;
mod versioning;

fn main() {
    logging::init();

    if let Err(e) = app::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
