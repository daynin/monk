use monk::*;

#[test]
fn test_config_parallel_parses() {
    let yaml = r#"
pre-commit:
  parallel: true
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
    test:
      run: cargo test
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);
    assert!(hooks[0].parallel);
    assert_eq!(hooks[0].commands.len(), 3);
}

#[test]
fn test_config_parallel_backward_compat() {
    let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(!hooks[0].parallel);
}

#[test]
fn test_config_parallel_legacy_format() {
    let yaml = r#"
pre-commit:
  parallel: true
  commands:
    - cargo fmt
    - cargo clippy
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].parallel);
    assert_eq!(hooks[0].commands.len(), 2);
}

#[test]
fn test_config_parallel_path_based() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      parallel: true
      commands:
        lint:
          run: npm run lint
        test:
          run: npm test
    "backend/":
      commands:
        fmt:
          run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 2);
    assert!(hooks[0].parallel);
    assert!(!hooks[1].parallel);
}

#[test]
fn test_config_parallel_with_glob() {
    let yaml = r#"
pre-commit:
  parallel: true
  commands:
    lint-js:
      run: eslint {staged_files}
      glob: "*.{js,ts}"
    lint-rs:
      run: cargo clippy
      glob: "*.rs"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].parallel);

    let lint_js = hooks[0].commands.get("lint-js").unwrap();
    assert_eq!(lint_js.glob, vec!["*.{js,ts}"]);

    let lint_rs = hooks[0].commands.get("lint-rs").unwrap();
    assert_eq!(lint_rs.glob, vec!["*.rs"]);
}
