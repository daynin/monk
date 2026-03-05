mod cli;
pub mod config;
mod git;
mod hooks;
mod runner;

use console::Emoji;

pub(crate) static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "✓ ");
pub(crate) static CROSS: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
pub(crate) static FOLDER: Emoji<'_, '_> = Emoji("📁 ", "> ");
pub(crate) static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ">> ");
pub(crate) static WRENCH: Emoji<'_, '_> = Emoji("🔧 ", "- ");

pub use cli::{Cli, Commands};
pub use config::{read_config, Command, Config, Hook, HookConfig};
pub use git::get_changed_files;
pub use hooks::{install_hook, install_hooks, uninstall_hooks};
pub use runner::{find_all_path_configs, find_matching_path_configs, run_hook};

pub fn init() {
    let config = read_config().expect("Failed to read monk.yaml configuration");
    install_hooks(&config);
}
