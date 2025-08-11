use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use colored::*;
use console::Emoji;

static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "✓ ");
static CROSS: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
static FOLDER: Emoji<'_, '_> = Emoji("📁 ", "> ");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ">> ");
static WRENCH: Emoji<'_, '_> = Emoji("🔧 ", "- ");

#[derive(Parser)]
#[command(
    name = "monk",
    about = "☯️ Monk - Simple Git hooks manager",
    long_about = "Monk is a simple and powerful Git hooks manager written in Rust.
It allows you to manage and automate Git hooks easily using a YAML configuration file.

Examples:
  monk install              Install all hooks from monk.yaml
  monk run pre-commit       Run pre-commit hook manually
  monk uninstall            Remove all monk hooks and restore backups

Configuration:
  Create a monk.yaml file in your project root with your hook definitions.
  
  Simple hook example:
    pre-commit:
      commands:
        - cargo fmt -- --check
        - cargo clippy
  
  Path-based hook example:
    pre-commit:
      paths:
        src/:
          commands:
            - cargo fmt -- --check
            - cargo clippy
        docs/:
          commands:
            - mdbook test

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
        hook_name: String 
    },
    #[command(about = "Uninstall all monk hooks and restore backups")]
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

pub fn find_all_path_configs<'a>(
    config: &'a Config,
    hook_name: &str,
) -> Vec<&'a Hook> {
    let mut all_hooks = Vec::new();

    if let Some(hook_config) = config.hooks.get(hook_name) {
        match hook_config {
            HookConfig::Simple(hook) => {
                all_hooks.push(hook);
            }
            HookConfig::PathBased { paths } => {
                for (_path_pattern, hook) in paths {
                    all_hooks.push(hook);
                }
            }
        }
    }

    all_hooks
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("monk.yaml")?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config)
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
            println!("{} Restored backup for {}", WRENCH, hook_name.yellow());
        } else if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
            println!("{} Removed existing hook {}", WRENCH, hook_name.yellow());
        }

        install_hook(hook_name);
        println!("{} Installed hook {}", CHECKMARK, hook_name.green().bold());
    }
    
    println!("{} All hooks installed successfully!", ROCKET);
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
            println!("{} Restored backup for {}", CHECKMARK, hook_name.green());
        } else if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
            println!("{} Removed hook {}", CHECKMARK, hook_name.green());
        } else {
            println!("{} No hook found for {}", WRENCH, hook_name.yellow());
        }
    }
    
    println!("{} All hooks uninstalled successfully!", CHECKMARK);
}

pub fn run_hook(config: &Config, hook_name: &str) {
    let changed_files = get_changed_files();
    
    let matching_hooks = if changed_files.is_empty() {
        find_all_path_configs(config, hook_name)
    } else {
        find_matching_path_configs(config, hook_name, &changed_files)
    };

    if matching_hooks.is_empty() {
        println!("{} No commands defined for hook '{}'", CROSS, hook_name.red().bold());
        std::process::exit(1);
    }

    println!("{} Running {} hook", ROCKET, hook_name.cyan().bold());

    for hook in matching_hooks {
        if let Some(ref working_dir) = hook.working_directory {
            println!("{} {}", FOLDER, working_dir.blue().bold());
        }
        
        for command_str in &hook.commands {
            let mut command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" });

            if cfg!(windows) {
                command.args(["/C", command_str]);
            } else {
                command.args(["-c", command_str]);
            }

            if let Some(ref working_dir) = hook.working_directory {
                command.current_dir(working_dir);
            }

            let status = command.status().expect("Failed to execute command");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
    
    println!("{} Hook {} completed successfully!", CHECKMARK, hook_name.green().bold());
}

pub fn init() {
    let config = read_config().expect("Failed to read monk.yaml configuration");
    install_hooks(&config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_path_configs_empty() {
        let yaml = r#"
other-hook:
  commands:
    - echo "test"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let hooks = find_all_path_configs(&config, "pre-commit");
        assert_eq!(hooks.len(), 0);
    }

    #[test]
    fn test_path_based_vs_simple_config() {
        let simple_yaml = r#"
pre-commit:
  commands:
    - cargo fmt
"#;
        let config: Config = serde_yaml::from_str(simple_yaml).unwrap();
        let hooks = find_all_path_configs(&config, "pre-commit");
        assert_eq!(hooks.len(), 1);

        let path_yaml = r#"
pre-commit:
  paths:
    "src/":
      commands:
        - cargo fmt
"#;
        let config: Config = serde_yaml::from_str(path_yaml).unwrap();
        let hooks = find_all_path_configs(&config, "pre-commit");
        assert_eq!(hooks.len(), 1);
    }

    #[test]
    fn test_multiple_commands_in_hook() {
        let yaml = r#"
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
    - cargo test
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let hooks = find_all_path_configs(&config, "pre-commit");
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].commands.len(), 3);
        assert_eq!(hooks[0].commands[0], "cargo fmt -- --check");
        assert_eq!(hooks[0].commands[1], "cargo clippy -- -D warnings");
        assert_eq!(hooks[0].commands[2], "cargo test");
    }
}
