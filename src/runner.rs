use colored::Colorize;

use crate::config::{Config, Hook, HookConfig};
use crate::git::{get_all_tracked_files, get_changed_files, get_push_files, get_staged_files};
use crate::glob_filter::filter_files_by_glob;
use crate::{CHECKMARK, CROSS, FOLDER, ROCKET, WRENCH};

const STAGED_FILES_PLACEHOLDER: &str = "{staged_files}";
const PUSH_FILES_PLACEHOLDER: &str = "{push_files}";
const ALL_FILES_PLACEHOLDER: &str = "{all_files}";

const MAX_COMMAND_LENGTH: usize = 131_072;

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

fn has_file_placeholder(template: &str) -> bool {
    template.contains(STAGED_FILES_PLACEHOLDER)
        || template.contains(PUSH_FILES_PLACEHOLDER)
        || template.contains(ALL_FILES_PLACEHOLDER)
}

fn has_glob_patterns(include_patterns: &[String], exclude_patterns: &[String]) -> bool {
    !include_patterns.is_empty() || !exclude_patterns.is_empty()
}

struct PlaceholderExpansion {
    placeholder: &'static str,
    files: Vec<String>,
}

fn collect_placeholder_expansions(
    template: &str,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Vec<PlaceholderExpansion> {
    let mut expansions = Vec::new();

    if template.contains(STAGED_FILES_PLACEHOLDER) {
        let files = filter_files_by_glob(get_staged_files(), include_patterns, exclude_patterns);
        expansions.push(PlaceholderExpansion {
            placeholder: STAGED_FILES_PLACEHOLDER,
            files,
        });
    }

    if template.contains(PUSH_FILES_PLACEHOLDER) {
        let files = filter_files_by_glob(get_push_files(), include_patterns, exclude_patterns);
        expansions.push(PlaceholderExpansion {
            placeholder: PUSH_FILES_PLACEHOLDER,
            files,
        });
    }

    if template.contains(ALL_FILES_PLACEHOLDER) {
        let files =
            filter_files_by_glob(get_all_tracked_files(), include_patterns, exclude_patterns);
        expansions.push(PlaceholderExpansion {
            placeholder: ALL_FILES_PLACEHOLDER,
            files,
        });
    }

    expansions
}

fn expand_simple(template: &str, expansions: &[PlaceholderExpansion]) -> String {
    let mut result = template.to_string();
    for expansion in expansions {
        let joined_files = expansion.files.join(" ");
        result = result.replace(expansion.placeholder, &joined_files);
    }
    result
}

pub fn expand_file_placeholders(
    template: &str,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Option<Vec<String>> {
    if !has_file_placeholder(template) {
        if !has_glob_patterns(include_patterns, exclude_patterns) {
            return Some(vec![template.to_string()]);
        }
        let matching_staged =
            filter_files_by_glob(get_staged_files(), include_patterns, exclude_patterns);
        if matching_staged.is_empty() {
            return None;
        }
        return Some(vec![template.to_string()]);
    }

    let expansions = collect_placeholder_expansions(template, include_patterns, exclude_patterns);

    let any_placeholder_empty = expansions.iter().any(|exp| exp.files.is_empty());
    if any_placeholder_empty {
        return None;
    }

    let simple_expansion = expand_simple(template, &expansions);

    if simple_expansion.len() <= MAX_COMMAND_LENGTH {
        return Some(vec![simple_expansion]);
    }

    let largest_expansion = expansions
        .iter()
        .max_by_key(|exp| exp.files.len())
        .expect("At least one expansion exists");

    Some(split_into_batches(
        &largest_expansion.files,
        template,
        largest_expansion.placeholder,
        &expansions,
    ))
}

fn split_into_batches(
    files: &[String],
    template: &str,
    batch_placeholder: &str,
    all_expansions: &[PlaceholderExpansion],
) -> Vec<String> {
    let base_length = template.len() - batch_placeholder.len();
    let non_batch_overhead: usize = all_expansions
        .iter()
        .filter(|exp| exp.placeholder != batch_placeholder)
        .map(|exp| {
            let joined = exp.files.join(" ");
            joined.len().saturating_sub(exp.placeholder.len())
        })
        .sum();

    let available_length = MAX_COMMAND_LENGTH
        .saturating_sub(base_length)
        .saturating_sub(non_batch_overhead);

    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_length: usize = 0;

    for file in files {
        let file_length = if current_batch.is_empty() {
            file.len()
        } else {
            file.len() + 1
        };

        if current_length + file_length > available_length && !current_batch.is_empty() {
            let batch_files = current_batch.join(" ");
            let mut command = template.replace(batch_placeholder, &batch_files);
            for expansion in all_expansions {
                if expansion.placeholder != batch_placeholder {
                    command = command.replace(expansion.placeholder, &expansion.files.join(" "));
                }
            }
            batches.push(command);
            current_batch = Vec::new();
            current_length = 0;
        }

        current_length += file_length;
        current_batch.push(file.as_str());
    }

    if !current_batch.is_empty() {
        let batch_files = current_batch.join(" ");
        let mut command = template.replace(batch_placeholder, &batch_files);
        for expansion in all_expansions {
            if expansion.placeholder != batch_placeholder {
                command = command.replace(expansion.placeholder, &expansion.files.join(" "));
            }
        }
        batches.push(command);
    }

    batches
}

fn execute_shell_command(command_string: &str, working_dir: Option<&String>) {
    let mut shell_command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" });

    if cfg!(windows) {
        shell_command.args(["/C", command_string]);
    } else {
        shell_command.args(["-c", command_string]);
    }

    if let Some(dir) = working_dir {
        shell_command.current_dir(dir);
    }

    let status = shell_command.status().expect("Failed to execute command");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
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
            let expanded_commands =
                match expand_file_placeholders(&command.run, &command.glob, &command.exclude) {
                    Some(commands) => commands,
                    None => {
                        println!(
                            "{} {} {}",
                            WRENCH,
                            command_name.cyan().bold(),
                            "(skip: no matching files)".yellow()
                        );
                        continue;
                    }
                };

            println!("{} {}", WRENCH, command_name.cyan().bold());

            let working_dir = resolve_working_directory(hook, &command.working_directory);

            for expanded_command in &expanded_commands {
                execute_shell_command(expanded_command, working_dir);
            }
        }
    }

    println!(
        "{} Hook {} completed successfully!",
        CHECKMARK,
        hook_name.green().bold()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_no_placeholders() {
        let result = expand_file_placeholders("cargo fmt -- --check", &[], &[]);
        assert_eq!(result, Some(vec!["cargo fmt -- --check".to_string()]));
    }

    #[test]
    fn test_has_file_placeholder_detection() {
        assert!(!has_file_placeholder("cargo fmt"));
        assert!(has_file_placeholder("eslint {staged_files}"));
        assert!(has_file_placeholder("test {push_files}"));
        assert!(has_file_placeholder("lint {all_files}"));
        assert!(has_file_placeholder("{staged_files} {all_files}"));
    }

    #[test]
    fn test_expand_simple_substitution() {
        let expansions = vec![PlaceholderExpansion {
            placeholder: STAGED_FILES_PLACEHOLDER,
            files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
        }];
        let result = expand_simple("eslint {staged_files}", &expansions);
        assert_eq!(result, "eslint src/main.rs src/lib.rs");
    }

    #[test]
    fn test_expand_simple_multiple_placeholders() {
        let expansions = vec![
            PlaceholderExpansion {
                placeholder: STAGED_FILES_PLACEHOLDER,
                files: vec!["a.rs".to_string()],
            },
            PlaceholderExpansion {
                placeholder: ALL_FILES_PLACEHOLDER,
                files: vec!["b.rs".to_string(), "c.rs".to_string()],
            },
        ];
        let result = expand_simple("diff {staged_files} -- {all_files}", &expansions);
        assert_eq!(result, "diff a.rs -- b.rs c.rs");
    }

    #[test]
    fn test_split_into_batches_single_batch() {
        let files: Vec<String> = vec!["a.rs".to_string(), "b.rs".to_string()];
        let expansions = vec![PlaceholderExpansion {
            placeholder: STAGED_FILES_PLACEHOLDER,
            files: files.clone(),
        }];

        let batches = split_into_batches(
            &files,
            "eslint {staged_files}",
            STAGED_FILES_PLACEHOLDER,
            &expansions,
        );

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], "eslint a.rs b.rs");
    }

    #[test]
    fn test_split_into_batches_multiple_batches() {
        let files: Vec<String> = (0..5000)
            .map(|index| format!("src/deeply/nested/module/submodule/component/file_{index}.rs"))
            .collect();

        let expansions = vec![PlaceholderExpansion {
            placeholder: STAGED_FILES_PLACEHOLDER,
            files: files.clone(),
        }];

        let batches = split_into_batches(
            &files,
            "eslint {staged_files}",
            STAGED_FILES_PLACEHOLDER,
            &expansions,
        );

        assert!(batches.len() > 1);

        for batch in &batches {
            assert!(batch.len() <= MAX_COMMAND_LENGTH);
            assert!(batch.starts_with("eslint "));
        }

        let total_files: usize = batches
            .iter()
            .map(|batch| batch.strip_prefix("eslint ").unwrap().split(' ').count())
            .sum();
        assert_eq!(total_files, 5000);
    }
}
