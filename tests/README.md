# Tests

This directory contains integration tests for the monk Git hooks manager.

## Test Organization

- `simple_config.rs` - Tests for simple/basic hook configuration format
- `path_based_config.rs` - Tests for path-based hook configuration and path matching logic
- `rust_submodules.rs` - Tests for multiple Rust submodule configurations

## Running Tests

To run all tests:
```bash
cargo test
```

To run a specific test file:
```bash
cargo test --test simple_config
cargo test --test path_based_config  
cargo test --test rust_submodules
```
