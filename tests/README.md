# Tests

This directory contains integration tests for the monk Git hooks manager.

## Test Organization

- `simple_config.rs` - Tests for simple/basic hook configuration format
- `path_based_config.rs` - Tests for path-based hook configuration and path matching logic
- `manual_execution.rs` - Tests for config reading, hook finding, and working directory handling
- `rust_submodules.rs` - Tests for multiple Rust submodule configurations
- `named_commands.rs` - Tests for named commands format, backward compatibility, and ordering
- `staged_files.rs` - Tests for `{staged_files}`, `{push_files}`, `{all_files}` placeholder expansion
- `glob_filtering.rs` - Tests for `glob:` and `exclude:` file filtering on commands
- `parallel_execution.rs` - Tests for `parallel: true` hook-level parallel execution
- `skip_conditions.rs` - Tests for `skip:` conditions on hooks and commands (merge, rebase, ref, run)
- `local_config.rs` - Tests for `monk-local.yaml` config overrides and deep-merge behavior
- `toml_config.rs` - Tests for `monk.toml` TOML configuration format and cross-format merging
- `piped_execution.rs` - Tests for `piped: true` sequential priority-ordered execution with `follow:` and `priority:`
- `env_rc.rs` - Tests for `env:` command-level environment variables and top-level `rc:` shell initialization

## Running Tests

To run all tests:
```bash
cargo test
```

To run a specific test file:
```bash
cargo test --test simple_config
cargo test --test path_based_config
cargo test --test manual_execution
cargo test --test rust_submodules
cargo test --test named_commands
cargo test --test staged_files
cargo test --test glob_filtering
cargo test --test parallel_execution
cargo test --test skip_conditions
cargo test --test local_config
cargo test --test toml_config
cargo test --test piped_execution
cargo test --test env_rc
```
