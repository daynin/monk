use monk::*;

#[test]
fn test_multiple_rust_submodules() {
    let yaml = r#"
pre-commit:
  paths:
    "api/":
      commands:
        - cargo fmt -- --check
        - cargo clippy -- -D warnings
      working_directory: "api"
    "worker/":
      commands:
        - cargo fmt -- --check
        - cargo clippy -- -D warnings
        - cargo test
      working_directory: "worker"
    "shared/":
      commands:
        - cargo fmt -- --check
        - cargo clippy -- -D warnings
      working_directory: "shared"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    let api_files = vec!["api/src/main.rs".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &api_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].working_directory, Some("api".to_string()));

    let worker_files = vec!["worker/src/lib.rs".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &worker_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 3);

    let multiple_modules = vec![
        "api/src/main.rs".to_string(),
        "shared/src/lib.rs".to_string(),
    ];
    let hooks = find_matching_path_configs(&config, "pre-commit", &multiple_modules);
    assert_eq!(hooks.len(), 2);
}

