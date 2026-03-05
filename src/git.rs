pub fn get_changed_files() -> Vec<String> {
    let staged_output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .expect("Failed to run git command");

    let staged_files: Vec<String> = String::from_utf8_lossy(&staged_output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if !staged_files.is_empty() {
        return staged_files;
    }

    let diff_output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .output()
        .expect("Failed to get changed files");

    String::from_utf8_lossy(&diff_output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
