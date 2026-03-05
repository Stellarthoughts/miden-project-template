# AGENTS.md

## Repo layout
- `contracts/`: Smart contracts (each is a separate crate, excluded from workspace)
  - Account components (`#[component]`), note scripts (`#[note]`), transaction scripts (`#[tx_script]`)
- `integration/`: Integration tests and deployment scripts (workspace member)
  - `src/bin/`: Rust binaries for on-chain interactions
  - `tests/`: Integration tests
- `.claude/skills/`: AI skill files for Miden SDK patterns, pitfalls, testing, and source exploration

## Building contracts
Contracts use `cargo-miden`, not `cargo build`:
```
cargo miden build --manifest-path contracts/<name>/Cargo.toml --release
```

Build contracts individually. They are not workspace members.

## Running tests
```
cargo test -p integration --release
```

Always build contracts before running tests.

## Adding a new contract
```
miden cargo-miden new --account contracts/my-account
```

## Key constraints
- All contract code must be `#![no_std]` with `#![feature(alloc_error_handler)]`
- Felt arithmetic is modular (subtraction wraps instead of panicking)
- Standard components (BasicWallet, BasicFungibleFaucet) are MASM-only, not callable from Rust
