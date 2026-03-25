use clap::Args;

#[derive(Args)]
pub(crate) struct InitArgs {
    /// Shell to configure (auto, zsh, bash, fish, or skip)
    #[arg(long, conflicts_with_all = ["template", "list_templates", "add_only"])]
    pub(crate) shell: Option<String>,

    /// Initialize the current directory with an official project template
    #[arg(long, conflicts_with_all = ["shell", "list_templates"])]
    pub(crate) template: Option<String>,

    /// List the built-in project templates
    #[arg(long = "list-templates", conflicts_with_all = ["shell", "template", "add_only"])]
    pub(crate) list_templates: bool,

    /// Preview changes without modifying files
    #[arg(long, conflicts_with = "list_templates")]
    pub(crate) dry_run: bool,

    /// Only add missing safe files and merge `.tool-versions` / `.gitignore`
    #[arg(long, requires = "template", conflicts_with_all = ["shell", "list_templates"])]
    pub(crate) add_only: bool,
}
