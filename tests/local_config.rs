use monk::*;

fn merge(base_yaml: &str, local_yaml: &str) -> Config {
    merge_yaml_configs(base_yaml, local_yaml).unwrap()
}

#[test]
fn test_local_adds_skip_to_existing_hook() {
    let merged = merge(
        r#"
pre-push:
  commands:
    test:
      run: cargo test
"#,
        r#"
pre-push:
  skip:
    - ref: main
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-push");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].skip, vec![SkipCondition::Ref("main".to_string())]);
    assert_eq!(hooks[0].commands.get("test").unwrap().run, "cargo test");
}

#[test]
fn test_local_overrides_and_adds_commands() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
"#,
        r#"
pre-commit:
  commands:
    clippy:
      run: cargo clippy
    mycheck:
      run: ./check.sh
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-commit");
    assert_eq!(hooks[0].commands.len(), 3);
    assert_eq!(
        hooks[0].commands.get("fmt").unwrap().run,
        "cargo fmt -- --check"
    );
    assert_eq!(hooks[0].commands.get("clippy").unwrap().run, "cargo clippy");
    assert_eq!(hooks[0].commands.get("mycheck").unwrap().run, "./check.sh");
}

#[test]
fn test_local_enables_parallel() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
    clippy:
      run: cargo clippy
"#,
        r#"
pre-commit:
  parallel: true
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-commit");
    assert!(hooks[0].parallel);
    assert_eq!(hooks[0].commands.len(), 2);
}

#[test]
fn test_local_adds_new_hook() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
        r#"
commit-msg:
  commands:
    lint:
      run: commitlint --edit
"#,
    );
    assert_eq!(merged.hooks.len(), 2);
    let hooks = find_all_path_configs(&merged, "commit-msg");
    assert_eq!(
        hooks[0].commands.get("lint").unwrap().run,
        "commitlint --edit"
    );
}

#[test]
fn test_local_base_only_hooks_preserved() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
pre-push:
  commands:
    test:
      run: cargo test
"#,
        r#"
pre-commit:
  parallel: true
"#,
    );
    assert_eq!(merged.hooks.len(), 2);
    let push_hooks = find_all_path_configs(&merged, "pre-push");
    assert_eq!(
        push_hooks[0].commands.get("test").unwrap().run,
        "cargo test"
    );
}

#[test]
fn test_local_command_override_replaces_glob() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
      glob: "*.js"
"#,
        r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
      glob:
        - "*.js"
        - "*.ts"
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-commit");
    let lint = hooks[0].commands.get("lint").unwrap();
    assert_eq!(lint.glob, vec!["*.js", "*.ts"]);
}

#[test]
fn test_local_only_scalars_no_commands_key() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
    clippy:
      run: cargo clippy
"#,
        r#"
pre-commit:
  parallel: true
  skip:
    - merge
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-commit");
    assert!(hooks[0].parallel);
    assert_eq!(hooks[0].skip, vec![SkipCondition::Merge]);
    assert_eq!(hooks[0].commands.len(), 2);
    assert_eq!(hooks[0].commands.get("fmt").unwrap().run, "cargo fmt");
    assert_eq!(hooks[0].commands.get("clippy").unwrap().run, "cargo clippy");
}

#[test]
fn test_local_working_directory_override() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    test:
      run: cargo test
"#,
        r#"
pre-commit:
  working_directory: backend
"#,
    );
    let hooks = find_all_path_configs(&merged, "pre-commit");
    assert_eq!(hooks[0].working_directory, Some("backend".to_string()));
    assert_eq!(hooks[0].commands.get("test").unwrap().run, "cargo test");
}
