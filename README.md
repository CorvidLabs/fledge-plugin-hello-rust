# fledge-plugin-hello-rust

**Reference implementation** of a fledge plugin written in Rust, demonstrating the **fledge-v1** protocol. Use this project as a starting point when building your own Rust-based fledge plugins.

This plugin exercises every message type in a single interactive walkthrough, serving as both a working example and a conformance test for the protocol.

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
fledge plugins install ./path/to/fledge-plugin-hello-rust
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

---

## The fledge-v1 Protocol for Rust Plugin Authors

This section documents everything you need to know to write a fledge plugin in Rust.

### Transport

The protocol uses **newline-delimited JSON (NDJSON)** over stdio:

- **stdin** -- receives messages from fledge (init, response, cancel)
- **stdout** -- sends messages to fledge (prompt, log, output, etc.)
- **stderr** -- goes directly to the terminal (debug output, never captured)

Each message is a single JSON object terminated by `\n`. Messages must not span multiple lines.

### Lifecycle

1. Fledge spawns the plugin binary and sends an **`init`** message on stdin.
2. The plugin sends outbound messages on stdout (fire-and-forget or requests).
3. For request messages (`prompt`, `confirm`, `select`, `multi_select`, `load`, `exec`, `metadata`), the plugin reads a **`response`** message from stdin before continuing.
4. At any time, fledge may send a **`cancel`** message (e.g., user pressed Ctrl+C). The plugin should exit promptly.
5. The plugin exits with code **0** on success, non-zero on failure.

### Init message (fledge -> plugin)

Sent once at startup. Shape:

```json
{
  "type": "init",
  "protocol": "fledge-v1",
  "args": ["--flag", "value"],
  "project": {
    "name": "my-project",
    "root": "/path/to/project",
    "language": "rust",
    "git": { "branch": "main", "dirty": false, "remote": "origin", "remote_url": "..." }
  },
  "plugin": { "name": "fledge-hello-rust", "version": "0.1.0", "dir": "/path/to/plugin" },
  "fledge": { "version": "0.9.0" },
  "capabilities": { "exec": true, "store": true, "metadata": true }
}
```

The `project` field is `null` if fledge cannot detect a project context. The `capabilities` object tells the plugin which privileged operations are permitted by the user's trust configuration.

### Response message (fledge -> plugin)

Sent after a request message. Shape:

```json
{ "type": "response", "id": "1", "value": "user input here" }
```

The `id` matches the request that triggered it. The `value` type depends on the request (string for prompt, bool for confirm, string for select, array of strings for multi_select, object for exec/metadata, string or null for load).

### Cancel message (fledge -> plugin)

Sent when the user cancels. Shape:

```json
{ "type": "cancel", "reason": "user_interrupt" }
```

The plugin should exit with a non-zero code.

### Outbound message types (plugin -> fledge)

All outbound messages have a `"type"` field. Request messages additionally require an `"id"` field (unique per request, used to correlate the response).

#### Fire-and-forget messages

| Type | Fields | Description |
|------|--------|-------------|
| `log` | `level` (trace/debug/info/warn/error), `message` | Structured log line, rendered with color |
| `output` | `text` | Raw text printed directly to the terminal |
| `progress` | `message?`, `current?`, `total?`, `done?` | Progress bar or spinner (see below) |
| `store` | `key`, `value` | Persist a string value under a key |

#### Request messages (require reading a response)

| Type | Fields | Response value |
|------|--------|----------------|
| `prompt` | `id`, `message`, `default?`, `validate?` | `string` |
| `confirm` | `id`, `message`, `default?` | `bool` |
| `select` | `id`, `message`, `options`, `default?` | `string` (the selected option) |
| `multi_select` | `id`, `message`, `options`, `defaults?` | `[string]` (selected options) |
| `load` | `id`, `key` | `string` or `null` |
| `exec` | `id`, `command`, `cwd?`, `timeout?` | `{ "code": int, "stdout": string, "stderr": string }` |
| `metadata` | `id`, `keys` | `{ key: value, ... }` |

#### Progress behavior

- **Determinate**: set `current` and `total` to show a progress bar.
- **Indeterminate (spinner)**: omit `current` and `total`, set only `message`.
- **Done**: send `{ "type": "progress", "done": true }` to clear the bar/spinner.

### Validation strings

The `validate` field on `prompt` accepts these values:

- `"non_empty"` -- reject blank input

### Plugin manifest (`plugin.toml`)

Every fledge plugin needs a `plugin.toml` at its root:

```toml
[plugin]
name = "fledge-hello-rust"
version = "0.1.0"
description = "Example plugin demonstrating the fledge-v1 protocol in Rust"
author = "CorvidLabs"
protocol = "fledge-v1"

[[commands]]
name = "hello-rust"
description = "Interactive hello-world using all protocol messages (Rust)"
binary = "target/release/fledge-hello-rust"
```

The `[[commands]]` table maps subcommand names to binaries. A plugin can expose multiple commands.

### Key Rust implementation patterns

This reference implementation demonstrates the recommended patterns:

1. **Tagged enum for outbound messages** -- Use `#[serde(tag = "type", rename_all = "snake_case")]` on an enum so each variant serializes with the correct `type` field.

2. **IO wrapper struct** -- A `PluginIO` struct holding locked stdin/stdout handles provides `send()`, `recv_response()`, and `request()` (send + recv) methods.

3. **Message ID generation** -- Use an `AtomicU64` counter for unique request IDs.

4. **Cancel handling** -- Check for `"cancel"` type in every response read; exit immediately if received.

5. **Stderr for debugging** -- Use `eprintln!` freely for debug output since stderr is never captured by the protocol.

6. **Minimal dependencies** -- Only `serde` and `serde_json` are needed. No async runtime required.

### Writing your own plugin

1. Copy this repository as a starting point.
2. Update `plugin.toml` with your plugin's name and commands.
3. Update `Cargo.toml` to match.
4. Replace the body of `run()` in `src/main.rs` with your logic.
5. Keep the `PluginIO` struct and message types -- they are the protocol layer.
6. Build with `cargo build --release` and install with `fledge plugins install .`

### Repository naming

Plugin repositories must be named `fledge-plugin-{name}` where `{name}` is a single word (the command name).

## License

MIT
