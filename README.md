# Cairo plugin with Scarb

This repo is mainly for debugging and experimentation.

## Architecture

- `demo_code` contains a minimal Cairo project that uses the plugin.
- `src` contains the source code for the plugin integrated to Scarb.

## Setup

1. Install Rust.
2. `cargo run`

## Issue related to Dojo

The main goal of this repo is to debug the issue related to Dojo where the diagnostic pointer is incorrect.
When the plugin rewrites a node of a supported custom attributes, everything works fine **if no children node is re-writter**.

If a child node is re-written, the diagnostic pointer is incorrect.

- `lib,cairo` -> A simple custom contract, that expands to a Starknet contract.
- `plugin.rs` -> The plugin attempts to rewrite the module with the `#[custom::contract]` attribute, replacing the original one. But during this process, if the `impl` node (children of the module) is re-written, the diagnostic pointer is incorrect.

So if you uncomment the `return 1_u256` into the `lib.cairo` only by copying the children nodes of the module into `plugin.rs`, the diagnostic pointer will be correct.

However, if you uncomment some code into `plugin.rs` that re-write a children node, the diagnostic pointer will be incorrect.
