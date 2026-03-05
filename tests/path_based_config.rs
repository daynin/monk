use monk::*;

#[test]
fn test_path_based_hook_config() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        - npm run lint
        - npm test
    "backend/":
      commands:
        - cargo fmt -- --check
        - cargo clippy
      working_directory: "backend"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    let frontend_files = vec!["frontend/src/App.js".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &frontend_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(
        hooks[0].commands.get_index(0).unwrap().1.run,
        "npm run lint"
    );

    let backend_files = vec!["backend/src/main.rs".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &backend_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(
        hooks[0].commands.get_index(0).unwrap().1.run,
        "cargo fmt -- --check"
    );
    assert_eq!(hooks[0].working_directory, Some("backend".to_string()));

    let mixed_files = vec![
        "frontend/src/App.js".to_string(),
        "backend/src/main.rs".to_string(),
    ];
    let hooks = find_matching_path_configs(&config, "pre-commit", &mixed_files);
    assert_eq!(hooks.len(), 2);
}

#[test]
fn test_no_matching_paths() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        - npm run lint
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    let unrelated_files = vec!["docs/README.md".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &unrelated_files);
    assert_eq!(hooks.len(), 0);
}
