use crate::advisories;
use crate::commands::current::{collect_current, CurrentEntry};
use crate::commands::doctor::{self, DoctorReport};
use crate::commands::updates::{collect_outdated, OutdatedReport};
use crate::config;
use crate::error::{Result, VexError};
use crate::fs_utils::path_size;
use crate::requested_versions;
use crate::resolver;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum View {
    Dashboard,
    Outdated,
    Doctor,
}

impl View {
    pub(super) fn title(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Outdated => "Outdated",
            View::Doctor => "Doctor",
        }
    }
}

pub(super) struct AppState {
    pub(super) view: View,
    pub(super) dashboard: DashboardState,
    pub(super) outdated: Option<std::result::Result<OutdatedReport, String>>,
    pub(super) doctor: Option<std::result::Result<DoctorReport, String>>,
}

impl AppState {
    pub(super) fn new(dashboard: DashboardState) -> Self {
        Self {
            view: View::Dashboard,
            dashboard,
            outdated: None,
            doctor: None,
        }
    }

    pub(super) fn ensure_outdated(&mut self) {
        if self.outdated.is_none() {
            self.outdated = Some(collect_outdated(None).map_err(|e| e.to_string()));
        }
    }

    pub(super) fn ensure_doctor(&mut self) {
        if self.doctor.is_none() {
            self.doctor = Some(doctor::collect().map_err(|e| e.to_string()));
        }
    }
}

#[derive(Debug)]
pub(super) struct DashboardState {
    pub(super) current_tools: Vec<CurrentEntry>,
    pub(super) warnings: Vec<String>,
    pub(super) disk_usage: Option<DiskUsage>,
    pub(super) managed_versions: HashMap<String, String>,
    pub(super) missing_installs: Vec<String>,
}

#[derive(Debug)]
pub(super) struct DiskUsage {
    pub(super) vex_size_mb: u64,
    pub(super) available_mb: u64,
}

pub(super) fn collect_dashboard_state() -> Result<DashboardState> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let pwd = resolver::current_dir();

    let current_report = collect_current()?;
    let current_tools = current_report.tools;

    let versions = resolver::resolve_versions(&pwd);
    let global_path = vex_dir.join("tool-versions");
    let global_versions = resolver::read_tool_versions_file(&global_path);

    let mut managed_versions = HashMap::new();
    managed_versions.extend(global_versions);
    managed_versions.extend(versions);

    let mut missing_installs = Vec::new();
    for (tool, version) in &managed_versions {
        if requested_versions::resolve_installed_version(&vex_dir, tool, version)?.is_none() {
            missing_installs.push(format!("{}@{}", tool, version));
        }
    }

    let mut warnings = Vec::new();
    for entry in &current_tools {
        let advisory = advisories::get_advisory(&entry.tool, &entry.version);
        if advisory.is_warning() {
            if let Some(message) = advisory.message {
                warnings.push(format!("{}: {}", entry.tool, message));
            }
        }
    }

    let disk_usage = calculate_disk_usage(&vex_dir);

    Ok(DashboardState {
        current_tools,
        warnings,
        disk_usage,
        managed_versions,
        missing_installs,
    })
}

fn calculate_disk_usage(vex_dir: &Path) -> Option<DiskUsage> {
    let toolchains_dir = vex_dir.join("toolchains");
    let cache_dir = vex_dir.join("cache");

    let mut total_size = 0u64;
    if toolchains_dir.exists() {
        total_size += path_size(&toolchains_dir);
    }
    if cache_dir.exists() {
        total_size += path_size(&cache_dir);
    }

    let available_mb = if fs::metadata(vex_dir).is_ok() {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        disks
            .iter()
            .find(|disk| vex_dir.starts_with(disk.mount_point()))
            .map(|disk| disk.available_space() / 1024 / 1024)
            .unwrap_or(0)
    } else {
        0
    };

    Some(DiskUsage {
        vex_size_mb: total_size / 1024 / 1024,
        available_mb,
    })
}
