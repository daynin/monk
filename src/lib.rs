use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "monk")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Install,
    Run { hook_name: String },
    Uninstall,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub hooks: HashMap<String, HookConfig>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum HookConfig {
    Simple(Hook),
    PathBased { paths: HashMap<String, Hook> },
}

#[derive(Deserialize)]
pub struct Hook {
    pub commands: Vec<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

pub fn get_changed_files() -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .unwrap_or_else(|_| {
            std::process::Command::new("git")
                .args(["diff", "--name-only", "HEAD~1"])
                .output()
                .expect("Failed to get changed files")
        });

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn find_matching_path_configs<'a>(
    config: &'a Config,
    hook_name: &str,
    changed_files: &[String],
) -> Vec<&'a Hook> {
    let mut matching_hooks = Vec::new();

    if let Some(hook_config) = config.hooks.get(hook_name) {
        match hook_config {
            HookConfig::Simple(hook) => {
                matching_hooks.push(hook);
            }
            HookConfig::PathBased { paths } => {
                for (path_pattern, hook) in paths {
                    if changed_files
                        .iter()
                        .any(|file| file.starts_with(path_pattern))
                    {
                        matching_hooks.push(hook);
                    }
                }
            }
        }
    }

    matching_hooks
}

pub fn read_config() -> Config {
    let config_str = fs::read_to_string("monk.yaml").expect("Failed to read monk.yaml");
    serde_yaml::from_str(&config_str).expect("Failed to parse monk.yaml")
}

pub fn install_hooks(config: &Config) {
    let git_hooks_dir = ".git/hooks";
    if !Path::new(git_hooks_dir).exists() {
        fs::create_dir_all(git_hooks_dir).expect("Failed to create .git/hooks directory");
    }

    for hook_name in config.hooks.keys() {
        let hook_path = format!("{git_hooks_dir}/{hook_name}");
        let backup_path = format!("{hook_path}.backup");

        if Path::new(&backup_path).exists() {
            fs::rename(&backup_path, &hook_path)
                .unwrap_or_else(|_| panic!("Failed to restore backup for {hook_name}"));
            println!("Restored backup for {hook_name}");
        } else if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
            println!("Removed hook {hook_name}");
        } else {
            println!("No hook or backup found for {hook_name}");
        }

        install_hook(hook_name);
    }
}

pub fn install_hook(hook_name: &str) {
    let git_hooks_dir = ".git/hooks";
    if !Path::new(git_hooks_dir).exists() {
        fs::create_dir_all(git_hooks_dir).expect("Failed to create .git/hooks directory");
    }

    let hook_path = format!("{git_hooks_dir}/{hook_name}");

    let hook_content = format!(
        "#!/bin/sh\n\
        if monk -h >/dev/null 2>&1\n\
        then\n\
            exec monk run {hook_name}\n\
        else\n\
            cargo install monk\n\
            exec monk run {hook_name}\n\
        fi"
    );

    fs::write(&hook_path, hook_content)
        .unwrap_or_else(|_| panic!("Failed to write hook script to {hook_path}"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .expect("Failed to get file permissions")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).expect("Failed to set file permissions");
    }
}

pub fn uninstall_hooks(config: &Config) {
    let git_hooks_dir = ".git/hooks";
    for hook_name in config.hooks.keys() {
        let hook_path = format!("{git_hooks_dir}/{hook_name}");
        let backup_path = format!("{hook_path}.backup");

        if Path::new(&backup_path).exists() {
            fs::rename(&backup_path, &hook_path)
                .unwrap_or_else(|_| panic!("Failed to restore backup for {hook_name}"));
            println!("Restored backup for {hook_name}");
        } else if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
            println!("Removed hook {hook_name}");
        } else {
            println!("No hook or backup found for {hook_name}");
        }
    }
}

pub fn run_hook(config: &Config, hook_name: &str) {
    let changed_files = get_changed_files();
    let matching_hooks = find_matching_path_configs(config, hook_name, &changed_files);

    if matching_hooks.is_empty() {
        eprintln!("No commands defined for hook '{hook_name}'");
        std::process::exit(1);
    }

    for hook in matching_hooks {
        for command_str in &hook.commands {
            println!("Running command: {command_str}");

            let mut command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" });

            if cfg!(windows) {
                command.args(["/C", command_str]);
            } else {
                command.args(["-c", command_str]);
            }

            if let Some(ref working_dir) = hook.working_directory {
                command.current_dir(working_dir);
                println!("  in directory: {working_dir}");
            }

            let status = command.status().expect("Failed to execute command");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
}

pub fn init() {
    let config = read_config();
    install_hooks(&config);
}
