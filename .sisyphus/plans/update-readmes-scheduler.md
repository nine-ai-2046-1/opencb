# Plan: Add English README.md and Cantonese README-ZH.md with Scheduling docs

## TL;DR
Create two README files:
- `README.md` — English, contains full project intro + new "Scheduling (SQLite & Admin)" section and examples.
- `README-ZH.md` — Cantonese, derived from the current repo README.md content plus the new scheduling section translated to Cantonese and with emoji.

This plan is decision-complete: it contains the exact file contents to write, the git commands to run, CI/QA checks, and acceptance criteria.

## Files to create (exact)

1) README.md (English) — EXACT CONTENT (copy this verbatim into README.md)

---BEGIN README.md---
# 🚀 OpenCB — Open CLI Broker/Bridge

> A lightweight Discord bot written in Rust that bridges CLI tools and Discord channels. It supports running as a long-lived "serve" bot or a short-lived "send" CLI to post messages.

## 📖 Project Overview

OpenCB (Open CLI Broker/Bridge) is a Rust-based Discord bot. Key features:
- Serve mode: connect to the Discord Gateway and stream structured message metadata as JSON.
- Send mode: post a one-off message via CLI / HTTP API and exit immediately.
- Pluggable scheduling: schedule sends for future delivery (new feature).

## 🛠️ Build & Install

Requirements
- Rust and Cargo (install via https://rustup.rs)

Build
```bash
git clone https://github.com/nine-ai-2046-1/opencb
cd opencb
cargo build            # debug
cargo build --release  # optimized
```

Install
```bash
cargo install --path .
# or copy binary to your PATH
```

## 🎮 Usage

First run to generate default config
```bash
opencb
# edit config.toml to add your Bot token
```

Send mode examples
```bash
opencb send "Hello World 🎉"
opencb send "This is a test" "part two"
```

Serve mode examples
```bash
opencb serve
opencb --config /path/to/config.toml serve
```

## 🔧 Configuration

Edit `config.toml` (auto-generated on first run). Example:
```toml
bot_token = "YOUR_DISCORD_BOT_TOKEN"
channel_id = 123456789012345678
debug = true
```

## ✅ New Feature: Scheduling (SQLite & Admin)

Overview
- The scheduler allows scheduling messages for future delivery. Storage precedence:
  1. SCHEDULED_JOBS_DB environment variable (absolute path to SQLite DB)
  2. If not set and you started `serve` with `--config /path/to/config.toml`, DB defaults to the same folder as the config file: `/path/to/scheduled_jobs.db`
  3. If no config provided, DB defaults to the executable folder (`<exe>/scheduled_jobs.db`) or the current working dir if exe parent can't be resolved.

Fallback
- If DB is not enabled, `opencb send -t "HH:MM"` falls back to appending to `scheduled_jobs.json` (existing behavior).

Admin endpoint (optional)
- Start a small admin HTTP server to accept scheduling via POST /schedule.
- Env vars:
  - `SCHEDULED_ADMIN_URL`: If set, CLI `send -t` will POST to `$SCHEDULED_ADMIN_URL/schedule` first.
  - `SCHEDULED_ADMIN_TOKEN`: Bearer token used to protect the admin endpoint.

CLI precedence for `send -t`
1. POST to admin if `SCHEDULED_ADMIN_URL` is set.
2. Else insert into SQLite DB if `SCHEDULED_JOBS_DB` is set or derived.
3. Else fallback to `scheduled_jobs.json` on disk.

Import behavior
- At startup, when DB mode is enabled and `scheduled_jobs.json` exists, the server imports jobs into DB inside a single transaction and renames the file to `scheduled_jobs.json.imported-<YYYYMMDD-HHMMSS>` to preserve a backup. Set `SCHEDULED_JOBS_IMPORT_SKIP=true` to disable import.

Environment variables
- `SCHEDULED_JOBS_DB` — optional path to SQLite DB file.
- `SCHEDULED_JOBS_PATH` — optional path to scheduled_jobs.json (disk fallback).
- `SCHEDULED_ADMIN_URL` — optional admin server base URL.
- `SCHEDULED_ADMIN_TOKEN` — optional Bearer token for admin endpoint.
- `SCHEDULED_JOBS_IMPORT_SKIP` — optional; set to `true` to skip import.

Testing & CI
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- Ensure CI installs sqlite dev libs (example snippet below).

Examples
```bash
# Admin (preferred)
export SCHEDULED_ADMIN_URL="http://127.0.0.1:9000"
export SCHEDULED_ADMIN_TOKEN="secret"
opencb send "hello" -t "16:22"

# Direct DB
export SCHEDULED_JOBS_DB="/var/lib/opencb/scheduled_jobs.db"
opencb send "hello" -t "16:22"
```

CI snippet for Ubuntu runners
```yaml
- name: Install sqlite dev
  run: |
    sudo apt-get update
    sudo apt-get install -y libsqlite3-dev
```

License
- See LICENSE

Happy coding! 🎉

---END README.md---

2) README-ZH.md (Cantonese) — EXACT CONTENT (use the repo's current README.md content as base and append the scheduling section in Cantonese). Copy the existing README.md (current repo content) and append the following Scheduling section in Cantonese (with emoji):

---BEGIN README-ZH.md ADDITION---

## ✅ 新功能：排程（SQLite & Admin）

概覽
- 排程功能支援三種儲存優先級：
  1. SCHEDULED_JOBS_DB 環境變數（SQLite DB 檔案絕對路徑）
  2. 如果冇設定且你用 `--config /path/to/config.toml` 啟動 `serve`，DB 會預設喺同一個資料夾：`/path/to/scheduled_jobs.db`
  3. 如果冇提供 config，DB 會預設到執行檔所在資料夾（`<exe>/scheduled_jobs.db`），如果無法解析則 fallback 到當前工作目錄。

Fallback
- 如果未啟用 DB，`opencb send -t "HH:MM"` 會 fallback 去把工作寫入 `scheduled_jobs.json`（維持既有行為）。

Admin 介面（可選）
- 可以啟動一個簡單嘅 admin HTTP server，接受 POST /schedule 用來新增排程。
- 環境變數：
  - `SCHEDULED_ADMIN_URL`：如果設定，CLI `send -t` 會先 POST 到 `$SCHEDULED_ADMIN_URL/schedule`。
  - `SCHEDULED_ADMIN_TOKEN`：保護 admin API 嘅 Bearer token。

匯入行為
- 當啟用 DB 並且發現 `scheduled_jobs.json`，啟動時會喺單一 transaction 內匯入 DB，並把原檔重新命名為 `scheduled_jobs.json.imported-<YYYYMMDD-HHMMSS>` 作為備份。若要跳過匯入，設 `SCHEDULED_JOBS_IMPORT_SKIP=true`。

環境變數小結
- `SCHEDULED_JOBS_DB` — 選填，SQLite DB 檔案路徑
- `SCHEDULED_JOBS_PATH` — 選填，scheduled_jobs.json 路徑
- `SCHEDULED_ADMIN_URL` — 選填，admin server 的 base URL
- `SCHEDULED_ADMIN_TOKEN` — 選填，admin endpoint 的 Bearer token
- `SCHEDULED_JOBS_IMPORT_SKIP` — 選填，設成 `true` 跳過匯入

Usage 範例
```bash
# admin（建議）
export SCHEDULED_ADMIN_URL="http://127.0.0.1:9000"
export SCHEDULED_ADMIN_TOKEN="secret"
opencb send "hello" -t "16:22"

# 直接寫 DB
export SCHEDULED_JOBS_DB="/var/lib/opencb/scheduled_jobs.db"
opencb send "hello" -t "16:22"
```

---END README-ZH.md ADDITION---

## Exact git commands (run from repo root)

1) Create branch
```bash
git checkout -b docs/scheduler-readme
```

2) Add files (the implementer will create README.md and README-ZH.md with the exact content above)
```bash
git add README.md README-ZH.md
git commit -m "docs: add English README and Cantonese README-ZH with Scheduling docs 📝"
git push -u origin docs/scheduler-readme
gh pr create --base dev/scheduler --head docs/scheduler-readme --title "docs: README (eng/zh) — add scheduling docs" --body "Add English README and Cantonese README-ZH explaining scheduling usage and env vars."
```

## QA / Acceptance (automatable)
- Verify files exist: `test -f README.md && test -f README-ZH.md`
- Formatting: ensure README.md and README-ZH.md contain the provided headers and the Scheduling section (grep for "Scheduling" and "排程").
- PR checklist: README changes only, examples copy-pasteable.

## Notes
- This plan does NOT modify code; it only prescribes README files content. Implementation must create the files with exact content blocks above.
- After docs PR merges, the implementation tasks (SqliteJobStore + admin) will be executed in separate branches per main plan.
