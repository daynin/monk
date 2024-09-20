<p align="center">
  <a href="https://github.com/daynin/monk">
    <img src="./logo.svg" height="200px"/>
  </a>
</p>

<h2 align="center">
    Monk is a simple Git hooks manager
</h2>

<p align="center">
  <a href="https://travis-ci.org/daynin/monk">
  <a href="https://github.com/daynin/monk/blob/master/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg">
  </a>
  <a href="https://github.com/daynin/monk/issues">
    <img alt="GitHub Issues" src="https://img.shields.io/github/issues/daynin/monk.svg">
  </a>
  <a href="https://crates.io/crates/monk">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/monk.svg">
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


### Usage

Create a configuration file `monk.yaml` in your project:

```yaml
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings

pre-push:
  commands:
    - cargo test

```


Then, install the hooks manually:

```sh
monk install
```

#### Or 

Install `monk` as a build dependency **(this is the preferred way)**:

```sh
cargo add --build monk
```

and create a build script `build.rs`:

```rust
pub fn main() {
    monk::init();
}
```

This way, `monk` will **automatically install hooks** for every team member during the build process.

#### Running hooks hooks manually
If you want to run specific hooks, use the `run` command:

```sh
monk run pre-commit
```


