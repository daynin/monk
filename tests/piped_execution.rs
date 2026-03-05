use monk::*;

#[test]
fn test_piped_config_parsing() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  piped: true
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 1);
    assert!(hooks[0].piped);
    assert!(!hooks[0].follow);
    assert!(!hooks[0].parallel);
    assert_eq!(hooks[0].commands.len(), 2);
}

#[test]
fn test_piped_with_follow() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  piped: true
  follow: true
  commands:
    fmt:
      run: cargo fmt -- --check
    test:
      run: cargo test
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].piped);
    assert!(hooks[0].follow);
}

#[test]
fn test_piped_with_priority() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  piped: true
  commands:
    install:
      run: npm install
      priority: 1
    lint:
      run: eslint .
      priority: 2
    test:
      run: npm test
      priority: 3
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    let commands = &hooks[0].commands;
    assert_eq!(commands.get("install").unwrap().priority, Some(1));
    assert_eq!(commands.get("lint").unwrap().priority, Some(2));
    assert_eq!(commands.get("test").unwrap().priority, Some(3));
}

#[test]
fn test_priority_without_piped() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  commands:
    lint:
      run: eslint .
      priority: 2
    fmt:
      run: prettier --write .
      priority: 1
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(!hooks[0].piped);
    assert_eq!(hooks[0].commands.get("lint").unwrap().priority, Some(2));
    assert_eq!(hooks[0].commands.get("fmt").unwrap().priority, Some(1));
}

#[test]
fn test_piped_defaults_to_false() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(!hooks[0].piped);
    assert!(!hooks[0].follow);
}

#[test]
fn test_piped_in_toml() {
    let config = parse_toml_config(
        r#"
[pre-commit]
piped = true

[pre-commit.commands.install]
run = "npm install"
priority = 1

[pre-commit.commands.lint]
run = "eslint ."
priority = 2
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].piped);
    assert_eq!(hooks[0].commands.get("install").unwrap().priority, Some(1));
    assert_eq!(hooks[0].commands.get("lint").unwrap().priority, Some(2));
}

#[test]
fn test_piped_with_follow_in_toml() {
    let config = parse_toml_config(
        r#"
[post-merge]
piped = true
follow = true

[post-merge.commands.bundle]
run = "bundle install"
priority = 1

[post-merge.commands.migrate]
run = "bundle exec rails db:migrate"
priority = 2
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "post-merge");
    assert!(hooks[0].piped);
    assert!(hooks[0].follow);
}

#[test]
fn test_piped_with_skip_and_glob() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  piped: true
  skip:
    - merge
  commands:
    lint:
      run: eslint {staged_files}
      glob: "*.js"
      priority: 1
    test:
      run: cargo test
      priority: 2
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert!(hooks[0].piped);
    assert_eq!(hooks[0].skip, vec![SkipCondition::Merge]);
    let lint = hooks[0].commands.get("lint").unwrap();
    assert_eq!(lint.glob, vec!["*.js"]);
    assert_eq!(lint.priority, Some(1));
}

#[test]
fn test_piped_path_based() {
    let config: Config = serde_yaml::from_str(
        r#"
pre-commit:
  paths:
    "frontend/":
      piped: true
      follow: true
      commands:
        install:
          run: npm install
          priority: 1
        lint:
          run: npm run lint
          priority: 2
    "backend/":
      piped: true
      commands:
        fmt:
          run: cargo fmt
          priority: 1
        test:
          run: cargo test
          priority: 2
"#,
    )
    .unwrap();

    let hooks = find_all_path_configs(&config, "pre-commit");
    assert_eq!(hooks.len(), 2);

    assert!(hooks[0].piped);
    assert!(hooks[0].follow);

    assert!(hooks[1].piped);
    assert!(!hooks[1].follow);
}
