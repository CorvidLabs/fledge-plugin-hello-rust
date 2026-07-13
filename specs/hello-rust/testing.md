---
spec: hello-rust.spec.md
---

## Test Plan

### Integration Tests

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build --release`
- Validate the fledge-v1 manifest and release binary path.
