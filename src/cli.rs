use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "monk",
    about = "Monk - Simple Git hooks manager",
    long_about = "Monk is a simple and powerful Git hooks manager written in Rust.
It allows you to manage and automate Git hooks easily using a YAML configuration file.

Examples:
  monk install              Install all hooks from monk.yaml
  monk run pre-commit       Run pre-commit hook manually
  monk uninstall            Remove all monk hooks and restore backups

Configuration:
  Create a monk.yaml file in your project root with your hook definitions.

  Named commands (recommended):
    pre-commit:
      commands:
        fmt:
          run: cargo fmt -- --check
        clippy:
          run: cargo clippy

  Simple commands:
    pre-commit:
      commands:
        - cargo fmt -- --check
        - cargo clippy

  Path-based hooks:
    pre-commit:
      paths:
        src/:
          commands:
            fmt:
              run: cargo fmt -- --check
        docs/:
          commands:
            test:
              run: mdbook test

For more examples and documentation, visit: https://github.com/daynin/monk",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Install all Git hooks from monk.yaml")]
    Install,
    #[command(about = "Run a specific hook manually")]
    Run {
        #[arg(help = "Name of the hook to run (e.g., pre-commit, pre-push)")]
        hook_name: String,
        #[arg(
            long,
            help = "Only run hooks for changed files (default: run all hooks)"
        )]
        changed_only: bool,
    },
    #[command(about = "Uninstall all monk hooks and restore backups")]
    Uninstall,
}
