---
spec: hello-rust.spec.md
---

## User Stories

- As a Rust plugin author, I want a typed reference for every fledge-v1 interaction.

## Acceptance Criteria

### REQ-hello-rust-001

The plugin SHALL deserialize the initialization message and keep protocol output separate from diagnostics.

Acceptance Criteria
- A valid initialization line is decoded before plugin work begins.
- Protocol JSON is written to standard output while diagnostics use standard error.

### REQ-hello-rust-002

The example SHALL serialize every supported outbound message variant using fledge-v1 JSON names.

Acceptance Criteria
- The release build compiles all typed outbound variants.
- Serialized message tags match the fledge-v1 snake-case names.

### REQ-hello-rust-003

Every request SHALL use a unique correlation identifier and process its response before continuing.

Acceptance Criteria
- Request identifiers advance monotonically within one process.
- Each request reads and validates its correlated host response.

### REQ-hello-rust-004

Cancellation, malformed input, and output failures SHALL terminate without a false successful completion.

Acceptance Criteria
- Malformed or missing protocol input returns a non-zero result.
- Cancellation and serialization or flush failures cannot report successful completion.

## Constraints

- This is a protocol example; Fledge owns rendering, storage, execution sandboxing, and metadata production.

## Out of Scope

- Defining a new protocol version or providing a reusable Rust client crate.
