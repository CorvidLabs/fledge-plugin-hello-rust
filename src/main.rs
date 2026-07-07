//! fledge-hello-rust: example plugin exercising every fledge-v1 protocol message.
//!
//! Communicates via JSON-lines on stdin (fledge -> plugin) and stdout (plugin -> fledge).
//! Stderr goes directly to the terminal for debug output.

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// Message ID generator
// ---------------------------------------------------------------------------

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> String {
    NEXT_ID.fetch_add(1, Ordering::Relaxed).to_string()
}

// ---------------------------------------------------------------------------
// Inbound messages (fledge -> plugin)
// ---------------------------------------------------------------------------

/// The `init` message sent by fledge on startup.
#[derive(Debug, Deserialize)]
struct InitMessage {
    #[allow(dead_code)]
    protocol: String,
    args: Vec<String>,
    project: Option<ProjectInfo>,
    plugin: PluginInfo,
    #[allow(dead_code)]
    fledge: FledgeInfo,
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    name: String,
    #[allow(dead_code)]
    root: String,
    #[allow(dead_code)]
    language: Option<String>,
    #[allow(dead_code)]
    git: Option<GitInfo>,
}

#[derive(Debug, Deserialize)]
struct GitInfo {
    #[allow(dead_code)]
    branch: String,
    #[allow(dead_code)]
    dirty: bool,
}

#[derive(Debug, Deserialize)]
struct PluginInfo {
    name: String,
    version: String,
    #[allow(dead_code)]
    dir: String,
}

#[derive(Debug, Deserialize)]
struct FledgeInfo {
    #[allow(dead_code)]
    version: String,
}

/// A generic inbound envelope -- we only need `type`, `id`, and `value`.
#[derive(Debug, Deserialize)]
struct InboundMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[allow(dead_code)]
    id: Option<String>,
    value: Option<serde_json::Value>,
    #[allow(dead_code)]
    reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Outbound messages (plugin -> fledge)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OutboundMessage {
    Prompt {
        id: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        validate: Option<String>,
    },
    Confirm {
        id: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<bool>,
    },
    Select {
        id: String,
        message: String,
        options: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<usize>,
    },
    MultiSelect {
        id: String,
        message: String,
        options: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        defaults: Option<Vec<usize>>,
    },
    Progress {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        current: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        done: Option<bool>,
    },
    Log {
        level: String,
        message: String,
    },
    Output {
        text: String,
    },
    Store {
        key: String,
        value: String,
    },
    Load {
        id: String,
        key: String,
    },
    Exec {
        id: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
    },
    Metadata {
        id: String,
        keys: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// IO helpers
// ---------------------------------------------------------------------------

struct PluginIO {
    stdin: io::StdinLock<'static>,
    stdout: io::StdoutLock<'static>,
}

impl PluginIO {
    fn new() -> Self {
        // Leak the handles so we can hold locks for the process lifetime.
        let stdin = Box::leak(Box::new(io::stdin())).lock();
        let stdout = Box::leak(Box::new(io::stdout())).lock();
        Self { stdin, stdout }
    }

    /// Read one JSON line from stdin.
    fn recv_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.stdin.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => Some(line),
            Err(e) => {
                eprintln!("stdin read error: {e}");
                None
            }
        }
    }

    /// Read and parse the init message.
    fn recv_init(&mut self) -> InitMessage {
        let line = self.recv_line().expect("expected init message on stdin");
        serde_json::from_str(&line).expect("failed to parse init message")
    }

    /// Read and parse a response message.
    fn recv_response(&mut self) -> InboundMessage {
        let line = self.recv_line().expect("expected response on stdin");
        let msg: InboundMessage =
            serde_json::from_str(&line).expect("failed to parse inbound message");

        if msg.msg_type == "cancel" {
            eprintln!("received cancel: {:?}", msg.reason);
            std::process::exit(1);
        }

        msg
    }

    /// Send a message to fledge (one JSON line to stdout).
    fn send(&mut self, msg: &OutboundMessage) {
        serde_json::to_writer(&mut self.stdout, msg).expect("failed to serialize message");
        writeln!(self.stdout).expect("failed to write newline");
        self.stdout.flush().expect("failed to flush stdout");
    }

    /// Send a request and wait for the response.
    fn request(&mut self, msg: &OutboundMessage) -> InboundMessage {
        self.send(msg);
        self.recv_response()
    }
}

// ---------------------------------------------------------------------------
// Plugin logic
// ---------------------------------------------------------------------------

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut io = PluginIO::new();

    // -- Step 0: Read init --------------------------------------------------
    let init = io.recv_init();
    eprintln!(
        "init received: plugin={} v{}",
        init.plugin.name, init.plugin.version
    );

    let project_name = init
        .project
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("unknown");

    // -- Step 1: Log --------------------------------------------------------
    io.send(&OutboundMessage::Log {
        level: "info".into(),
        message: format!("fledge-hello-rust started (project: {project_name})"),
    });

    // -- Step 2: Output -----------------------------------------------------
    io.send(&OutboundMessage::Output {
        text: "\n  Welcome to fledge-hello-rust!\n  \
               This plugin demonstrates every fledge-v1 protocol message.\n\n"
            .into(),
    });

    if !init.args.is_empty() {
        io.send(&OutboundMessage::Log {
            level: "debug".into(),
            message: format!("args: {:?}", init.args),
        });
    }

    // -- Step 3: Prompt -----------------------------------------------------
    let resp = io.request(&OutboundMessage::Prompt {
        id: next_id(),
        message: "What is your name?".into(),
        default: Some("world".into()),
        validate: Some("non_empty".into()),
    });
    let name = resp
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("world")
        .to_string();

    io.send(&OutboundMessage::Output {
        text: format!("  Hello, {name}!\n\n"),
    });

    // -- Step 4: Confirm ----------------------------------------------------
    let resp = io.request(&OutboundMessage::Confirm {
        id: next_id(),
        message: "Run the full demo?".into(),
        default: Some(true),
    });
    let confirmed = resp
        .value
        .as_ref()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !confirmed {
        io.send(&OutboundMessage::Output {
            text: "  Okay, exiting early. Bye!\n".into(),
        });
        return Ok(());
    }

    // -- Step 5: Select -----------------------------------------------------
    let resp = io.request(&OutboundMessage::Select {
        id: next_id(),
        message: "Pick a color:".into(),
        options: vec!["red".into(), "green".into(), "blue".into()],
        default: Some(1),
    });
    let color = resp
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("green")
        .to_string();

    io.send(&OutboundMessage::Log {
        level: "info".into(),
        message: format!("You picked: {color}"),
    });

    // -- Step 6: Multi-select -----------------------------------------------
    let resp = io.request(&OutboundMessage::MultiSelect {
        id: next_id(),
        message: "Select toppings:".into(),
        options: vec![
            "cheese".into(),
            "pepperoni".into(),
            "mushrooms".into(),
            "olives".into(),
        ],
        defaults: Some(vec![0, 1]),
    });
    let toppings: Vec<String> = resp
        .value
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    io.send(&OutboundMessage::Log {
        level: "info".into(),
        message: format!("Toppings: {}", toppings.join(", ")),
    });

    // -- Step 7: Progress bar -----------------------------------------------
    io.send(&OutboundMessage::Output { text: "\n".into() });

    let total = 5;
    for i in 1..=total {
        io.send(&OutboundMessage::Progress {
            message: Some("Baking pizza".into()),
            current: Some(i),
            total: Some(total),
            done: None,
        });
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    io.send(&OutboundMessage::Progress {
        message: None,
        current: None,
        total: None,
        done: Some(true),
    });

    // -- Step 8: Store and Load ---------------------------------------------
    io.send(&OutboundMessage::Store {
        key: "last_user".into(),
        value: name.clone(),
    });
    io.send(&OutboundMessage::Store {
        key: "favorite_color".into(),
        value: color.clone(),
    });

    let resp = io.request(&OutboundMessage::Load {
        id: next_id(),
        key: "last_user".into(),
    });
    let loaded = resp
        .value
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("<null>");

    io.send(&OutboundMessage::Log {
        level: "debug".into(),
        message: format!("Store/load roundtrip: stored '{name}', loaded '{loaded}'"),
    });

    // -- Step 9: Exec -------------------------------------------------------
    let resp = io.request(&OutboundMessage::Exec {
        id: next_id(),
        command: "date +%Y-%m-%d".into(),
        cwd: None,
        timeout: Some(5),
    });
    let date = resp
        .value
        .as_ref()
        .and_then(|v| v.get("stdout"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    io.send(&OutboundMessage::Log {
        level: "info".into(),
        message: format!("Today is {date}"),
    });

    // -- Step 10: Metadata --------------------------------------------------
    let resp = io.request(&OutboundMessage::Metadata {
        id: next_id(),
        keys: vec!["git_tags".into(), "fledge_config".into()],
    });
    let _metadata = resp.value; // Use as needed

    io.send(&OutboundMessage::Log {
        level: "debug".into(),
        message: "Metadata response received".into(),
    });

    // -- Step 11: Spinner progress ------------------------------------------
    io.send(&OutboundMessage::Progress {
        message: Some("Finishing up".into()),
        current: None,
        total: None,
        done: None,
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    io.send(&OutboundMessage::Progress {
        message: None,
        current: None,
        total: None,
        done: Some(true),
    });

    // -- Done ---------------------------------------------------------------
    io.send(&OutboundMessage::Output {
        text: "\n  All done! Every protocol message exercised successfully.\n\n".into(),
    });
    io.send(&OutboundMessage::Log {
        level: "info".into(),
        message: "fledge-hello-rust finished".into(),
    });

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("plugin error: {e}");
        std::process::exit(1);
    }
}
