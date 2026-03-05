use monk::*;

#[test]
fn test_skip_single_merge() {
    let yaml = r#"
pre-commit:
  skip: merge
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].skip, vec![SkipCondition::Merge]);
}

#[test]
fn test_skip_single_rebase() {
    let yaml = r#"
pre-commit:
  skip: rebase
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].skip, vec![SkipCondition::Rebase]);
}

#[test]
fn test_skip_list_merge_and_rebase() {
    let yaml = r#"
pre-commit:
  skip:
    - merge
    - rebase
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(
        hooks[0].skip,
        vec![SkipCondition::Merge, SkipCondition::Rebase]
    );
}

#[test]
fn test_skip_ref_exact_branch() {
    let yaml = r#"
pre-push:
  commands:
    deploy:
      run: ./deploy.sh
      skip:
        - ref: main
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-push");
    let deploy = hooks[0].commands.get("deploy").unwrap();
    assert_eq!(deploy.skip, vec![SkipCondition::Ref("main".to_string())]);
}

#[test]
fn test_skip_ref_glob_pattern() {
    let yaml = r#"
pre-push:
  commands:
    test:
      run: cargo test
      skip:
        - ref: "release/*"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-push");
    let test_cmd = hooks[0].commands.get("test").unwrap();
    assert_eq!(
        test_cmd.skip,
        vec![SkipCondition::Ref("release/*".to_string())]
    );
}

#[test]
fn test_skip_run_shell_condition() {
    let yaml = r#"
pre-commit:
  commands:
    slow-test:
      run: cargo test --all
      skip:
        - run: "test -n \"$CI\""
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    let cmd = hooks[0].commands.get("slow-test").unwrap();
    assert_eq!(
        cmd.skip,
        vec![SkipCondition::Run("test -n \"$CI\"".to_string())]
    );
}

#[test]
fn test_skip_mixed_conditions() {
    let yaml = r#"
pre-commit:
  skip:
    - merge
    - rebase
    - ref: main
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(
        hooks[0].skip,
        vec![
            SkipCondition::Merge,
            SkipCondition::Rebase,
            SkipCondition::Ref("main".to_string()),
        ]
    );
}

#[test]
fn test_skip_defaults_to_empty() {
    let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].skip.is_empty());
    assert!(hooks[0].commands.get("fmt").unwrap().skip.is_empty());
}

#[test]
fn test_skip_on_both_hook_and_command() {
    let yaml = r#"
pre-commit:
  skip:
    - merge
  commands:
    fmt:
      run: cargo fmt -- --check
      skip:
        - ref: main
    clippy:
      run: cargo clippy
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].skip, vec![SkipCondition::Merge]);
    assert_eq!(
        hooks[0].commands.get("fmt").unwrap().skip,
        vec![SkipCondition::Ref("main".to_string())]
    );
    assert!(hooks[0].commands.get("clippy").unwrap().skip.is_empty());
}

#[test]
fn test_skip_with_path_based_config() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      skip:
        - merge
      commands:
        lint:
          run: npm run lint
    "backend/":
      commands:
        fmt:
          run: cargo fmt -- --check
          skip:
            - ref: "release/*"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].skip, vec![SkipCondition::Merge]);
    assert!(hooks[1].skip.is_empty());
    assert_eq!(
        hooks[1].commands.get("fmt").unwrap().skip,
        vec![SkipCondition::Ref("release/*".to_string())]
    );
}

#[test]
fn test_skip_with_parallel() {
    let yaml = r#"
pre-commit:
  parallel: true
  skip:
    - rebase
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].parallel);
    assert_eq!(hooks[0].skip, vec![SkipCondition::Rebase]);
}
