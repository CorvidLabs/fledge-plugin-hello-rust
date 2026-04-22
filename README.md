# fledge-hello-rust

Example fledge plugin written in Rust, demonstrating the **fledge-v1** protocol. Exercises every message type in a single interactive walkthrough.

## Building

```bash
cargo build --release
```

Or with fledge:

```bash
fledge run build
```

## Installing

```bash
fledge plugin install ./path/to/fledge-plugin-hello-rust
fledge hello-rust
```

## Message types demonstrated

| # | Type | Direction | What happens |
|---|------|-----------|-------------|
| 1 | `log` | plugin -> fledge | Colored structured logging |
| 2 | `output` | plugin -> fledge | Raw text passthrough |
| 3 | `prompt` | plugin -> fledge -> plugin | Ask for text input with validation |
| 4 | `confirm` | plugin -> fledge -> plugin | Yes/no dialog |
| 5 | `select` | plugin -> fledge -> plugin | Pick one from a list |
| 6 | `multi_select` | plugin -> fledge -> plugin | Pick multiple from a list |
| 7 | `progress` | plugin -> fledge | Determinate progress bar |
| 8 | `store` / `load` | plugin -> fledge | Key-value persistence roundtrip |
| 9 | `exec` | plugin -> fledge -> plugin | Sandboxed shell command |
| 10 | `metadata` | plugin -> fledge -> plugin | Project context query |
| 11 | `progress` (spinner) | plugin -> fledge | Indeterminate spinner |

## Protocol overview

The fledge-v1 protocol uses newline-delimited JSON (NDJSON) over stdio:

- **stdin** receives messages from fledge (init, response, cancel)
- **stdout** sends messages to fledge (prompt, log, output, etc.)
- **stderr** goes directly to the terminal (debug output)

The plugin lifecycle:

1. Fledge sends an `init` message with project context
2. Plugin sends outbound messages (requests or fire-and-forget)
3. For requests (`prompt`, `confirm`, `select`, `load`, `exec`, `metadata`), read the response from stdin
4. Exit with code 0 on success

## Writing your own Rust plugin

Copy this project as a starting point. The key patterns:

- Define an `OutboundMessage` enum with `#[serde(tag = "type")]` for clean serialization
- Use a simple `PluginIO` struct to wrap stdin/stdout with JSON line reading/writing
- Handle `cancel` messages in the response reader so Ctrl+C works
- Use `eprintln!` for debug output (stderr is never captured)
