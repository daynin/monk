use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::config::Config;
use crate::{CHECKMARK, ROCKET, WRENCH};

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
            exec monk run {hook_name} --changed-only\n\
        else\n\
            cargo install monk\n\
            exec monk run {hook_name} --changed-only\n\
        fi"
    );

    fs::write(&hook_path, hook_content)
        .unwrap_or_else(|_| panic!("Failed to write hook script to {hook_path}"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook_path)
            .expect("Failed to get file permissions")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).expect("Failed to set file permissions");
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
