# 🚀 OpenCB — Open CLI Broker/Bridge

A lightweight Discord bot written in Rust that bridges CLI tools and Discord channels. It supports running as a long-lived "serve" bot or a short-lived "send" CLI to post messages.

Key features
- Serve mode: connect to Discord Gateway and stream structured message metadata as JSON.
- Send mode: post a one-off message via CLI / HTTP API and exit immediately.
- Pluggable scheduling: schedule sends for future delivery (new feature).

Build & install
- Requirements: Rust + Cargo
- Build: `cargo build` (debug) or `cargo build --release` (release)
- Install: `cargo install --path .`

Usage highlights
- Create a config (first run auto-creates `config.toml`), edit it with your Bot Token.
- Send example: `opencb send "Hello World 🎉"`
- Serve example: `opencb serve` or `opencb --config /path/to/config.toml serve`

Scheduling (SQLite & Admin)

Overview
- The scheduler supports three storage options in this precedence:
  1. SCHEDULED_JOBS_DB environment variable (absolute path to SQLite DB file)
  2. If not set and you started `serve` with `--config /path/to/config.toml`, the DB defaults to the same folder as the config file: `/path/to/scheduled_jobs.db`
  3. If no config path provided, the DB defaults to the executable folder (`<exe>/scheduled_jobs.db`) or the current working directory if exe parent can't be resolved.

If no DB is used, `opencb send -t "HH:MM"` falls back to persisting to `scheduled_jobs.json` (unchanged behavior).

Admin endpoint (optional)
- You can run a small admin HTTP server that accepts scheduled job submissions via POST /schedule. This is useful for remote scheduling or CI automation.
- Env vars:
  - SCHEDULED_ADMIN_URL: If set, CLI `send -t` will POST to `$SCHEDULED_ADMIN_URL/schedule` first.
  - SCHEDULED_ADMIN_TOKEN: Bearer token used to protect the admin endpoint. If set, requests must include `Authorization: Bearer <token>`.

CLI precedence (send with -t)
1. If SCHEDULED_ADMIN_URL is set, `send -t` will try POST /schedule and respect the admin response.
2. Else if SCHEDULED_JOBS_DB is set (or derived via config/exe/cwd), the job is inserted into the SQLite DB.
3. Else fallback: append to `scheduled_jobs.json` in the configured or default path.

Migration & import
- When DB mode is enabled at startup, if `scheduled_jobs.json` exists the server will import its jobs into the DB inside a single transaction, and the original file will be renamed to `scheduled_jobs.json.imported-<YYYYMMDD-HHMMSS>` as a backup.
- To skip automatic import, set `SCHEDULED_JOBS_IMPORT_SKIP=true`.

Environment variables (summary)
- SCHEDULED_JOBS_DB — optional. Absolute path to SQLite DB file.
- SCHEDULED_JOBS_PATH — optional. Path to scheduled_jobs.json (fallback file). Default resolution: env -> exe folder -> cwd.
- SCHEDULED_ADMIN_URL — optional. Admin server base URL.
- SCHEDULED_ADMIN_TOKEN — optional. Bearer token for admin endpoint.
- SCHEDULED_JOBS_IMPORT_SKIP — optional. If "true", skip importing scheduled_jobs.json into DB at startup.

Testing & CI notes
- Tests: `cargo test`
- Formatting: `cargo fmt -- --check`
- Linting: `cargo clippy --all-targets -- -D warnings`
- CI must install sqlite dev libs on Linux runners. Example GitHub Actions step:

```yaml
- name: Install sqlite dev
  run: |
    sudo apt-get update
    sudo apt-get install -y libsqlite3-dev
```

Examples
- Start server with config (DB default next to config):
  `opencb --config /etc/opencb/config.toml serve`
- Schedule via admin (preferred):
  ```bash
  export SCHEDULED_ADMIN_URL="http://127.0.0.1:9000"
  export SCHEDULED_ADMIN_TOKEN="secret"
  opencb send "hello" -t "16:22"
  ```
- Schedule to DB directly:
  ```bash
  export SCHEDULED_JOBS_DB="/var/lib/opencb/scheduled_jobs.db"
  opencb send "hello" -t "16:22"
  ```

License
- See LICENSE file.

Happy coding! 🎉
