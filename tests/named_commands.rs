use monk::*;

#[test]
fn test_named_commands_simple() {
    let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 2);

    let fmt_command = hooks[0].commands.get("fmt").unwrap();
    assert_eq!(fmt_command.run, "cargo fmt -- --check");

    let clippy_command = hooks[0].commands.get("clippy").unwrap();
    assert_eq!(clippy_command.run, "cargo clippy -- -D warnings");
}

#[test]
fn test_named_commands_path_based() {
    let yaml = r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: npm run lint
        typecheck:
          run: npx tsc --noEmit
    "backend/":
      commands:
        fmt:
          run: cargo fmt -- --check
      working_directory: "backend"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    let frontend_files = vec!["frontend/index.ts".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &frontend_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 2);
    assert_eq!(hooks[0].commands.get("lint").unwrap().run, "npm run lint");
    assert_eq!(
        hooks[0].commands.get("typecheck").unwrap().run,
        "npx tsc --noEmit"
    );

    let backend_files = vec!["backend/src/main.rs".to_string()];
    let hooks = find_matching_path_configs(&config, "pre-commit", &backend_files);
    assert_eq!(hooks.len(), 1);
    assert_eq!(
        hooks[0].commands.get("fmt").unwrap().run,
        "cargo fmt -- --check"
    );
    assert_eq!(hooks[0].working_directory, Some("backend".to_string()));
}

#[test]
fn test_mixed_hooks_different_formats() {
    let yaml = r#"
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy
pre-push:
  commands:
    test:
      run: cargo test
    integration:
      run: cargo test --test integration
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    let pre_commit_hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(pre_commit_hooks.len(), 1);
    assert_eq!(pre_commit_hooks[0].commands.len(), 2);

    let (name, command) = pre_commit_hooks[0].commands.get_index(0).unwrap();
    assert_eq!(name, "cmd1");
    assert_eq!(command.run, "cargo fmt -- --check");

    let pre_push_hooks = find_all_path_configs(&config, "pre-push");
    assert_eq!(pre_push_hooks.len(), 1);
    assert_eq!(pre_push_hooks[0].commands.len(), 2);
    assert_eq!(
        pre_push_hooks[0].commands.get("test").unwrap().run,
        "cargo test"
    );
    assert_eq!(
        pre_push_hooks[0].commands.get("integration").unwrap().run,
        "cargo test --test integration"
    );
}

#[test]
fn test_legacy_auto_generated_names() {
    let yaml = r#"
pre-commit:
  commands:
    - echo first
    - echo second
    - echo third
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);

    let command_names: Vec<&String> = hooks[0].commands.keys().collect();
    assert_eq!(command_names, vec!["cmd1", "cmd2", "cmd3"]);

    let run_strings: Vec<&str> = hooks[0]
        .commands
        .values()
        .map(|cmd| cmd.run.as_str())
        .collect();
    assert_eq!(run_strings, vec!["echo first", "echo second", "echo third"]);
}

#[test]
fn test_named_commands_with_command_level_working_directory() {
    let yaml = r#"
pre-commit:
  commands:
    frontend_lint:
      run: npm run lint
      working_directory: frontend
    backend_test:
      run: cargo test
      working_directory: backend
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);

    let frontend = hooks[0].commands.get("frontend_lint").unwrap();
    assert_eq!(frontend.run, "npm run lint");
    assert_eq!(frontend.working_directory, Some("frontend".to_string()));

    let backend = hooks[0].commands.get("backend_test").unwrap();
    assert_eq!(backend.run, "cargo test");
    assert_eq!(backend.working_directory, Some("backend".to_string()));
}

#[test]
fn test_named_commands_preserve_order() {
    let yaml = r#"
pre-commit:
  commands:
    ztest:
      run: echo z
    aformat:
      run: echo a
    mlint:
      run: echo m
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "pre-commit");

    let command_names: Vec<&String> = hooks[0].commands.keys().collect();
    assert_eq!(command_names, vec!["ztest", "aformat", "mlint"]);
}

#[test]
fn test_single_named_command() {
    let yaml = r#"
commit-msg:
  commands:
    validate:
      run: ./scripts/check-commit-msg.sh
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    let hooks = find_all_path_configs(&config, "commit-msg");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].commands.len(), 1);
    assert_eq!(
        hooks[0].commands.get("validate").unwrap().run,
        "./scripts/check-commit-msg.sh"
    );
}
