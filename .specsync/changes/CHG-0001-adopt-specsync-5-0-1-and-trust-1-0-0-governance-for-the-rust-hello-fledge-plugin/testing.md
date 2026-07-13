---
change: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-rust-hello-fledge-plugin
artifact: testing
---

# Testing

Local acceptance requires the Fledge verification lane, strict 100% coverage, four installed integrations, a healthy Trust doctor, and a clean diff.

## Requirement Evidence

- `REQ-hello-rust-001`: `cargo build --release` type-checks initialization and separated stream handling; `cargo clippy -- -D warnings` checks the protocol path.
- `REQ-hello-rust-002`: `cargo build --release` compiles every typed outbound variant and Serde tag.
- `REQ-hello-rust-003`: `cargo build --release` and `cargo clippy -- -D warnings` validate the correlation loop and response handling.
- `REQ-hello-rust-004`: `cargo build --release` and `cargo clippy -- -D warnings` validate the explicit error and cancellation paths.

Hosted acceptance requires the new `trust` job and existing Rust CI to pass on Ubuntu.
