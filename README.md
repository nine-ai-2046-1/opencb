# 🚀 OpenCB (Open CLI Broker/Bridge)

> An open-source tool for handling Agent & CLI channel messages, implemented as a Discord Bot. ✨

## 📖 Project Introduction

OpenCB (Open CLI Broker/Bridge) is a Discord Bot written in Rust. Its main functions include:

- Enabling communication between Agents and Non-Agents via Discord channels

- 📥 **Serve Mode**: Connects to the Discord Gateway, listens for messages in real-time, and outputs JSON metadata.

- 📤 **Send Mode**: Sends single messages via HTTP API, eliminating the need for persistent connections.

- 📊 **Metadata Extraction**: Automatically extracts message content, author, channel, mentions, attachments, and other information.

## 🛠️ Build Method

### Prerequisites

- 🦀 Rust (Recommended to install using rustup: https://rustup.rs/)

- 📦 Cargo (Rust package manager, installed with Rust)

### Build Commands

```bash

# Clone the project (if not already cloned)

git clone https://github.com/nine-ai-2046-1/opencb

cd opencb

# Build debug version (for development)

cargo build

# Build release version (optimized, fast)

cargo build --release

```

After build:

- Debug version: target/debug/opencb

- Release version: target/release/opencb

## 🌍 Install to global path

### Method 1: Use Cargo install (recommended)

```bash

# Execute in the project root directory

cargo install --path .

# Then you can use the opencb command directly from anywhere

opencb --help

```

## Method 2: Manually copy to PATH

```bash

# Copy to /usr/local/bin (requires sudo)

sudo cp target/release/opencb /usr/local/bin/

# Or copy to ~/.local/bin (no sudo needed, make sure ~/.local/bin is in PATH)

mkdir -p ~/.local/bin

cp target/release/opencb ~/.local/bin/

```

### Verify Installation

```bash

opencb --version

# Should output: opencb 0.1.0

```

## 🎮 Usage

### First Use (Creating a Configuration File)

```bash

# Execute once, will automatically create config.toml

opencb

# Then edit config.toml, fill in your Bot Token

vim config.toml

```

### Configuration File Description (config.toml)

```toml

bot_token = "Your_Discord_Bot_Token"
channel_id = 123456789012345678 # Channel ID
owner_id = None # Optional: Owner ID
debug = true # Optional: Enable debug logging

```

## 🎯 New Feature: External CLI Target (chat-with-cli)

You can define a target CLI (e.g., `[opencode]`) in `config.toml`. When the Bot receives a message, it will call this CLI and reply with the execution result to the channel.

Example (added to config.toml):

```toml

[opencode]

cmd = "opencode"

argv = ["run", "#INPUT#"]

# work_dir = "/path/to/workdir" # Optional

```

Usage:

- Start the bot and specify the target:

```bash
opencb opencode

```

- When the Discord channel receives a message (e.g., `hello`), the bot will execute: `opencode run "hello"`, and then send the CLI stdout as a reply.

Notes:

- The bot will only trigger the external CLI for messages sent to non-bots to avoid infinite loops.

- CLI execution results will be truncated if they are too long (default 1900 characters).

- Executing external commands carries risks; please only use trusted CLIs and pay attention to the work_dir setting.

> ⚠️ **Important**: Go to https://discord.com/developers/applications Create an application, add a bot, and copy the Token to `config.toml`

### Command 1: Send a message (Send mode)

```bash

# Send a message to the channel specified in the configuration file
opencb send "Hello World 🎉"

# Send multiple words

opencb send "This is a test message" "Part 2" "Part 3"

```

Features:

- ✅ Uses HTTP API, no gateway connection required

- ✅ Exits immediately after sending, suitable for script calls

- ✅ Does not require long-term operation

### Command 2: Start the bot (Serve mode)

```bash

# Method 1: Execute directly (default serve)

opencb

# Method 2: Explicitly specify serve

opencb serve

# Optional: Specify the configuration file path

opencb --config /path/to/config.toml serve

# Optional: Set the CHANNEL_ID environment variable, bot A test message will be sent when ready.

export CHANNEL_ID=123456789012345678
opencb serve

```

Features:

- 🔄 Continuously runs, listening to all messages

- 📊 Outputs message metadata to stdout in JSON format

- 📝 Suitable for pipelining to other tools or logging systems

### Full Command List

```bash
opencb --help

# Output:

# Usage: opencb [OPTIONS] [COMMAND]

#
# Commands:

# serve Starts the Discord Bot, listening for messages (default)

# send Sends a message to the specified channel

#
# Options:

# -c, --config <FILE> Specifies the path to the configuration file

# -h, --help Displays the description

# -V, --version Displays the version

```

## 📁 Project Structure

```
delivery/dev/

├── 📄 Cargo.toml # Project configuration, dependency definitions

├── 🔒 Cargo.lock # Dependency version locking

├── 📖 README.md # Project description (this file)

├── ⚖️ LICENSE # License

├── 🚫 .gitignore # Git ignore rules

├── ⚙️ config.toml # Configuration file (automatically generated at runtime)

├── 📂 src/

│ ├── 🚀 main.rs # Main program entry point (line 159)

│ ├── 📊 types.rs # Message metadata type definitions (line 66)

│ ├── ⚙️ config.rs # Configuration processing module (line 80)

│ ├── 🎯 cli.rs # Command line argument parsing (line 31)

│ ├── 🚨 error.rs # Discord error handling (line 36)

│ ├── 📤 outbound.rs # Outbound message sending (line 20)

│ ├── 📥 inbound.rs # Inbound message handling (line 75)

│ └── 🤖 handler.rs # Discord event handling (line 57)

└── 📂 docs/

├── 📖 README.md # This file

└── 🔬 TECH.md # Technical details document

```

### Module Description

| Module | Line Number | Responsibility |

|------|------|------|

| `main.rs` | 159 | 🚀 Program entry point, assembling various modules |

| `types.rs` | 66 | 📊 Define structured types such as MessageMetadata |

| `config.rs` | 80 | ⚙️ Read, validate, and generate config.toml |

| `cli.rs` | 31 | 🎯 Parse CLI parameters using clap |

| `error.rs` | 36 | 🚨 Handle Discord errors and provide user-friendly prompts |

| `outbound.rs` | 20 | 📤 Send messages via Context |

| `inbound.rs` | 75 | 📥 Extract metadata from Discord Messages |

| `handler.rs` | 57 | 🤖 Implement EventHandler to handle message events |

## 🧪 Testing

```bash

# Run all tests
cargo test

# Expected output:

# running 4 tests

# test tests::test_cli_parsing_serve ... ok

# test tests::test_cli_parsing_send ... ok

# test tests::test_cli_parsing_default ... ok

# test tests::test_message_metadata_serialization ... ok

# test result: ok. 4 passed; 0 failed;

```

## 📝 Frequently Asked Questions

### Q: How do I get a Bot Token?

A: Go to https://discord.com/developers/applications → Create Application → Bot → Reset Token → Copy the Token and paste it into config.toml

### Q: How do I get a channel ID?

A: Discord User Settings → Advanced → Enable "Developer Mode" → Right-click on the channel → "Copy ID"

### Q: How do I add a Bot to a server?

A: Discord Developer Portal → OAuth2 → URL Generator → Check `bot` → Copy the generated URL → Open in browser → Select server → Authorize

### Q: Why isn't my bot receiving messages?

A: Ensure:

1. The bot has `MESSAGE CONTENT INTENT` permissions (Discord Developer Portal → Bot → Privileged Gateway Intents)

2. The bot is online (running in server mode)

3. The token in config.toml is correct

## 📄 License

This project uses the license specified in the LICENSE file.

---

🎉 **Happy Coding!** Feel free to submit an issue or contact the maintainer if you have any questions 😊
