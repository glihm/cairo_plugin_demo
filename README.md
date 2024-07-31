# Cairo plugin with Scarb

This repo is mainly for debugging and experimentation.

## Architecture

- `contracts` contains a minimal Cairo project that uses the plugin.
- `plugin` contains the source code for the demo plugin and compiler to extends Cairo and Scarb.
- `bins` contains the binaries for the demo with a `compiler` and `ls` (Language Server).

## Setup

1. Install Rust.
2. `cargo build -r --workspace` or `cargo run -r --bin compiler -- --manifest-path contracts/Scarb.toml`
