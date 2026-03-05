mod cli;
pub mod config;
mod git;
mod glob_filter;
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
pub use git::{get_all_tracked_files, get_changed_files, get_push_files, get_staged_files};
pub use glob_filter::filter_files_by_glob;
pub use hooks::{install_hook, install_hooks, uninstall_hooks};
pub use runner::{
    expand_file_placeholders, find_all_path_configs, find_matching_path_configs, run_hook,
};

pub fn init() {
    let config = read_config().expect("Failed to read monk.yaml configuration");
    install_hooks(&config);
}
