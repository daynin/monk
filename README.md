<p align="center">
  <a href="https://github.com/daynin/monk">
    <img src="./logo.svg" height="200px"/>
  </a>
</p>

<h2 align="center">
    Monk is a simple Git hooks manager
</h2>

<p align="center">
  <a href="https://www.bestpractices.dev/en/projects/10442">
    <img alt="OpenSSF Best Practices" src="https://www.bestpractices.dev/projects/6505/badge">
  </a>
  <a href="https://github.com/daynin/monk/blob/master/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg">
  </a>
  <a href="https://github.com/daynin/monk/issues">
    <img alt="GitHub Issues" src="https://img.shields.io/github/issues/daynin/monk.svg">
  </a>
  <a href="https://crates.io/crates/monk">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/monk.svg">
  </a>
  <a href="https://crates.io/crates/monk">
    <img alt="Downloads" src="https://img.shields.io/crates/d/monk">
  </a>
</p>

### Monk's features:

- 🦀 **Easily set up in your Rust project.** No need to install additional package managers.
- ⚙️ **Works with custom `build.rs` files.** Automate the hooks installation process.
- 💻 **Run your hooks via CLI.** Test your hooks without triggering them via Git.

> Keep calm, monk will protect your repo!

### Installation

You can install it using `cargo`:

```sh
cargo install monk
```

#### Or

You can add it as a build dependency:

```sh
cargo add --build monk
```

Then create a `build.rs` file:

```rust
pub fn main() {
    monk::init();
}
```

In this case, `monk` will be installed automatically and will initialize all hooks from `monk.yaml`
.
This is the most convenient option for Rust projects, as it doesn't require contributors to install `monk` manually.

#### Installing monk with Nix

You can also install `monk` using Nix:

```sh
nix profile install github:daynin/monk
```

#### Installing monk with Guix

You can install `monk` using GNU Guix directly from GitHub:

```sh
# Install latest version from main branch
guix package -f <(curl -s https://raw.githubusercontent.com/daynin/monk/main/monk.scm)
```

Note: This will automatically fetch and build the latest version from the main branch.


### Usage

Create a configuration file named `monk.yaml` (or `monk.toml`) in your project root:

```yaml
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
```

Then install the hooks:

```sh
monk install
```

If you added monk as a build dependency with `build.rs` (see above), hooks are installed automatically when you build your project.

---

### Documentation

#### Named Commands

Each command has a name and a `run` field:

```yaml
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
    test:
      run: cargo test
```

Commands run in the order they are defined. If any command fails, execution stops and the hook fails.

<details>
<summary>Legacy format (backward compatible)</summary>

Plain string arrays still work:

```yaml
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
```

Commands are auto-named `cmd1`, `cmd2`, etc. The named format is recommended for new configs.

</details>

#### File Placeholders

Use placeholders to pass file lists to your tools:

| Placeholder | Expands to |
|---|---|
| `{staged_files}` | Files staged for commit (`git diff --cached`) |
| `{push_files}` | Files changed between local and remote |
| `{all_files}` | All tracked files in the repository |

```yaml
pre-commit:
  commands:
    lint:
      run: eslint {staged_files}
    fmt:
      run: prettier --write {staged_files}

pre-push:
  commands:
    test:
      run: cargo test {push_files}
```

When a placeholder expands to an empty file list, the command is automatically skipped. If the expanded command exceeds the OS argument length limit, it is automatically split into batches.

#### Glob Filtering

Use `glob` and `exclude` to filter which files a command applies to:

```yaml
pre-commit:
  commands:
    lint-js:
      run: eslint {staged_files}
      glob: "*.{js,ts}"
      exclude: "*.min.js"
    lint-rs:
      run: cargo clippy
      glob: "*.rs"
    fmt:
      run: prettier --write {staged_files}
      glob:
        - "*.js"
        - "*.ts"
        - "*.css"
```

Both `glob` and `exclude` accept a single pattern or a list of patterns. Patterns without a `/` match files in any directory (e.g., `*.rs` matches `src/main.rs`).

When `glob` is set but no files match, the command is skipped. When `glob` is used with a placeholder like `{staged_files}`, only matching files are passed to the command.

When `glob` is set without a file placeholder, monk checks staged files against the pattern and skips the command if none match.

#### Path-Based Configuration

For monorepos with multiple modules or mixed technologies:

```yaml
pre-commit:
  paths:
    "frontend/":
      commands:
        lint:
          run: npm run lint
        test:
          run: npm test
      working_directory: frontend
    "backend/":
      commands:
        fmt:
          run: cargo fmt -- --check
        clippy:
          run: cargo clippy -- -D warnings
      working_directory: backend
```

When using `monk run --changed-only` (the default for installed hooks), only hooks whose path prefix matches the changed files will run.

#### Working Directory

Set `working_directory` at the hook level or the command level. Command-level overrides hook-level:

```yaml
pre-commit:
  commands:
    frontend-lint:
      run: npm run lint
      working_directory: frontend
    backend-test:
      run: cargo test
      working_directory: backend
```

Or at the hook level for all commands:

```yaml
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
    test:
      run: cargo test
  working_directory: backend
```

#### Parallel Execution

Run all commands in a hook concurrently with `parallel: true`:

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      run: cargo fmt -- --check
    clippy:
      run: cargo clippy -- -D warnings
    test:
      run: cargo test
```

All commands run simultaneously and their output is buffered. A summary with pass/fail status and timing is printed after all commands finish. If any command fails, the hook fails.

#### Skip Conditions

Skip hooks or individual commands based on git state, branch, or shell conditions:

```yaml
pre-commit:
  skip:
    - merge
    - rebase
  commands:
    fmt:
      run: cargo fmt -- --check
    slow-test:
      run: cargo test --all
      skip:
        - run: "test -n \"$CI\""

pre-push:
  commands:
    deploy:
      run: ./deploy.sh
      skip:
        - ref: main
    test:
      run: cargo test
      skip:
        - ref: "release/*"
```

| Condition | Skips when |
|---|---|
| `merge` | A merge is in progress (`.git/MERGE_HEAD` exists) |
| `rebase` | A rebase is in progress |
| `ref: <pattern>` | Current branch matches the pattern (supports globs like `release/*`) |
| `run: <command>` | Shell command exits with code 0 |

Skip accepts a single condition or a list. If any condition matches, the hook or command is skipped.

To disable all hooks globally, set the environment variable `MONK=0`.

#### Local Config Overrides

Create a `monk-local.yaml` file (add it to `.gitignore`) to override or extend your project's `monk.yaml` without affecting teammates:

```yaml
pre-commit:
  parallel: true
  commands:
    clippy:
      run: cargo clippy
    mycheck:
      run: ./my-local-check.sh

pre-push:
  skip:
    - ref: main
```

Merge rules:
- **Hooks**: merged by name. New hooks are added, existing hooks are deep-merged.
- **Commands**: merged by name. A local command with the same name fully replaces the base command. New commands are added.
- **Scalar fields** (`parallel`, `working_directory`): local value overrides base.
- **Skip conditions**: local replaces base (not concatenated).
- **Different hook variants** (Simple vs PathBased): local replaces base entirely.

If `monk-local.yaml` does not exist, `monk.yaml` is used as-is. Local configs also support TOML format (`monk-local.toml`). Main and local configs can use different formats.

#### TOML Configuration

Monk supports TOML as an alternative to YAML. Create a `monk.toml` file instead of `monk.yaml`:

```toml
[pre-commit]
parallel = true

[pre-commit.commands.fmt]
run = "cargo fmt -- --check"
glob = ["*.rs"]

[pre-commit.commands.clippy]
run = "cargo clippy -- -D warnings"

[pre-push.commands.test]
run = "cargo test"
skip = ["merge"]
```

Monk searches for config files in this order: `monk.yaml`, `monk.toml`. The first one found is used. The same applies to local overrides: `monk-local.yaml`, `monk-local.toml`.

#### CLI

```sh
monk install       # Install hooks defined in monk.yaml
monk run <hook>    # Run a hook manually (e.g., monk run pre-commit)
monk uninstall     # Remove hooks and restore backups
```

`monk` automatically backs up existing hooks before installing. Running `monk uninstall` restores the original hooks.
