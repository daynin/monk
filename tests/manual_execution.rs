use monk::*;

#[test]
fn test_find_all_path_configs_simple() {
    let yaml = r#"
pre-commit:
  commands:
    - cargo fmt
    - cargo clippy
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 2);
}

#[test]
fn test_find_all_path_configs_path_based() {
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
    "docs/":
      commands:
        - mdbook test
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 3);
    
    let command_counts: Vec<usize> = hooks.iter().map(|h| h.commands.len()).collect();
    assert!(command_counts.contains(&2));
    assert!(command_counts.contains(&1));
    
    let has_working_dir = hooks.iter().any(|h| h.working_directory.is_some());
    assert!(has_working_dir);
}

#[test]
fn test_find_all_path_configs_nonexistent_hook() {
    let yaml = r#"
pre-commit:
  commands:
    - cargo fmt
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-push");
    assert_eq!(hooks.len(), 0);
}

#[test]
fn test_find_all_vs_matching_path_configs() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        - npm run lint
    "backend/":
      commands:
        - cargo fmt
    "docs/":
      commands:
        - mdbook test
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    
    let all_hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(all_hooks.len(), 3);
    
    let frontend_files = vec!["frontend/src/App.js".to_string()];
    let matching_hooks = find_matching_path_configs(&config, "pre-commit", &frontend_files);
    assert_eq!(matching_hooks.len(), 1);
    
    let no_files: Vec<String> = vec![];
    let no_matching_hooks = find_matching_path_configs(&config, "pre-commit", &no_files);
    assert_eq!(no_matching_hooks.len(), 0);
}

#[test]
fn test_config_reading_success() {
    let temp_dir = std::env::temp_dir().join("monk_test_success");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let config_path = temp_dir.join("monk.yaml");
    
    let yaml_content = r#"
pre-commit:
  commands:
    - echo "test"
"#;
    std::fs::write(&config_path, yaml_content).unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let result = read_config();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.hooks.contains_key("pre-commit"));
    
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_config_reading_failure() {
    let temp_dir = std::env::temp_dir().join("monk_test_failure");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let result = read_config();
    assert!(result.is_err());
    
    std::env::set_current_dir(original_dir).unwrap();
    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_working_directory_configuration() {
    let yaml = r#"
pre-commit:
  paths:
    "api/":
      commands:
        - cargo test
      working_directory: "api"
    "frontend/":
      commands:
        - npm test
      working_directory: "frontend"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    
    assert_eq!(hooks.len(), 2);
    for hook in hooks {
        assert!(hook.working_directory.is_some());
        let wd = hook.working_directory.as_ref().unwrap();
        assert!(wd == "api" || wd == "frontend");
    }
}
