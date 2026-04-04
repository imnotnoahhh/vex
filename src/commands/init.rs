mod setup;

use crate::error::Result;
use crate::resolver;
use crate::templates;

pub fn run(
    shell: Option<&str>,
    template: Option<&str>,
    list_templates: bool,
    dry_run: bool,
    add_only: bool,
) -> Result<()> {
    if list_templates {
        templates::print_templates();
        return Ok(());
    }

    if let Some(template_name) = template {
        let conflict_mode = if add_only {
            templates::ConflictMode::AddOnly
        } else {
            templates::ConflictMode::Strict
        };
        templates::init_template(
            &resolver::current_dir(),
            template_name,
            dry_run,
            conflict_mode,
        )?;
        return Ok(());
    }

    setup::init_vex(shell.unwrap_or("skip"), dry_run)
}
