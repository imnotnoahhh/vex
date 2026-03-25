use crate::alias;
use crate::cli::AliasCommands;
use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::tools;
use owo_colors::OwoColorize;

pub fn run(subcmd: &AliasCommands) -> Result<()> {
    let vex = vex_dir()?;
    let alias_manager = alias::AliasManager::new(&vex);

    match subcmd {
        AliasCommands::Set {
            tool,
            alias: alias_name,
            version,
            project,
        } => {
            let _ = tools::get_tool(tool)?;

            if *project {
                alias_manager.set_project(tool, alias_name, version)?;
                println!(
                    "{}",
                    format!("Set project alias: {}@{} -> {}", tool, alias_name, version).green()
                );
                println!("  (saved to .vex.toml in current directory)");
            } else {
                alias_manager.set_global(tool, alias_name, version)?;
                println!(
                    "{}",
                    format!("Set global alias: {}@{} -> {}", tool, alias_name, version).green()
                );
                println!("  (saved to ~/.vex/aliases.toml)");
            }
        }
        AliasCommands::List {
            tool,
            project,
            global,
        } => {
            let show_project = *project || !*global;
            let show_global = *global || !*project;

            if show_project {
                let project_aliases = alias_manager.list_project(tool.as_deref())?;
                if !project_aliases.is_empty() {
                    println!("{}", "Project aliases (.vex.toml):".cyan().bold());
                    for (tool_name, aliases) in project_aliases {
                        println!("\n  {}:", tool_name.yellow());
                        for (alias_name, version) in aliases {
                            println!("    {:<16} -> {}", alias_name, version);
                        }
                    }
                    println!();
                } else if *project {
                    println!("No project aliases found");
                }
            }

            if show_global {
                let global_aliases = alias_manager.list_global(tool.as_deref())?;
                if !global_aliases.is_empty() {
                    if show_project {
                        println!();
                    }
                    println!("{}", "Global aliases (~/.vex/aliases.toml):".cyan().bold());
                    for (tool_name, aliases) in global_aliases {
                        println!("\n  {}:", tool_name.yellow());
                        for (alias_name, version) in aliases {
                            println!("    {:<16} -> {}", alias_name, version);
                        }
                    }
                    println!();
                } else if *global {
                    println!("No global aliases found");
                }
            }

            if show_project && show_global {
                let project_aliases = alias_manager.list_project(tool.as_deref())?;
                let global_aliases = alias_manager.list_global(tool.as_deref())?;
                if project_aliases.is_empty() && global_aliases.is_empty() {
                    if let Some(tool_name) = tool {
                        println!("No aliases found for {}", tool_name);
                    } else {
                        println!("No aliases found");
                    }
                    println!("\nCreate an alias with: vex alias set <tool> <alias> <version>");
                }
            }
        }
        AliasCommands::Delete {
            tool,
            alias: alias_name,
            project,
        } => {
            let removed = if *project {
                alias_manager.delete_project(tool, alias_name)?
            } else {
                alias_manager.delete_global(tool, alias_name)?
            };

            if removed {
                let scope = if *project { "project" } else { "global" };
                println!(
                    "{}",
                    format!("Deleted {} alias: {}@{}", scope, tool, alias_name).green()
                );
            } else {
                let scope = if *project { "project" } else { "global" };
                return Err(VexError::Config(format!(
                    "Alias '{}' not found for {} in {} aliases",
                    alias_name, tool, scope
                )));
            }
        }
    }

    Ok(())
}
