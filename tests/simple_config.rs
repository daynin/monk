use monk::*;

#[test]
fn test_simple_hook_config() {
    let yaml = r#"
pre-commit:
  commands:
    - cargo fmt
    - cargo clippy
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert!(config.hooks.contains_key("pre-commit"));

    let changed_files = vec!["src/main.rs".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &changed_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 2);
}

