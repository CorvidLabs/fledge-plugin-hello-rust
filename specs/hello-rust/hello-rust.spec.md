---
module: hello-rust
version: 2
status: active
files:
  - src/main.rs

db_tables: []
depends_on: []
---

# Hello-rust

## Purpose

Provide the Rust reference implementation of the `fledge-v1` JSON-lines plugin protocol, demonstrating every supported interactive and one-way message type.

## Public API

| Surface | Behavior |
|---------|----------|
| hello-rust | Consume a fledge-v1 initialization message and demonstrate all protocol message variants. |

## Invariants

1. Standard input and output contain exactly one JSON object per protocol line.
2. Diagnostics use standard error and never contaminate protocol output.
3. Request identifiers increase atomically and correlate host responses.
4. Cancellation terminates non-zero rather than continuing with partial state.
5. Outbound messages serialize with the fledge-v1 snake-case type names.
6. The manifest declares the fledge-v1 protocol and release binary.

## Behavioral Examples

```
Given a valid initialization message and host responses
When Fledge launches the Rust example
Then the plugin demonstrates log, output, prompts, selections, progress, storage, execution, and metadata through JSON lines
```

## Error Cases

| Error | When | Behavior |
|-------|------|----------|
| Missing init | Standard input closes before initialization | Fail rather than fabricate project context. |
| Invalid JSON | Init or response cannot be deserialized | Fail with a protocol parsing diagnostic. |
| Cancel response | Host returns a cancellation message | Report cancellation and exit non-zero. |
| Output failure | Serialization, write, or flush fails | Fail immediately rather than emit a partial message. |

## Dependencies

- Rust 2021
- `serde` and `serde_json`
- Fledge host implementing `fledge-v1`

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07-12 | Document the existing Rust fledge-v1 reference behavior for SpecSync 5 adoption. |
| 2026-07-13 | CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-rust-hello-fledge-plugin: Adopt SpecSync 5.0.1 and Trust 1.0.0 governance for the Rust Hello Fledge plugin |
