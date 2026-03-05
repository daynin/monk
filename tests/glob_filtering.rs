use monk::*;

#[test]
fn test_config_with_glob_parses() {
    let yaml = r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
      glob: "*.{js,ts}"
    fmt:
      run: prettier --write {staged_files}
      glob:
        - "*.js"
        - "*.css"
      exclude: "*.min.js"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);

    let lint = hooks[0].commands.get("lint").unwrap();
    assert_eq!(lint.run, "eslint {staged_files}");
    assert_eq!(lint.glob, vec!["*.{js,ts}"]);
    assert!(lint.exclude.is_empty());

    let fmt = hooks[0].commands.get("fmt").unwrap();
    assert_eq!(fmt.run, "prettier --write {staged_files}");
    assert_eq!(fmt.glob, vec!["*.js", "*.css"]);
    assert_eq!(fmt.exclude, vec!["*.min.js"]);
}

#[test]
fn test_config_glob_with_path_based() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: eslint {staged_files}
          glob: "*.{js,ts}"
    "backend/":
      commands:
        fmt:
          run: cargo fmt {staged_files}
          glob: "*.rs"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 2);

    let frontend_lint = hooks[0].commands.get("lint").unwrap();
    assert_eq!(frontend_lint.glob, vec!["*.{js,ts}"]);

    let backend_fmt = hooks[1].commands.get("fmt").unwrap();
    assert_eq!(backend_fmt.glob, vec!["*.rs"]);
}

#[test]
fn test_config_glob_backward_compat() {
    let yaml = r#"
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);

    for (_name, command) in &hooks[0].commands {
        assert!(command.glob.is_empty());
        assert!(command.exclude.is_empty());
    }
}

#[test]
fn test_filter_files_basic() {
    let files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
        "frontend/app.js".to_string(),
        "frontend/style.css".to_string(),
    ];

    let result = filter_files_by_glob(files, &["*.rs".to_string()], &[]);
    assert_eq!(result, vec!["src/main.rs", "src/lib.rs"]);
}

#[test]
fn test_filter_files_include_and_exclude() {
    let files = vec![
        "app.js".to_string(),
        "utils.js".to_string(),
        "bundle.min.js".to_string(),
        "style.css".to_string(),
    ];

    let result = filter_files_by_glob(files, &["*.js".to_string()], &["*.min.js".to_string()]);
    assert_eq!(result, vec!["app.js", "utils.js"]);
}
