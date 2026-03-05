use monk::*;

#[test]
fn test_toml_full_feature_config() {
    let config = parse_toml_config(
        r#"
[pre-commit]
parallel = true
skip = ["merge"]

[pre-commit.commands.fmt]
run = "cargo fmt -- --check"
glob = ["*.rs"]

[pre-commit.commands.clippy]
run = "cargo clippy -- -D warnings"

[pre-push.commands.test]
run = "cargo test"
"#,
    )
    .unwrap();

    let pre_commit = find_all_path_configs(&config, "pre-commit");
    assert_eq!(pre_commit.len(), 1);
    assert!(pre_commit[0].parallel);
    assert_eq!(pre_commit[0].skip, vec![SkipCondition::Merge]);
    assert_eq!(pre_commit[0].commands.len(), 2);

    let fmt = pre_commit[0].commands.get("fmt").unwrap();
    assert_eq!(fmt.run, "cargo fmt -- --check");
    assert_eq!(fmt.glob, vec!["*.rs"]);

    let pre_push = find_all_path_configs(&config, "pre-push");
    assert_eq!(pre_push.len(), 1);
    assert_eq!(pre_push[0].commands.get("test").unwrap().run, "cargo test");
}

#[test]
fn test_toml_with_command_skip_ref() {
    let config = parse_toml_config(
        r#"
[pre-push.commands.deploy]
run = "./deploy.sh"
skip = [{ ref = "main" }]

[pre-push.commands.test]
run = "cargo test"
skip = [{ ref = "release/*" }]
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-push");
    let deploy = hooks[0].commands.get("deploy").unwrap();
    assert_eq!(deploy.skip, vec![SkipCondition::Ref("main".to_string())]);

    let test_cmd = hooks[0].commands.get("test").unwrap();
    assert_eq!(
        test_cmd.skip,
        vec![SkipCondition::Ref("release/*".to_string())]
    );
}

#[test]
fn test_toml_path_based_config() {
    let config = parse_toml_config(
        r#"
[pre-commit.paths."frontend/"]
parallel = true

[pre-commit.paths."frontend/".commands.lint]
run = "npm run lint"

[pre-commit.paths."frontend/".commands.test]
run = "npm test"

[pre-commit.paths."backend/".commands.fmt]
run = "cargo fmt -- --check"
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 2);
}

#[test]
fn test_toml_with_working_directory() {
    let config = parse_toml_config(
        r#"
[pre-commit]
working_directory = "backend"

[pre-commit.commands.test]
run = "cargo test"
working_directory = "backend/core"
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks[0].working_directory, Some("backend".to_string()));
    assert_eq!(
        hooks[0].commands.get("test").unwrap().working_directory,
        Some("backend/core".to_string())
    );
}

#[test]
fn test_cross_format_merge_yaml_base_toml_local() {
    let merged = merge_toml_into_yaml(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
pre-push:
  commands:
    test:
      run: cargo test
"#,
        r#"
[pre-commit]
parallel = true

[pre-commit.commands.clippy]
run = "cargo clippy"

[pre-push]
skip = ["merge"]
"#,
    )
    .unwrap();

    let pre_commit = find_all_path_configs(&merged, "pre-commit");
    assert!(pre_commit[0].parallel);
    assert_eq!(pre_commit[0].commands.len(), 2);
    assert_eq!(
        pre_commit[0].commands.get("clippy").unwrap().run,
        "cargo clippy"
    );
    assert_eq!(
        pre_commit[0].commands.get("fmt").unwrap().run,
        "cargo fmt -- --check"
    );

    let pre_push = find_all_path_configs(&merged, "pre-push");
    assert_eq!(pre_push[0].skip, vec![SkipCondition::Merge]);
    assert_eq!(pre_push[0].commands.get("test").unwrap().run, "cargo test");
}
