use crate::cli::{Cli, Commands};
use crate::error;
use crate::error::Result;
use crate::{commands, output, shell, updater};
use clap::Parser;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Init(args) => commands::init::run(
            args.shell.as_deref(),
            args.template.as_deref(),
            args.list_templates,
            args.dry_run,
            args.add_only,
        )?,
        Commands::Install(args) => {
            if !args.specs.is_empty() {
                commands::toolchain::install_specs(
                    &args.specs,
                    args.no_switch,
                    args.force,
                    args.offline,
                )?;
            } else if let Some(source) = args.from {
                commands::toolchain::install_from_source(&source, args.offline)?;
            } else {
                commands::toolchain::install_from_version_files_with_frozen(
                    args.frozen,
                    args.offline,
                )?;
            }
        }
        Commands::Sync(args) => {
            if let Some(source) = args.from {
                commands::toolchain::sync_from_source(&source, args.offline)?;
            } else {
                commands::toolchain::sync_from_current_context_with_frozen(
                    args.frozen,
                    args.offline,
                )?;
            }
        }
        Commands::Use(args) => {
            if args.auto {
                commands::toolchain::auto_switch()?;
            } else if let Some(spec) = args.spec {
                commands::manage::use_spec(&spec)?;
            } else {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0) or use --auto".to_string(),
                ));
            }
        }
        Commands::Relink(args) => {
            commands::manage::relink_tool(&args.tool)?;
        }
        Commands::List(args) => {
            commands::versions::list_installed(
                &args.tool,
                output::OutputMode::from_json_flag(args.json),
                args.verbose,
            )?;
        }
        Commands::ListRemote(args) => {
            commands::versions::list_remote(
                &args.tool,
                args.filter,
                !args.no_cache,
                args.offline,
                output::OutputMode::from_json_flag(args.json),
            )?;
        }
        Commands::Current(args) => {
            commands::current::show(output::OutputMode::from_json_flag(args.json), args.verbose)?;
        }
        Commands::Uninstall(args) => {
            commands::manage::uninstall_spec(&args.spec)?;
        }
        Commands::Env(args) => {
            if args.exports {
                commands::process::print_exports(&args.shell)?;
            } else {
                match shell::generate_hook(&args.shell) {
                    Ok(hook) => print!("{}", hook),
                    Err(err) => return Err(error::VexError::Parse(err)),
                }
            }
        }
        Commands::Local(args) => {
            commands::manage::set_project_version(&args.spec)?;
        }
        Commands::Global(args) => {
            commands::manage::set_global_version(&args.spec)?;
        }
        Commands::Lock => {
            commands::toolchain::generate_lockfile()?;
        }
        Commands::Upgrade(args) => {
            commands::updates::upgrade(args.tool.as_deref(), args.all)?;
        }
        Commands::Outdated(args) => {
            commands::updates::outdated(
                args.tool.as_deref(),
                output::OutputMode::from_json_flag(args.json),
            )?;
        }
        Commands::Prune(args) => {
            commands::prune::run(args.dry_run)?;
        }
        Commands::Alias(subcmd) => {
            commands::aliases::run(&subcmd)?;
        }
        Commands::Exec(args) => exit_on_failure(commands::process::exec_command(&args.command)?),
        Commands::Run(args) => {
            exit_on_failure(commands::process::run_task(&args.task, &args.args)?)
        }
        Commands::Doctor(args) => {
            commands::doctor::run(output::OutputMode::from_json_flag(args.json), args.verbose)?;
        }
        Commands::Repair(args) => commands::repair::run(&args)?,
        Commands::SelfUpdate => {
            updater::self_update()?;
        }
        Commands::Tui => {
            commands::tui::run()?;
        }
        Commands::Python(args) => commands::python::run_subcommand(&args.subcmd)?,
        Commands::Rust(args) => commands::rust::run(&args)?,
    }

    Ok(())
}

fn exit_on_failure(code: i32) {
    if code != 0 {
        std::process::exit(code);
    }
}
