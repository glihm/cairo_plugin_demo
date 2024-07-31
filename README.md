# Cairo plugin with Scarb

This repo is mainly for debugging and experimentation.

## Architecture

- `bins` contains the binaries for the demo with a `compiler` that loads the plugin in memory and `ls` (Language Server).
- `contracts` contains a minimal Cairo project that uses the plugin.
- `plugin` contains the source code for the demo plugin and compiler to extends Cairo and Scarb. The very simple plugin just rewrites implementations found into a module with a `#[custom::contract]` attribute. `self` is automatically added by the plugin, or you can precise `r: R` to inject `ref self: ContractState` instead. If the implementation is named `bad`, the plugin will emit a diagnostic.

## Setup

1. Install Rust.
2. `cargo build -r --workspace` or `cargo run -r --bin compiler -- --manifest-path contracts/Scarb.toml`

## Test on VSCode

1. Build the language server with `cargo build -r --bin demo-ls`.
2. Install the extension built for `2.7.0-rc-3` present in the repositoy (`cairo1-2.7.0-rc.3.vsix`). In VSCode you have in the extension panel a button to `Install from VSIX`.
3. Adjust the paths inside `settings.json` and use this file setting into VSCode.
4. Open the folder `contracts`.
5. Open the file `lib.cairo`.
6. Make some changes inside the function and the diagnostics should appear inside the `PROBLEMS` tag, but not inside the editor panel.
