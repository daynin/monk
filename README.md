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


### Usage

Create a configuration file named `monk.yaml` in your project root:

```yaml
pre-commit:
  commands:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings

pre-push:
  commands:
    - cargo test

```


If you installed `monk` manually, run:

```sh
monk install
```

If you added it as a build dependency and set up `build.rs` as shown above, the hooks will be installed automatically when you build your project.

#### Running hooks manually

To run specific hooks manually, use the `run` command

```sh
monk run pre-commit
```

#### Removing Hooks

`monk` automatically creates backup files for existing hooks and restores them when you remove monk's hooks.

To remove the hooks, run:

```sh  
monk uninstall
```
