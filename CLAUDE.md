# Miden Project

This is a Miden smart contract project using the Rust SDK and compiler.

## Project Structure

- `contracts/` — Smart contracts (each is a separate crate, excluded from workspace)
  - Account components: `#[component]` macro
  - Note scripts: `#[note]` macro
  - Transaction scripts: `#[tx_script]` macro
- `integration/` — Integration tests and deployment scripts (workspace member)

## Build & Test

Contracts are built individually with cargo-miden (not `cargo build`):
```
cargo miden build --manifest-path contracts/<name>/Cargo.toml --release
```

Tests run via the workspace:
```
cargo test -p integration --release
```

Always build contracts before running tests — tests compile contracts via `build_project_in_dir()`.

## SDK Quick Reference

### Account Component
```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::{component, felt, Felt, StorageMap, StorageMapAccess, Word};

#[component]
struct MyComponent {
    #[storage(description = "my storage map")]
    my_map: StorageMap,
}

#[component]
impl MyComponent {
    pub fn get_value(&self) -> Felt { ... }
    pub fn set_value(&mut self, val: Felt) { ... }
}
```

### Note Script
```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::*;
use crate::bindings::miden::my_component::my_component;

#[note]
struct MyNote;
#[note]
impl MyNote {
    #[note_script]
    fn run(self, _arg: Word) { my_component::set_value(felt!(42)); }
}
```

### Cargo.toml Metadata
Account: `project-kind = "account"`, `supported-types = ["RegularAccountImmutableCode"]`
Note: `project-kind = "note-script"`, add `[package.metadata.miden.dependencies]` for cross-component calls
Tx script: `project-kind = "tx-script"`

All contracts require: `crate-type = ["cdylib"]`, `miden = { version = "0.9" }`

## Critical Pitfalls

**Felt arithmetic is modular (SECURITY CRITICAL)**: Subtraction wraps around the field modulus instead of panicking. ALWAYS validate before subtraction:
```rust
assert!(current.as_u64() >= amount.as_u64(), "Insufficient balance");
let result = current - amount;
```

**Felt comparisons are wrong for business logic**: `<`, `>`, `<=`, `>=` compare field elements, not natural numbers. ALWAYS convert first: `a.as_u64() < b.as_u64()`

**No-std required**: All contracts must use `#![no_std]` and `#![feature(alloc_error_handler)]`. For heap allocation, use `extern crate alloc;` and `BumpAlloc`.

## Skills

For detailed guidance, Claude will auto-load these skills when relevant:
- `rust-sdk-patterns` — Complete SDK macro, type, and API reference
- `miden-testing-patterns` — MockChain testing workflow and helpers
- `miden-concepts` — Miden architecture from a developer perspective
- `miden-pitfalls` — All known pitfalls with safe/unsafe code examples

## Advanced Development

For complex applications beyond basic patterns (multi-contract apps, novel note flows, custom asset handling):

1. Clone Miden source repos alongside this project (see `miden-source-guide` skill for repo list and clone commands)
2. Use Plan Mode first — Claude explores source repos to design the architecture before writing code
3. Claude uses sub-agents to explore repos efficiently without filling main context
4. The build hook provides verification — build, check errors, search source repos for correct pattern, fix, rebuild

The basic skills cover ~80% of patterns. Source repos provide the remaining 20% for advanced builders.

## Verification Workflow

After modifying contract code, always:
1. Build the contract: `cargo miden build --manifest-path contracts/<name>/Cargo.toml --release`
2. Run tests: `cargo test -p integration --release`
