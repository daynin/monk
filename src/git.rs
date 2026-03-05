fn run_git_command(args: &[&str]) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .expect("Failed to run git command");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

pub fn get_staged_files() -> Vec<String> {
    run_git_command(&["diff", "--cached", "--name-only", "--diff-filter=ACM"])
}

pub fn get_push_files() -> Vec<String> {
    let files = run_git_command(&["diff", "--name-only", "@{push}..HEAD"]);
    if !files.is_empty() {
        return files;
    }
    run_git_command(&["diff", "--name-only", "@{upstream}..HEAD"])
}

pub fn get_all_tracked_files() -> Vec<String> {
    run_git_command(&["ls-files"])
}

pub fn get_changed_files() -> Vec<String> {
    let staged = get_staged_files();
    if !staged.is_empty() {
        return staged;
    }
    run_git_command(&["diff", "--name-only", "HEAD~1"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_git_command_filters_empty_lines() {
        let files = run_git_command(&["version"]);
        assert!(!files.is_empty());
        assert!(files[0].starts_with("git version"));
    }

    #[test]
    fn test_get_all_tracked_files_includes_current_file() {
        let files = get_all_tracked_files();
        assert!(files.iter().any(|file| file == "src/git.rs"));
    }
}
