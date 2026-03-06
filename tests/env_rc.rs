use monk::{merge_yaml_configs, parse_toml_config, Config, HookConfig};

fn parse_config(yaml: &str) -> Config {
    serde_yaml::from_str(yaml).unwrap()
}

fn merge(base_yaml: &str, local_yaml: &str) -> Config {
    merge_yaml_configs(base_yaml, local_yaml).unwrap()
}

#[test]
fn test_env_config_parsing() {
    let config = parse_config(
        r#"
pre-commit:
  commands:
    lint:
      run: eslint .
      env:
        NODE_ENV: production
        FORCE_COLOR: "1"
    fmt:
      run: cargo fmt
"#,
    );
    if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
        let lint = hook.commands.get("lint").unwrap();
        assert_eq!(lint.env.len(), 2);
        assert_eq!(lint.env.get("NODE_ENV").unwrap(), "production");
        assert_eq!(lint.env.get("FORCE_COLOR").unwrap(), "1");

        let fmt = hook.commands.get("fmt").unwrap();
        assert!(fmt.env.is_empty());
    } else {
        panic!("Expected Simple hook config");
    }
}

#[test]
fn test_rc_config_parsing() {
    let config = parse_config(
        r#"
rc: .monkrc
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
    );
    assert_eq!(config.rc, Some(".monkrc".to_string()));
}

#[test]
fn test_rc_defaults_to_none() {
    let config = parse_config(
        r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
    );
    assert_eq!(config.rc, None);
}

#[test]
fn test_env_in_toml() {
    let config = parse_toml_config(
        r#"
[pre-commit.commands.lint]
run = "eslint ."

[pre-commit.commands.lint.env]
NODE_ENV = "production"
FORCE_COLOR = "1"
"#,
    )
    .unwrap();
    if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
        let lint = hook.commands.get("lint").unwrap();
        assert_eq!(lint.env.get("NODE_ENV").unwrap(), "production");
        assert_eq!(lint.env.get("FORCE_COLOR").unwrap(), "1");
    } else {
        panic!("Expected Simple hook config");
    }
}

#[test]
fn test_rc_in_toml() {
    let config = parse_toml_config(
        r#"
rc = ".monkrc"

[pre-commit.commands.fmt]
run = "cargo fmt"
"#,
    )
    .unwrap();
    assert_eq!(config.rc, Some(".monkrc".to_string()));
}

#[test]
fn test_env_merge() {
    let merged = merge(
        r#"
pre-commit:
  commands:
    lint:
      run: eslint .
      env:
        NODE_ENV: development
        DEBUG: "true"
"#,
        r#"
pre-commit:
  commands:
    lint:
      run: eslint .
      env:
        NODE_ENV: production
        FORCE_COLOR: "1"
"#,
    );
    if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
        let lint = hook.commands.get("lint").unwrap();
        assert_eq!(lint.env.get("NODE_ENV").unwrap(), "production");
        assert_eq!(lint.env.get("DEBUG").unwrap(), "true");
        assert_eq!(lint.env.get("FORCE_COLOR").unwrap(), "1");
    } else {
        panic!("Expected Simple hook config");
    }
}

#[test]
fn test_rc_merge() {
    let merged = merge(
        r#"
rc: .monkrc
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
        r#"
rc: .local-monkrc
"#,
    );
    assert_eq!(merged.rc, Some(".local-monkrc".to_string()));
}

#[test]
fn test_rc_preserved_when_local_has_no_rc() {
    let merged = merge(
        r#"
rc: .monkrc
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
        r#"
pre-commit:
  commands:
    clippy:
      run: cargo clippy
"#,
    );
    assert_eq!(merged.rc, Some(".monkrc".to_string()));
}

#[test]
fn test_rc_with_env_together() {
    let config = parse_config(
        r#"
rc: .monkrc
pre-commit:
  commands:
    lint:
      run: eslint .
      env:
        NODE_ENV: production
    fmt:
      run: cargo fmt
"#,
    );
    assert_eq!(config.rc, Some(".monkrc".to_string()));
    if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
        let lint = hook.commands.get("lint").unwrap();
        assert_eq!(lint.env.get("NODE_ENV").unwrap(), "production");
        assert!(hook.commands.get("fmt").unwrap().env.is_empty());
    } else {
        panic!("Expected Simple hook config");
    }
}

#[test]
fn test_env_in_path_based() {
    let config = parse_config(
        r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: eslint .
          env:
            NODE_ENV: production
    "backend/":
      commands:
        fmt:
          run: cargo fmt
"#,
    );
    if let HookConfig::PathBased { paths } = config.hooks.get("pre-commit").unwrap() {
        let frontend = paths.get("frontend/").unwrap();
        let lint = frontend.commands.get("lint").unwrap();
        assert_eq!(lint.env.get("NODE_ENV").unwrap(), "production");

        let backend = paths.get("backend/").unwrap();
        assert!(backend.commands.get("fmt").unwrap().env.is_empty());
    } else {
        panic!("Expected PathBased hook config");
    }
}
