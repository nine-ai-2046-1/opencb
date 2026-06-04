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

# Execute once, will automatically create config at ~/.config/opencb/config.toml

opencb

# Then edit the config file, fill in your Bot Token

vim ~/.config/opencb/config.toml

```

The default config path is `~/.config/opencb/config.toml`. If the file does not exist, OpenCB will automatically create it with default values and prompt you to fill in your Bot Token.

You can also specify a custom config path with `-c` / `--config`:

```bash
opencb -c /path/to/your/config.toml serve
opencb --config /path/to/your/config.toml send "Hello"
```

### Configuration File (~/.config/opencb/config.toml)

OpenCB uses a **profile-based** config format. Each profile holds its own token, channel filters, and send targets.

```toml
debug = false

[profiles.default]
profile_id = "default"
channel_type = "discord"

# channel_ids: which channels the bot LISTENS to in serve mode.
# Use ["*"] to accept messages from any channel.
channel_ids = ["*"]

bot_token = "YOUR_BOT_TOKEN_HERE"

# default_send_to_channel_ids: which channels the `send` command writes to.
# Must be specific IDs — wildcards are not allowed here.
default_send_to_channel_ids = ["123456789012345678"]

[profiles.default.targets.opencode]
cmd = "opencode"
argv = ["run", "#INPUT#"]
# work_dir = "/path/to/workdir"  # Optional
```

> **Note:** `channel_ids = ["*"]` means "accept from any channel" in serve mode.
> For the `send` command you must always provide specific IDs via `default_send_to_channel_ids` or `--rc`.

> **Legacy format:** The old flat `bot_token` / `channel_id` top-level format is still accepted as a compatibility fallback, but the profiles format above is the recommended approach.

## 🎯 New Feature: External CLI Target (chat-with-cli)

You can define a target CLI (e.g., `[opencode]`) in `~/.config/opencb/config.toml`. When the Bot receives a message, it will call this CLI and reply with the execution result to the channel.

Example (added to ~/.config/opencb/config.toml):

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

> ⚠️ **Important**: Go to https://discord.com/developers/applications Create an application, add a bot, and copy the Token to `~/.config/opencb/config.toml`

### Command 1: Send a message (Send mode)

```bash
# Send to channels defined in default_send_to_channel_ids of the default profile
opencb send "Hello World 🎉"

# Multi-word message — no quotes needed, words are joined automatically
opencb send Hello World this is a test

# Override target channel for this send only (--rc takes priority over config)
opencb send "Hello" --rc "123456789012345678"

# Use a specific profile instead of "default"
opencb send "Hello" --profile myprofile

# Send as a Direct Message to one or more users
opencb send "Hello" --ru "111222333444555666"

# Append user mentions at the end of the message
opencb send "Heads up!" --mu "111222333444555666,999888777666555444"

# Combine flags — override channel and append a mention
opencb send "Release done" --rc "123456789012345678" --mu "111222333444555666"
```

Features:

- ✅ Uses HTTP API, no gateway connection required
- ✅ Exits immediately after sending, suitable for script calls
- ✅ `--rc` overrides `default_send_to_channel_ids` for a single invocation
- ✅ `--ru` sends a DM instead of (or in addition to) channel messages
- ✅ `--mu` appends `<@id>` mentions to the message text

### Command 2: Start the bot (Serve mode)

```bash
# Start with the default profile
opencb serve

# Start with a specific profile
opencb serve --profile myprofile

# Optional: specify a custom config file path
opencb -c /path/to/config.toml serve
```

Features:

- 🔄 Continuously runs, listening to all messages
- 📊 Outputs message metadata to stdout in JSON format
- 📝 Suitable for pipelining to other tools or logging systems
- 🎯 Filters messages to channels listed in the profile's `channel_ids` (`["*"]` = all channels)

### Native Slash Commands

When the bot starts in serve mode, it automatically registers Discord slash commands:

| Command | Description |
|---------|-------------|
| `/echo <text>` | Echoes the text back to the channel, preserving formatting |
| `/cli <args>` | Invokes `nine-cli` with the given arguments. Supports quoted args (e.g. `"hello world"`). Streams stdout live to Discord with rolling updates; times out after 10 minutes |

Slash commands appear in Discord's `/` autocomplete menu immediately after the bot starts. No manual registration needed.

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
opencb/
├── 📄 Cargo.toml              # Project configuration, dependency definitions
├── 🔒 Cargo.lock              # Dependency version locking
├── 📖 README.md               # Project description (this file)
├── 📖 README-ZH.md            # Project description (Cantonese)
├── ⚖️  LICENSE                # License
├── 🚫 .gitignore              # Git ignore rules
├── ⚙️  config.sample.toml     # Sample configuration file
├── 📂 libs/
│   └── 📂 argv-parser/
│       └── mod.rs             # Quote-aware argv tokenizer (state machine)
├── 📂 src/
│   ├── 🚀 main.rs             # Main program entry point
│   ├── 📊 types.rs            # Message metadata type definitions
│   ├── ⚙️  config.rs          # Configuration processing module
│   ├── 🎯 cli.rs              # Command line argument parsing
│   ├── 🚨 error.rs            # Discord error handling
│   ├── 📤 outbound.rs         # Outbound message sending
│   ├── 📥 inbound.rs          # Inbound message metadata extraction
│   ├── 🤖 handler.rs          # Discord event handling (message + interaction)
│   ├── ✂️  splitter.rs        # Long message splitting
│   ├── 🕐 scheduler.rs        # Scheduled message job store
│   └── 📂 slash_commands/
│       ├── mod.rs             # SlashCommand trait (async), ResponseHandle, CommandDispatch enum, registration
│       ├── echo.rs            # /echo command implementation
│       └── cli.rs             # /cli command — nine-cli streaming implementation
└── 📂 openspec/               # Change management artifacts
```

### Module Description

| Module | Responsibility |
|--------|----------------|
| `main.rs` | 🚀 Program entry point, wires all modules together |
| `types.rs` | 📊 Defines `MessageMetadata` and related structs |
| `config.rs` | ⚙️ Reads, validates, and generates `~/.config/opencb/config.toml` |
| `cli.rs` | 🎯 Parses CLI arguments using clap (`serve`, `send` subcommands) |
| `error.rs` | 🚨 Handles Discord errors with user-friendly messages |
| `outbound.rs` | 📤 Sends messages via serenity HTTP |
| `inbound.rs` | 📥 Extracts structured metadata from Discord `Message` objects |
| `handler.rs` | 🤖 Implements `EventHandler`: message filter, slash command routing, interaction handler |
| `splitter.rs` | ✂️ Splits long messages into ≤2000-char Discord-safe chunks |
| `scheduler.rs` | 🕐 In-memory scheduled job store for `send -t` |
| `slash_commands/mod.rs` | 🎯 Async `SlashCommand` trait, `ResponseHandle`, `CommandDispatch` enum, command registry, Discord API registration |
| `slash_commands/echo.rs` | 💬 `/echo` command — echoes args verbatim |
| `slash_commands/cli.rs` | 🖥️ `/cli` command — tokenizes args, spawns `nine-cli`, streams stdout to Discord with rate-limited live edits and 10-min timeout |
| `libs/argv-parser/mod.rs` | 🔤 Quote-aware argv tokenizer (`tokenize_argv`) — handles single/double quotes and backslash escapes |

## 🧪 Testing

```bash
# Run all tests
cargo test

# Expected output:
# test result: ok. 72 passed; 0 failed; 0 ignored
```

## 📝 Frequently Asked Questions

### Q: How do I get a Bot Token?

A: Go to https://discord.com/developers/applications → Create Application → Bot → Reset Token → Copy the Token and paste it into `~/.config/opencb/config.toml`

### Q: How do I get a channel ID?

A: Discord User Settings → Advanced → Enable "Developer Mode" → Right-click on the channel → "Copy ID"

### Q: How do I add a Bot to a server?

A: Discord Developer Portal → OAuth2 → URL Generator → Check `bot` → Copy the generated URL → Open in browser → Select server → Authorize

### Q: Why isn't my bot receiving messages?

A: Ensure:

1. The bot has `MESSAGE CONTENT INTENT` permissions (Discord Developer Portal → Bot → Privileged Gateway Intents)

2. The bot is online (running in server mode)

3. The token in `~/.config/opencb/config.toml` is correct

## 📄 License

This project uses the license specified in the LICENSE file.

---

🎉 **Happy Coding!** Feel free to submit an issue or contact the maintainer if you have any questions 😊
