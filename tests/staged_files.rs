use monk::*;

#[test]
fn test_config_with_staged_files_placeholder() {
    let yaml = r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
    fmt:
      run: prettier --write {staged_files}
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 2);

    let lint = hooks[0].commands.get("lint").unwrap();
    assert_eq!(lint.run, "eslint {staged_files}");

    let fmt = hooks[0].commands.get("fmt").unwrap();
    assert_eq!(fmt.run, "prettier --write {staged_files}");
}

#[test]
fn test_config_with_push_files_placeholder() {
    let yaml = r#"
pre-push:
  commands:
    test:
      run: cargo test {push_files}
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-push");
    assert_eq!(hooks.len(), 1);
    assert_eq!(
        hooks[0].commands.get("test").unwrap().run,
        "cargo test {push_files}"
    );
}

#[test]
fn test_config_with_all_files_placeholder() {
    let yaml = r#"
pre-commit:
  commands:
    check:
      run: eslint {all_files}
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(
        hooks[0].commands.get("check").unwrap().run,
        "eslint {all_files}"
    );
}

#[test]
fn test_config_mixed_placeholders_and_plain() {
    let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    lint:
      run: eslint {staged_files}
    clippy:
      run: cargo clippy -- -D warnings
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].commands.len(), 3);

    assert_eq!(
        hooks[0].commands.get("fmt").unwrap().run,
        "cargo fmt -- --check"
    );
    assert_eq!(
        hooks[0].commands.get("lint").unwrap().run,
        "eslint {staged_files}"
    );
    assert_eq!(
        hooks[0].commands.get("clippy").unwrap().run,
        "cargo clippy -- -D warnings"
    );
}

#[test]
fn test_expand_no_placeholders_returns_unchanged() {
    let result = expand_file_placeholders("cargo fmt -- --check", &[], &[]);
    assert_eq!(result, Some(vec!["cargo fmt -- --check".to_string()]));
}

#[test]
fn test_expand_all_files_returns_populated_list() {
    let result = expand_file_placeholders("wc -l {all_files}", &[], &[]);
    assert!(result.is_some());
    let commands = result.unwrap();
    assert!(!commands.is_empty());
    assert!(!commands[0].contains("{all_files}"));
    assert!(commands[0].starts_with("wc -l "));
}

#[test]
fn test_path_based_with_placeholders() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: eslint {staged_files}
    "backend/":
      commands:
        fmt:
          run: cargo fmt {staged_files}
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 2);

    assert_eq!(
        hooks[0].commands.get("lint").unwrap().run,
        "eslint {staged_files}"
    );
    assert_eq!(
        hooks[1].commands.get("fmt").unwrap().run,
        "cargo fmt {staged_files}"
    );
}
