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

Scheduling (in-memory admin-driven)

Overview
- The scheduler now runs in pure in-memory mode by default. Scheduled jobs are kept in the running serve process memory and are not persisted to disk or SQLite by default.

Admin endpoint
- The serve process exposes a small admin HTTP endpoint POST /schedule which accepts ScheduledJob JSON and inserts it into the in-memory queue. This is the recommended way to schedule jobs when serve is running.
- Configuration: set scheduled_admin_bind in config.toml (default: "127.0.0.1:19001") to control the bind address for the admin HTTP endpoint. You can still set SCHEDULED_ADMIN_BIND env var as an alternate override.
- Authentication: if SCHEDULED_ADMIN_TOKEN is set (env), the admin endpoint requires `Authorization: Bearer <token>`.

CLI behavior (send with -t)
- The CLI `opencb send -t "HH:MM"` will attempt to POST to the configured admin endpoint (SCHEDULED_ADMIN_URL env or default localhost). If the admin server is not available or returns an error, scheduling will fail (there is no file/DB fallback in pure in-memory mode).

Migration & notes
- The previous SQLite and scheduled_jobs.json fallback code remains in the repository for now but is unused in the default in-memory flow. If you rely on persistent scheduling, tell us and we can re-enable or provide migration steps.

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
