use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use crate::project::{self, ProjectNetworkConfig};
use crate::resolver;

pub(super) fn collect_project_config_check() -> DoctorCheck {
    let cwd = resolver::current_dir();
    match project::load_nearest_project_config(&cwd) {
        Ok(Some(loaded)) => {
            let mut details = vec![format!("Project config: {}", loaded.path.display())];
            details.push(format!("Command tasks: {}", loaded.config.commands.len()));
            details.push(format!("Project env vars: {}", loaded.config.env.len()));
            details.push(format!("Project mirrors: {}", loaded.config.mirrors.len()));
            details.push(format!(
                "Project network overrides: {}",
                count_project_network_overrides(&loaded.config.network)
            ));
            if let Some(auto_switch) = loaded.config.behavior.auto_switch {
                details.push(format!("auto_switch = {}", auto_switch));
            }
            if let Some(auto_activate_venv) = loaded.config.behavior.auto_activate_venv {
                details.push(format!("auto_activate_venv = {}", auto_activate_venv));
            }
            if let Some(non_interactive) = loaded.config.behavior.non_interactive {
                details.push(format!("non_interactive = {}", non_interactive));
            }

            DoctorCheck {
                id: "project_config".to_string(),
                status: CheckStatus::Ok,
                summary: "nearest .vex.toml is valid".to_string(),
                details,
            }
        }
        Ok(None) => DoctorCheck {
            id: "project_config".to_string(),
            status: CheckStatus::Ok,
            summary: "no .vex.toml found in the current project tree".to_string(),
            details: Vec::new(),
        },
        Err(err) => DoctorCheck {
            id: "project_config".to_string(),
            status: CheckStatus::Warn,
            summary: ".vex.toml could not be parsed".to_string(),
            details: vec![err.to_string()],
        },
    }
}

fn count_project_network_overrides(network: &ProjectNetworkConfig) -> usize {
    [
        network.connect_timeout_secs.is_some(),
        network.read_timeout_secs.is_some(),
        network.download_retries.is_some(),
        network.retry_base_delay_secs.is_some(),
        network.max_concurrent_downloads.is_some(),
        network.max_http_redirects.is_some(),
        network.proxy.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count()
}
