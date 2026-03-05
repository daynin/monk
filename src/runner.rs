use colored::Colorize;

use crate::config::{Config, Hook, HookConfig};
use crate::git::get_changed_files;
use crate::{CHECKMARK, CROSS, FOLDER, ROCKET, WRENCH};

pub fn find_matching_path_configs<'a>(
    config: &'a Config,
    hook_name: &str,
    changed_files: &[String],
) -> Vec<&'a Hook> {
    let Some(hook_config) = config.hooks.get(hook_name) else {
        return Vec::new();
    };

    match hook_config {
        HookConfig::Simple(hook) => vec![hook],
        HookConfig::PathBased { paths } => paths
            .iter()
            .filter(|(path_pattern, _hook)| {
                changed_files
                    .iter()
                    .any(|file| file.starts_with(path_pattern.as_str()))
            })
            .map(|(_path_pattern, hook)| hook)
            .collect(),
    }
}

pub fn find_all_path_configs<'a>(config: &'a Config, hook_name: &str) -> Vec<&'a Hook> {
    let Some(hook_config) = config.hooks.get(hook_name) else {
        return Vec::new();
    };

    match hook_config {
        HookConfig::Simple(hook) => vec![hook],
        HookConfig::PathBased { paths } => paths.values().collect(),
    }
}

fn resolve_working_directory<'a>(
    hook: &'a Hook,
    command_working_dir: &'a Option<String>,
) -> Option<&'a String> {
    command_working_dir
        .as_ref()
        .or(hook.working_directory.as_ref())
}

pub fn run_hook(config: &Config, hook_name: &str, changed_only: bool) {
    let changed_files = get_changed_files();

    let matching_hooks = if changed_only && !changed_files.is_empty() {
        find_matching_path_configs(config, hook_name, &changed_files)
    } else {
        find_all_path_configs(config, hook_name)
    };

    if matching_hooks.is_empty() {
        println!(
            "{} No commands defined for hook '{}'",
            CROSS,
            hook_name.red().bold()
        );
        std::process::exit(1);
    }

    println!("{} Running {} hook", ROCKET, hook_name.cyan().bold());

    for hook in matching_hooks {
        if let Some(ref working_dir) = hook.working_directory {
            println!("{} {}", FOLDER, working_dir.blue().bold());
        }

        for (command_name, command) in &hook.commands {
            println!("{} {}", WRENCH, command_name.cyan().bold());

            let working_dir = resolve_working_directory(hook, &command.working_directory);

            let mut shell_command =
                std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" });

            if cfg!(windows) {
                shell_command.args(["/C", &command.run]);
            } else {
                shell_command.args(["-c", &command.run]);
            }

            if let Some(dir) = working_dir {
                shell_command.current_dir(dir);
            }

            let status = shell_command.status().expect("Failed to execute command");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }

    println!(
        "{} Hook {} completed successfully!",
        CHECKMARK,
        hook_name.green().bold()
    );
}
