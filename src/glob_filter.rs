use globset::{Glob, GlobSetBuilder};

fn normalize_pattern(pattern: &str) -> String {
    if pattern.contains('/') {
        pattern.to_string()
    } else {
        format!("**/{pattern}")
    }
}

fn build_glob_set(patterns: &[String]) -> globset::GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let normalized = normalize_pattern(pattern);
        builder.add(Glob::new(&normalized).expect("Invalid glob pattern"));
    }
    builder.build().expect("Failed to build glob set")
}

fn apply_include_filter(files: Vec<String>, patterns: &[String]) -> Vec<String> {
    if patterns.is_empty() {
        return files;
    }

    let glob_set = build_glob_set(patterns);
    files
        .into_iter()
        .filter(|file| glob_set.is_match(file))
        .collect()
}

fn apply_exclude_filter(files: Vec<String>, patterns: &[String]) -> Vec<String> {
    if patterns.is_empty() {
        return files;
    }

    let glob_set = build_glob_set(patterns);
    files
        .into_iter()
        .filter(|file| !glob_set.is_match(file))
        .collect()
}

pub fn filter_files_by_glob(
    files: Vec<String>,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Vec<String> {
    let included = apply_include_filter(files, include_patterns);
    apply_exclude_filter(included, exclude_patterns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_files() -> Vec<String> {
        vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "frontend/index.js".to_string(),
            "frontend/app.ts".to_string(),
            "frontend/style.css".to_string(),
            "frontend/bundle.min.js".to_string(),
            "README.md".to_string(),
        ]
    }

    #[test]
    fn test_filter_with_no_patterns() {
        let files = sample_files();
        let result = filter_files_by_glob(files.clone(), &[], &[]);
        assert_eq!(result, files);
    }

    #[test]
    fn test_include_single_extension() {
        let result = filter_files_by_glob(sample_files(), &["*.rs".to_string()], &[]);
        assert_eq!(result, vec!["src/main.rs", "src/lib.rs"]);
    }

    #[test]
    fn test_include_brace_expansion() {
        let result = filter_files_by_glob(sample_files(), &["*.{js,ts}".to_string()], &[]);
        assert_eq!(
            result,
            vec![
                "frontend/index.js",
                "frontend/app.ts",
                "frontend/bundle.min.js"
            ]
        );
    }

    #[test]
    fn test_include_multiple_patterns() {
        let result = filter_files_by_glob(
            sample_files(),
            &["*.rs".to_string(), "*.md".to_string()],
            &[],
        );
        assert_eq!(result, vec!["src/main.rs", "src/lib.rs", "README.md"]);
    }

    #[test]
    fn test_exclude_single_pattern() {
        let result = filter_files_by_glob(sample_files(), &[], &["*.rs".to_string()]);
        assert_eq!(
            result,
            vec![
                "frontend/index.js",
                "frontend/app.ts",
                "frontend/style.css",
                "frontend/bundle.min.js",
                "README.md",
            ]
        );
    }

    #[test]
    fn test_include_then_exclude() {
        let result = filter_files_by_glob(
            sample_files(),
            &["*.js".to_string()],
            &["*.min.js".to_string()],
        );
        assert_eq!(result, vec!["frontend/index.js"]);
    }

    #[test]
    fn test_all_filtered_returns_empty() {
        let result = filter_files_by_glob(sample_files(), &["*.py".to_string()], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pattern_with_directory() {
        let result = filter_files_by_glob(sample_files(), &["src/*.rs".to_string()], &[]);
        assert_eq!(result, vec!["src/main.rs", "src/lib.rs"]);
    }

    #[test]
    fn test_normalize_pattern_without_slash() {
        assert_eq!(normalize_pattern("*.rs"), "**/*.rs");
        assert_eq!(normalize_pattern("*.{js,ts}"), "**/*.{js,ts}");
    }

    #[test]
    fn test_normalize_pattern_with_slash() {
        assert_eq!(normalize_pattern("src/*.rs"), "src/*.rs");
        assert_eq!(normalize_pattern("frontend/**/*.js"), "frontend/**/*.js");
    }
}
