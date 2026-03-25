mod home;
mod integration;
mod messaging;

use crate::error::Result;
use crate::paths::vex_dir;
use home::initialize_vex_home;
use integration::{configure_shell_integration, resolve_shell};
use messaging::{
    print_home_init_message, print_manual_shell_instructions, print_skip_instructions,
};

pub(super) fn init_vex(shell_arg: &str, dry_run: bool) -> Result<()> {
    let vex_dir = vex_dir()?;
    initialize_vex_home(&vex_dir, dry_run)?;
    print_home_init_message(&vex_dir, dry_run);

    match resolve_shell(shell_arg)? {
        Some(shell_name) => configure_shell_integration(&shell_name, dry_run),
        None if shell_arg == "skip" => {
            print_skip_instructions();
            Ok(())
        }
        None => {
            print_manual_shell_instructions();
            Ok(())
        }
    }
}
