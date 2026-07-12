---
spec: hello-rust.spec.md
---

## User Stories

- As a Rust plugin author, I want a typed reference for every fledge-v1 interaction.

## Acceptance Criteria

### REQ-hello-rust-001

The plugin SHALL deserialize the initialization message and keep protocol output separate from diagnostics.

### REQ-hello-rust-002

The example SHALL serialize every supported outbound message variant using fledge-v1 JSON names.

### REQ-hello-rust-003

Every request SHALL use a unique correlation identifier and process its response before continuing.

### REQ-hello-rust-004

Cancellation, malformed input, and output failures SHALL terminate without a false successful completion.

## Constraints

- This is a protocol example; Fledge owns rendering, storage, execution sandboxing, and metadata production.

## Out of Scope

- Defining a new protocol version or providing a reusable Rust client crate.
