use indexmap::IndexMap;
use serde::de::Deserializer;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(flatten)]
    pub hooks: IndexMap<String, HookConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum HookConfig {
    Simple(Hook),
    PathBased { paths: IndexMap<String, Hook> },
}

#[derive(Deserialize, Debug)]
pub struct Hook {
    #[serde(deserialize_with = "deserialize_commands")]
    pub commands: IndexMap<String, Command>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Command {
    pub run: String,
    #[serde(default)]
    pub working_directory: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CommandsFormat {
    Named(IndexMap<String, Command>),
    Legacy(Vec<String>),
}

fn deserialize_commands<'de, D>(deserializer: D) -> Result<IndexMap<String, Command>, D::Error>
where
    D: Deserializer<'de>,
{
    match CommandsFormat::deserialize(deserializer)? {
        CommandsFormat::Named(named) => Ok(named),
        CommandsFormat::Legacy(strings) => {
            let commands = strings
                .into_iter()
                .enumerate()
                .map(|(index, run)| {
                    let name = format!("cmd{}", index + 1);
                    let command = Command {
                        run,
                        working_directory: None,
                    };
                    (name, command)
                })
                .collect();
            Ok(commands)
        }
    }
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("monk.yaml")?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_legacy_commands_format() {
        let yaml = r#"
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let hook_config = config.hooks.get("pre-commit").unwrap();

        if let HookConfig::Simple(hook) = hook_config {
            assert_eq!(hook.commands.len(), 2);

            let (first_name, first_cmd) = hook.commands.get_index(0).unwrap();
            assert_eq!(first_name, "cmd1");
            assert_eq!(first_cmd.run, "cargo fmt -- --check");

            let (second_name, second_cmd) = hook.commands.get_index(1).unwrap();
            assert_eq!(second_name, "cmd2");
            assert_eq!(second_cmd.run, "cargo clippy -- -D warnings");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_deserialize_named_commands_format() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let hook_config = config.hooks.get("pre-commit").unwrap();

        if let HookConfig::Simple(hook) = hook_config {
            assert_eq!(hook.commands.len(), 2);

            let (first_name, first_cmd) = hook.commands.get_index(0).unwrap();
            assert_eq!(first_name, "fmt");
            assert_eq!(first_cmd.run, "cargo fmt -- --check");

            let (second_name, second_cmd) = hook.commands.get_index(1).unwrap();
            assert_eq!(second_name, "clippy");
            assert_eq!(second_cmd.run, "cargo clippy -- -D warnings");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_named_commands_preserve_insertion_order() {
        let yaml = r#"
pre-commit:
  commands:
    zebra:
      run: echo zebra
    alpha:
      run: echo alpha
    middle:
      run: echo middle
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let names: Vec<&String> = hook.commands.keys().collect();
            assert_eq!(names, vec!["zebra", "alpha", "middle"]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_hook_level_working_directory() {
        let yaml = r#"
pre-commit:
  commands:
    test:
      run: cargo test
  working_directory: backend
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.working_directory, Some("backend".to_string()));
            assert_eq!(hook.commands.get("test").unwrap().run, "cargo test");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_command_level_working_directory() {
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

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let frontend = hook.commands.get("frontend_lint").unwrap();
            assert_eq!(frontend.run, "npm run lint");
            assert_eq!(frontend.working_directory, Some("frontend".to_string()));

            let backend = hook.commands.get("backend_test").unwrap();
            assert_eq!(backend.run, "cargo test");
            assert_eq!(backend.working_directory, Some("backend".to_string()));
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_path_based_with_named_commands() {
        let yaml = r#"
pre-commit:
  paths:
    "frontend/":
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

        if let HookConfig::PathBased { paths } = config.hooks.get("pre-commit").unwrap() {
            let frontend = paths.get("frontend/").unwrap();
            assert_eq!(frontend.commands.len(), 2);
            assert_eq!(frontend.commands.get("lint").unwrap().run, "npm run lint");
            assert_eq!(frontend.commands.get("test").unwrap().run, "npm test");

            let backend = paths.get("backend/").unwrap();
            assert_eq!(backend.commands.len(), 1);
            assert_eq!(
                backend.commands.get("fmt").unwrap().run,
                "cargo fmt -- --check"
            );
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_empty_commands_list() {
        let yaml = r#"
pre-commit:
  commands: []
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.len(), 0);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_multiple_hooks_in_config() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
pre-push:
  commands:
    test:
      run: cargo test
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.hooks.len(), 2);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks.contains_key("pre-push"));
    }
}
