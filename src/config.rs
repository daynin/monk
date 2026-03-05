use indexmap::IndexMap;
use serde::de::Deserializer;
use serde::Deserialize;
use std::fs;
use std::path::Path;

enum ConfigFormat {
    Yaml,
    Toml,
}

const CONFIG_EXTENSIONS: &[(&str, ConfigFormat)] =
    &[(".yaml", ConfigFormat::Yaml), (".toml", ConfigFormat::Toml)];

fn find_config_file(base_name: &str) -> Option<(String, &'static ConfigFormat)> {
    for (extension, format) in CONFIG_EXTENSIONS {
        let path = format!("{base_name}{extension}");
        if Path::new(&path).exists() {
            return Some((path, format));
        }
    }
    None
}

fn parse_to_yaml_value(
    content: &str,
    format: &ConfigFormat,
) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    match format {
        ConfigFormat::Yaml => Ok(serde_yaml::from_str(content)?),
        ConfigFormat::Toml => {
            let toml_value: toml::Value = toml::from_str(content)?;
            Ok(serde_yaml::to_value(toml_value)?)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkipCondition {
    Merge,
    Rebase,
    Ref(String),
    Run(String),
}

fn deserialize_skip_conditions<'de, D>(deserializer: D) -> Result<Vec<SkipCondition>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = match Option::<serde_yaml::Value>::deserialize(deserializer)? {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    let raw_list = match value {
        serde_yaml::Value::Sequence(seq) => seq,
        other => vec![other],
    };

    raw_list
        .into_iter()
        .map(|item| parse_skip_condition(item).map_err(serde::de::Error::custom))
        .collect()
}

fn parse_skip_condition(value: serde_yaml::Value) -> Result<SkipCondition, String> {
    match value {
        serde_yaml::Value::String(keyword) => match keyword.as_str() {
            "merge" => Ok(SkipCondition::Merge),
            "rebase" => Ok(SkipCondition::Rebase),
            other => Err(format!("Unknown skip condition: {other}")),
        },
        serde_yaml::Value::Mapping(map) => {
            if let Some(pattern) = map.get(serde_yaml::Value::String("ref".to_string())) {
                let pattern = pattern
                    .as_str()
                    .ok_or("'ref' value must be a string")?
                    .to_string();
                Ok(SkipCondition::Ref(pattern))
            } else if let Some(command) = map.get(serde_yaml::Value::String("run".to_string())) {
                let command = command
                    .as_str()
                    .ok_or("'run' value must be a string")?
                    .to_string();
                Ok(SkipCondition::Run(command))
            } else {
                Err("Skip map must have 'ref' or 'run' key".to_string())
            }
        }
        _ => Err("Skip condition must be a string or map".to_string()),
    }
}

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
#[serde(deny_unknown_fields)]
pub struct Hook {
    #[serde(default, deserialize_with = "deserialize_commands")]
    pub commands: IndexMap<String, Command>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub parallel: bool,
    #[serde(default)]
    pub piped: bool,
    #[serde(default)]
    pub follow: bool,
    #[serde(default, deserialize_with = "deserialize_skip_conditions")]
    pub skip: Vec<SkipCondition>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Command {
    pub run: String,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_list")]
    pub glob: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_list")]
    pub exclude: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_skip_conditions")]
    pub skip: Vec<SkipCondition>,
    #[serde(default)]
    pub priority: Option<u32>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrList {
    Single(String),
    Multiple(Vec<String>),
}

fn deserialize_string_or_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<StringOrList>::deserialize(deserializer)? {
        Some(StringOrList::Single(pattern)) => Ok(vec![pattern]),
        Some(StringOrList::Multiple(patterns)) => Ok(patterns),
        None => Ok(Vec::new()),
    }
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
                        glob: Vec::new(),
                        exclude: Vec::new(),
                        skip: Vec::new(),
                        priority: None,
                    };
                    (name, command)
                })
                .collect();
            Ok(commands)
        }
    }
}

fn deep_merge_yaml(base: serde_yaml::Value, local: serde_yaml::Value) -> serde_yaml::Value {
    match (base, local) {
        (serde_yaml::Value::Mapping(mut base_map), serde_yaml::Value::Mapping(local_map)) => {
            for (key, local_value) in local_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => deep_merge_yaml(base_value, local_value),
                    None => local_value,
                };
                base_map.insert(key, merged);
            }
            serde_yaml::Value::Mapping(base_map)
        }
        (_, local) => local,
    }
}

fn is_hook_variant_change(base: &serde_yaml::Value, local: &serde_yaml::Value) -> bool {
    let paths_key = serde_yaml::Value::String("paths".to_string());
    let base_has_paths = base
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(&paths_key));
    let local_has_paths = local
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(&paths_key));
    base_has_paths != local_has_paths
}

fn merge_top_level_yaml(base: serde_yaml::Value, local: serde_yaml::Value) -> serde_yaml::Value {
    match (base, local) {
        (serde_yaml::Value::Mapping(mut base_map), serde_yaml::Value::Mapping(local_map)) => {
            for (key, local_value) in local_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => {
                        if is_hook_variant_change(&base_value, &local_value) {
                            local_value
                        } else {
                            deep_merge_yaml(base_value, local_value)
                        }
                    }
                    None => local_value,
                };
                base_map.insert(key, merged);
            }
            serde_yaml::Value::Mapping(base_map)
        }
        (_, local) => local,
    }
}

pub fn parse_toml_config(toml_content: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let value = parse_to_yaml_value(toml_content, &ConfigFormat::Toml)?;
    Ok(serde_yaml::from_value(value)?)
}

pub fn merge_yaml_configs(
    base_yaml: &str,
    local_yaml: &str,
) -> Result<Config, Box<dyn std::error::Error>> {
    let base_value: serde_yaml::Value = serde_yaml::from_str(base_yaml)?;
    let local_value: serde_yaml::Value = serde_yaml::from_str(local_yaml)?;
    let merged_value = merge_top_level_yaml(base_value, local_value);
    let config: Config = serde_yaml::from_value(merged_value)?;
    Ok(config)
}

pub fn merge_toml_into_yaml(
    base_yaml: &str,
    local_toml: &str,
) -> Result<Config, Box<dyn std::error::Error>> {
    let base_value = parse_to_yaml_value(base_yaml, &ConfigFormat::Yaml)?;
    let local_value = parse_to_yaml_value(local_toml, &ConfigFormat::Toml)?;
    let merged_value = merge_top_level_yaml(base_value, local_value);
    Ok(serde_yaml::from_value(merged_value)?)
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let (config_path, config_format) =
        find_config_file("monk").ok_or("No monk.yaml or monk.toml found")?;
    let config_content = fs::read_to_string(&config_path)?;
    let base_value = parse_to_yaml_value(&config_content, config_format)?;

    if let Some((local_path, local_format)) = find_config_file("monk-local") {
        let local_content = fs::read_to_string(&local_path)?;
        let local_value = parse_to_yaml_value(&local_content, local_format)?;
        let merged = merge_top_level_yaml(base_value, local_value);
        return Ok(serde_yaml::from_value(merged)?);
    }

    Ok(serde_yaml::from_value(base_value)?)
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
    fn test_deserialize_glob_as_string() {
        let yaml = r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
      glob: "*.js"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let lint = hook.commands.get("lint").unwrap();
            assert_eq!(lint.glob, vec!["*.js"]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_deserialize_glob_as_list() {
        let yaml = r#"
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
      glob:
        - "*.js"
        - "*.ts"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let lint = hook.commands.get("lint").unwrap();
            assert_eq!(lint.glob, vec!["*.js", "*.ts"]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_deserialize_no_glob() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let fmt = hook.commands.get("fmt").unwrap();
            assert!(fmt.glob.is_empty());
            assert!(fmt.exclude.is_empty());
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_deserialize_glob_and_exclude_together() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: prettier --write {staged_files}
      glob: "*.{js,ts,css}"
      exclude: "*.min.js"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let fmt = hook.commands.get("fmt").unwrap();
            assert_eq!(fmt.glob, vec!["*.{js,ts,css}"]);
            assert_eq!(fmt.exclude, vec!["*.min.js"]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_legacy_commands_have_empty_glob() {
        let yaml = r#"
pre-commit:
  commands:
    - cargo fmt
    - cargo clippy
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            for (_name, command) in &hook.commands {
                assert!(command.glob.is_empty());
                assert!(command.exclude.is_empty());
            }
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_parallel_true() {
        let yaml = r#"
pre-commit:
  parallel: true
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.parallel);
            assert_eq!(hook.commands.len(), 2);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_parallel_defaults_to_false() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(!hook.parallel);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_parallel_with_path_based() {
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

        if let HookConfig::PathBased { paths } = config.hooks.get("pre-commit").unwrap() {
            let frontend = paths.get("frontend/").unwrap();
            assert!(frontend.parallel);

            let backend = paths.get("backend/").unwrap();
            assert!(!backend.parallel);
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_skip_single_string() {
        let yaml = r#"
pre-commit:
  skip: merge
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.skip, vec![SkipCondition::Merge]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_list() {
        let yaml = r#"
pre-commit:
  skip:
    - merge
    - rebase
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.skip, vec![SkipCondition::Merge, SkipCondition::Rebase]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_ref() {
        let yaml = r#"
pre-commit:
  skip:
    - ref: main
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.skip, vec![SkipCondition::Ref("main".to_string())]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_run_condition() {
        let yaml = r#"
pre-commit:
  skip:
    - run: test -n "$CI"
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(
                hook.skip,
                vec![SkipCondition::Run("test -n \"$CI\"".to_string())]
            );
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_mixed() {
        let yaml = r#"
pre-commit:
  skip:
    - merge
    - ref: "release/*"
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(
                hook.skip,
                vec![
                    SkipCondition::Merge,
                    SkipCondition::Ref("release/*".to_string())
                ]
            );
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_defaults_to_empty() {
        let yaml = r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.skip.is_empty());
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_skip_on_command() {
        let yaml = r#"
pre-commit:
  commands:
    deploy:
      run: ./deploy.sh
      skip:
        - ref: main
    test:
      run: cargo test
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let deploy = hook.commands.get("deploy").unwrap();
            assert_eq!(deploy.skip, vec![SkipCondition::Ref("main".to_string())]);

            let test = hook.commands.get("test").unwrap();
            assert!(test.skip.is_empty());
        } else {
            panic!("Expected Simple hook config");
        }
    }

    fn parse_config(yaml: &str) -> Config {
        serde_yaml::from_str(yaml).unwrap()
    }

    fn merge(base_yaml: &str, local_yaml: &str) -> Config {
        merge_yaml_configs(base_yaml, local_yaml).unwrap()
    }

    #[test]
    fn test_merge_adds_new_hook() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-push:
  commands:
    test:
      run: cargo test
"#,
        );
        assert_eq!(merged.hooks.len(), 2);
        assert!(merged.hooks.contains_key("pre-commit"));
        assert!(merged.hooks.contains_key("pre-push"));
    }

    #[test]
    fn test_merge_overrides_command() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    clippy:
      run: cargo clippy -- -D warnings
"#,
            r#"
pre-commit:
  commands:
    clippy:
      run: cargo clippy
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.get("clippy").unwrap().run, "cargo clippy");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_adds_new_command() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  commands:
    mycheck:
      run: ./check.sh
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.len(), 2);
            assert_eq!(hook.commands.get("fmt").unwrap().run, "cargo fmt");
            assert_eq!(hook.commands.get("mycheck").unwrap().run, "./check.sh");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_overrides_parallel() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  parallel: true
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert!(hook.parallel);
            assert_eq!(hook.commands.get("fmt").unwrap().run, "cargo fmt");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_overrides_working_directory() {
        let merged = merge(
            r#"
pre-commit:
  working_directory: frontend
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  working_directory: backend
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.working_directory, Some("backend".to_string()));
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_overrides_skip() {
        let merged = merge(
            r#"
pre-commit:
  skip:
    - merge
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  skip:
    - rebase
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.skip, vec![SkipCondition::Rebase]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_preserves_base_skip_when_local_empty() {
        let merged = merge(
            r#"
pre-commit:
  skip:
    - merge
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  commands:
    mycheck:
      run: ./check.sh
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.skip, vec![SkipCondition::Merge]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_preserves_base_commands() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
    clippy:
      run: cargo clippy
    test:
      run: cargo test
"#,
            r#"
pre-commit:
  commands:
    clippy:
      run: cargo clippy --all
"#,
        );
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.len(), 3);
            assert_eq!(hook.commands.get("fmt").unwrap().run, "cargo fmt");
            assert_eq!(
                hook.commands.get("clippy").unwrap().run,
                "cargo clippy --all"
            );
            assert_eq!(hook.commands.get("test").unwrap().run, "cargo test");
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_different_hook_variants_local_wins() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
            r#"
pre-commit:
  paths:
    "src/":
      commands:
        lint:
          run: cargo clippy
"#,
        );
        if let HookConfig::PathBased { paths } = merged.hooks.get("pre-commit").unwrap() {
            assert!(paths.contains_key("src/"));
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_merge_path_based_hooks() {
        let merged = merge(
            r#"
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: npm run lint
    "backend/":
      commands:
        fmt:
          run: cargo fmt
"#,
            r#"
pre-commit:
  paths:
    "frontend/":
      parallel: true
      commands:
        test:
          run: npm test
    "infra/":
      commands:
        validate:
          run: terraform validate
"#,
        );
        if let HookConfig::PathBased { paths } = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(paths.len(), 3);

            let frontend = paths.get("frontend/").unwrap();
            assert!(frontend.parallel);
            assert_eq!(frontend.commands.len(), 2);
            assert!(frontend.commands.contains_key("lint"));
            assert!(frontend.commands.contains_key("test"));

            assert!(paths.contains_key("backend/"));
            assert!(paths.contains_key("infra/"));
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_merge_empty_local() {
        let merged = merge(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
            "{}",
        );
        assert_eq!(merged.hooks.len(), 1);
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.get("fmt").unwrap().run, "cargo fmt");
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

    fn parse_toml(toml_str: &str) -> Config {
        parse_toml_config(toml_str).unwrap()
    }

    #[test]
    fn test_toml_simple_config() {
        let config = parse_toml(
            r#"
[pre-commit.commands.fmt]
run = "cargo fmt -- --check"

[pre-commit.commands.clippy]
run = "cargo clippy -- -D warnings"
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.len(), 2);
            assert_eq!(
                hook.commands.get("fmt").unwrap().run,
                "cargo fmt -- --check"
            );
            assert_eq!(
                hook.commands.get("clippy").unwrap().run,
                "cargo clippy -- -D warnings"
            );
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_toml_with_parallel_and_skip() {
        let config = parse_toml(
            r#"
[pre-commit]
parallel = true
skip = ["merge", "rebase"]

[pre-commit.commands.fmt]
run = "cargo fmt -- --check"
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.parallel);
            assert_eq!(hook.skip, vec![SkipCondition::Merge, SkipCondition::Rebase]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_toml_with_glob_and_exclude() {
        let config = parse_toml(
            r#"
[pre-commit.commands.lint]
run = "eslint {staged_files}"
glob = ["*.js", "*.ts"]
exclude = ["*.min.js"]
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            let lint = hook.commands.get("lint").unwrap();
            assert_eq!(lint.glob, vec!["*.js", "*.ts"]);
            assert_eq!(lint.exclude, vec!["*.min.js"]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_toml_with_working_directory() {
        let config = parse_toml(
            r#"
[pre-commit]
working_directory = "backend"

[pre-commit.commands.test]
run = "cargo test"
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.working_directory, Some("backend".to_string()));
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_toml_path_based() {
        let config = parse_toml(
            r#"
[pre-commit.paths."frontend/".commands.lint]
run = "npm run lint"

[pre-commit.paths."backend/".commands.fmt]
run = "cargo fmt -- --check"
"#,
        );
        if let HookConfig::PathBased { paths } = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(paths.len(), 2);
            assert_eq!(
                paths
                    .get("frontend/")
                    .unwrap()
                    .commands
                    .get("lint")
                    .unwrap()
                    .run,
                "npm run lint"
            );
            assert_eq!(
                paths
                    .get("backend/")
                    .unwrap()
                    .commands
                    .get("fmt")
                    .unwrap()
                    .run,
                "cargo fmt -- --check"
            );
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_toml_multiple_hooks() {
        let config = parse_toml(
            r#"
[pre-commit.commands.fmt]
run = "cargo fmt -- --check"

[pre-push.commands.test]
run = "cargo test"
"#,
        );
        assert_eq!(config.hooks.len(), 2);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks.contains_key("pre-push"));
    }

    #[test]
    fn test_toml_skip_ref_as_map() {
        let config = parse_toml(
            r#"
[pre-push.commands.deploy]
run = "./deploy.sh"
skip = [{ ref = "main" }]
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-push").unwrap() {
            let deploy = hook.commands.get("deploy").unwrap();
            assert_eq!(deploy.skip, vec![SkipCondition::Ref("main".to_string())]);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_piped_true() {
        let config = parse_config(
            r#"
pre-commit:
  piped: true
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.piped);
            assert!(!hook.follow);
            assert_eq!(hook.commands.len(), 2);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_piped_defaults_to_false() {
        let config = parse_config(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(!hook.piped);
            assert!(!hook.follow);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_piped_with_follow() {
        let config = parse_config(
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
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.piped);
            assert!(hook.follow);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_command_priority() {
        let config = parse_config(
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
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert_eq!(hook.commands.get("install").unwrap().priority, Some(1));
            assert_eq!(hook.commands.get("lint").unwrap().priority, Some(2));
            assert_eq!(hook.commands.get("test").unwrap().priority, None);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_piped_in_path_based() {
        let config = parse_config(
            r#"
pre-commit:
  paths:
    "frontend/":
      piped: true
      commands:
        lint:
          run: npm run lint
          priority: 1
        test:
          run: npm test
          priority: 2
    "backend/":
      commands:
        fmt:
          run: cargo fmt -- --check
"#,
        );
        if let HookConfig::PathBased { paths } = config.hooks.get("pre-commit").unwrap() {
            let frontend = paths.get("frontend/").unwrap();
            assert!(frontend.piped);

            let backend = paths.get("backend/").unwrap();
            assert!(!backend.piped);
        } else {
            panic!("Expected PathBased hook config");
        }
    }

    #[test]
    fn test_toml_piped_with_priority() {
        let config = parse_toml(
            r#"
[pre-commit]
piped = true
follow = true

[pre-commit.commands.install]
run = "npm install"
priority = 1

[pre-commit.commands.lint]
run = "eslint ."
priority = 2

[pre-commit.commands.test]
run = "npm test"
"#,
        );
        if let HookConfig::Simple(hook) = config.hooks.get("pre-commit").unwrap() {
            assert!(hook.piped);
            assert!(hook.follow);
            assert_eq!(hook.commands.get("install").unwrap().priority, Some(1));
            assert_eq!(hook.commands.get("lint").unwrap().priority, Some(2));
            assert_eq!(hook.commands.get("test").unwrap().priority, None);
        } else {
            panic!("Expected Simple hook config");
        }
    }

    #[test]
    fn test_merge_yaml_base_with_toml_local() {
        let merged = merge_toml_into_yaml(
            r#"
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
"#,
            r#"
[pre-commit]
parallel = true

[pre-commit.commands.clippy]
run = "cargo clippy"

[pre-commit.commands.mycheck]
run = "./check.sh"
"#,
        )
        .unwrap();
        if let HookConfig::Simple(hook) = merged.hooks.get("pre-commit").unwrap() {
            assert!(hook.parallel);
            assert_eq!(hook.commands.len(), 3);
            assert_eq!(
                hook.commands.get("fmt").unwrap().run,
                "cargo fmt -- --check"
            );
            assert_eq!(hook.commands.get("clippy").unwrap().run, "cargo clippy");
            assert_eq!(hook.commands.get("mycheck").unwrap().run, "./check.sh");
        } else {
            panic!("Expected Simple hook config");
        }
    }
}
