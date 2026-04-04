use crate::activation::{self, ActivationPlan};
use crate::config;
use crate::error::{Result, VexError};
use crate::project;
use crate::resolver;
use std::process::Command;

pub fn exec_command(command: &[String]) -> Result<i32> {
    if command.is_empty() {
        return Err(VexError::Parse(
            "Please provide a command after '--' (for example: 'vex exec -- node -v')".to_string(),
        ));
    }

    let cwd = resolver::current_dir();
    let plan = activation::build_activation_plan(&cwd)?;
    spawn_direct_command(&plan, &cwd, command)
}

pub fn run_task(task: &str, args: &[String]) -> Result<i32> {
    let cwd = resolver::current_dir();
    let plan = activation::build_activation_plan(&cwd)?;
    let project = plan.project.as_ref().ok_or_else(|| {
        VexError::Config(
            "No .vex.toml found in the current project tree. Create one before using 'vex run'."
                .to_string(),
        )
    })?;

    let command =
        project.config.commands.get(task).ok_or_else(|| {
            VexError::Config(format!("Task '{}' was not found in .vex.toml", task))
        })?;

    let shell = resolve_shell(project::load_nearest_project_config(&cwd)?.as_ref())?;
    // Use a non-login shell so rc/profile files cannot overwrite the activation
    // environment we inject for the task process.
    let shell_flag = "-c";

    let mut full_command = command.clone();
    if !args.is_empty() {
        full_command.push(' ');
        full_command.push_str(
            &args
                .iter()
                .map(|arg| shell_quote(arg))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    let mut process = Command::new(shell);
    process.arg(shell_flag).arg(full_command);
    process.current_dir(&project.root);
    apply_activation_environment(&mut process, &plan);

    let status = process.status()?;
    Ok(status.code().unwrap_or(1))
}

pub fn print_exports(shell: &str) -> Result<()> {
    let cwd = resolver::current_dir();
    let plan = activation::build_activation_plan(&cwd)?;
    let exports = crate::shell::generate_exports(shell, &plan).map_err(VexError::Parse)?;
    print!("{}", exports);
    Ok(())
}

fn spawn_direct_command(
    plan: &ActivationPlan,
    cwd: &std::path::Path,
    command: &[String],
) -> Result<i32> {
    let mut process = Command::new(&command[0]);
    process.args(&command[1..]);
    process.current_dir(cwd);
    apply_activation_environment(&mut process, plan);
    let status = process.status()?;
    Ok(status.code().unwrap_or(1))
}

fn apply_activation_environment(process: &mut Command, plan: &ActivationPlan) {
    for key in &plan.unset_env {
        process.env_remove(key);
    }

    for (key, value) in &plan.set_env {
        process.env(key, value);
    }

    process.env("PATH", activation::exec_path(plan));
}

fn resolve_shell(project: Option<&project::LoadedProjectConfig>) -> Result<String> {
    let global_default_shell = config::default_shell()?;
    Ok(project
        .and_then(|loaded| loaded.config.behavior.default_shell.clone())
        .or(global_default_shell)
        .or_else(|| std::env::var("SHELL").ok())
        .unwrap_or_else(|| "/bin/zsh".to_string()))
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':' | '='))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}
