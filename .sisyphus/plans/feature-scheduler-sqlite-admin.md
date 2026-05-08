# Plan: feature/scheduler-sqlite-admin

TL;DR
- Commit current working tree to branch feature/scheduler-sqlite-admin, open preparatory PR to dev/scheduler.
- Implement SqliteJobStore (rusqlite) and optional admin HTTP POST /schedule endpoint (axum). CLI will prefer admin URL, then DB, then disk fallback.
- Add unit tests, run cargo fmt/clippy/tests, update README and docs, add a backlog section.

Effort: Medium — single branch, 6–10 commits. Parallelizable: NO (single sequence recommended).

Assumptions (no-more-questions defaults)
- PR base: dev/scheduler
- Admin endpoint: bind 127.0.0.1:9000, auth via Bearer token in env SCHEDULED_ADMIN_TOKEN
- DB path env: SCHEDULED_JOBS_DB (default ./scheduled_jobs.db)
- CLI preference order: SCHEDULED_ADMIN_URL -> SCHEDULED_JOBS_DB -> disk scheduled_jobs.json
- CI checks: cargo test, cargo fmt -- --check, cargo clippy --all-targets -- -D warnings

Files to add/change (explicit)
- Cargo.toml — add dependencies
  - under [dependencies] add these exact lines:
    axum = "0.7"
    tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
    rusqlite = "0.29"
    r2d2 = "0.8"                 # optional pool if chosen
    r2d2-rusqlite = "0.20"      # optional

- src/scheduler.rs — add SqliteJobStore implementation and DB helpers (schema + transactional fetch+remove). Keep InMemoryJobStore as default.
  - New public functions to add (exact signatures):
    pub struct SqliteJobStore { conn: std::sync::Mutex<rusqlite::Connection>, }
    impl SqliteJobStore {
        pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> { /* open, set WAL, create table if not exists */ }
    }
    impl JobStore for SqliteJobStore {
        fn add_job(&self, job: ScheduledJob) -> Result<(), Box<dyn Error + Send + Sync>> {
            // begin transaction; INSERT INTO jobs (...); commit
        }
        fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob> {
            // begin transaction; SELECT * WHERE run_at_local_minute = ?; DELETE those rows; commit; return Vec
        }
    }
  - SQL schema (exact):
    CREATE TABLE IF NOT EXISTS scheduled_jobs (
      id TEXT PRIMARY KEY,
      job_type TEXT NOT NULL,
      message TEXT NOT NULL,
      run_at_iso TEXT NOT NULL,
      run_at_local_minute TEXT NOT NULL,
      created_at TEXT NOT NULL,
      meta TEXT
    );

- src/admin.rs — new file implementing admin server
  - exports: pub fn start_admin_server(addr: &str, token: Option<String>, store: Arc<dyn JobStore>) -> tokio::task::JoinHandle<()>
  - POST /schedule accepts JSON { message, time, date? } builds job via scheduler::build_job and calls store.add_job(job). Responds 201 with created job as JSON. Validates Authorization: Bearer <token> if token set; otherwise rejects.

- src/main.rs — change serve init
  - On serve startup, choose store implementation:
    let store: Arc<dyn JobStore> = if let Ok(db) = std::env::var("SCHEDULED_JOBS_DB") { Arc::new(SqliteJobStore::new(&db)? ) } else { Arc::new(InMemoryJobStore::new()) };
  - If DB used and disk file exists: import scheduled_jobs.json into DB then delete file (unless SCHEDULED_JOBS_IMPORT_SKIP=true).
  - Start admin server if SCHEDULED_ADMIN_ADDR env exists (default 127.0.0.1:9000) using start_admin_server.
  - CLI send path: if time flag present, prefer to POST to SCHEDULED_ADMIN_URL if set; else persist to DB if SCHEDULED_JOBS_DB set; else fallback to persist_job_to_disk.

- src/cli.rs — no change to argument parsing; CLI behavior change occurs in main.rs send branch (decision-complete commands included below)

- tests/sqlite_jobstore_tests.rs — new tests
  - test_sqlite_store_add_and_fetch_due_now: create SqliteJobStore with :memory:, add job with run_at_local_minute = now minute, run fetch_and_remove_due_jobs(now) -> assert job returned and store empty afterwards.
  - test_sqlite_store_persist_and_reload: create file DB path, add job via store1, close, open store2 pointing same DB, fetch for that minute -> found.

- tests/admin_handler_tests.rs — new tests (handler-level)
  - test_admin_handler_returns_201_on_valid_post: call handler function directly with JSON payload and mock store (implement simple in-memory store impl that records add_job calls).
  - test_admin_handler_rejects_invalid_time: invalid time returns 400.

Decision-complete CLI behavior (exact sequence)
1) In main.rs send branch if Some(t):
   - Build job via scheduler::build_job(msg, date_opt, &t) -> Result<Job, Err>
   - If env SCHEDULED_ADMIN_URL present -> POST to $SCHEDULED_ADMIN_URL/schedule with JSON job payload (no id yet) and header Authorization: Bearer $SCHEDULED_ADMIN_TOKEN (if set). If POST returns 201 -> print "✅ Scheduled job {id} at {run_at_local_minute} (via admin)" and exit 0. If POST returns 401/403 -> print error and exit 1. If network error -> fall through to next option.
   - Else if env SCHEDULED_JOBS_DB present -> connect to DB path and call persist_job_to_db(db_path, &job) (helper to insert). Print "✅ Scheduled job {id} at {run_at_local_minute} (persisted to DB: {dbpath})" and exit 0.
   - Else -> call persist_job_to_disk(&path, &job) (existing behavior). Print persisted path and exit 0.

Exact Git + PR steps (copy/paste)
1) Create branch and commit current workspace
   git checkout dev/scheduler
   git pull --ff-only origin dev/scheduler
   git checkout -b feature/scheduler-sqlite-admin
   git add -A
   git commit -m "chore(scheduler): commit workspace before sqlite+admin feature work"
   git push -u origin feature/scheduler-sqlite-admin

2) Create PR (gh CLI recommended)
   gh pr create --base dev/scheduler --head feature/scheduler-sqlite-admin --title "chore: checkpoint current workspace before scheduler feature" --body "Checkpoint commit. Next: add SqliteJobStore and admin /schedule endpoint."

3) Implement changes in incremental commits (exact commit messages):
   - git add Cargo.toml
     git commit -m "chore(deps): add axum and rusqlite for admin endpoint and sqlite job store"
   - git add src/scheduler.rs
     git commit -m "feat(scheduler): add SqliteJobStore + DB helpers; keep disk fallback"
   - git add src/admin.rs src/main.rs
     git commit -m "feat(admin): add admin HTTP POST /schedule endpoint + server starter"
   - git add tests/
     git commit -m "test(scheduler): add SqliteJobStore and admin handler tests"
   - git push

CI & QA (exact commands)
- Local: cargo fmt --all -- --check
- Local: cargo clippy --all-targets -- -D warnings
- Local: cargo test --tests
- Runtime smoke test (example):
  export SCHEDULED_JOBS_DB="./test-scheduler.db"
  export SCHEDULED_ADMIN_ADDR="127.0.0.1:9001"
  export SCHEDULED_ADMIN_TOKEN="testtoken"
  RUST_LOG=info cargo run -- serve &
  # POST using curl to admin endpoint (next minute):
  curl -s -X POST "http://127.0.0.1:9001/schedule" -H "Authorization: Bearer testtoken" -H "Content-Type: application/json" -d '{"message":"hi","time":"00:54"}' | jq .

Rollback steps (copy/paste)
- Revert merge: git checkout dev/scheduler; git revert <merge-commit-sha> -m 1; git push
- Delete branch if needed: git push origin --delete feature/scheduler-sqlite-admin; git branch -D feature/scheduler-sqlite-admin
- If DB migration caused issues: restore DB backup: cp scheduled_jobs.db.bak scheduled_jobs.db

Cleanup & tests
- Remove unused functions or mark #[allow(dead_code)] if pending refactor.
- Add unit tests above; run cargo test and fix any warnings flagged by clippy.

README & docs changes (decision-complete)
- README.md: add section "Scheduling: admin endpoint and DB-backed persistence" with exact env vars and examples (copy the QA snippets). Also include security note about SCHEDULED_ADMIN_TOKEN and default binding to localhost.
- docs/SCHEDULER.md: append "Backlog" section listing future items (GET/DELETE endpoints, pagination, role-based auth, WAL tuning, feature-flag compile gating).

Acceptance criteria (must be automated)
- All tests pass (cargo test). cargo fmt and cargo clippy pass under commands above.
- Admin endpoint returns 201 and job stored in DB for valid POST with token.
- CLI send -t uses admin URL if set; else DB path if set; else disk fallback.

Backlog (short)
- GET /jobs, DELETE /jobs/:id, job retry/retry-count, exponential backoff, advanced scheduling (cron), role-based auth, multi-process coordination with distributed lock, WAL & connection pooling tuning.

What I will do now
- Create branch, commit current workspace, open PR (preparatory) and then implement SqliteJobStore + admin endpoint + tests + README updates, following the exact steps above. I will push incremental commits and update the PR until CI is green.

If you want a different auth method, DB path, or PR base, say it now; otherwise I proceed.
