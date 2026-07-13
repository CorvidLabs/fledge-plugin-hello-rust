---
spec: hello-rust.spec.md
---

## Context

This repository complements the Bash reference with typed Rust messages and JSON serialization while remaining a small standalone plugin example.

## Related Modules

- Fledge plugin host and fledge-v1 protocol.
- `plugin.toml` command registration.

## Design Decisions

- Model outbound messages as a tagged enum so JSON types remain explicit.
- Hold locked standard streams for the process lifetime to keep message ordering deterministic.
