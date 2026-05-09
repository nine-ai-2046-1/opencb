# Plan: Implement SqliteJobStore + Admin /schedule + README updates

## TL;DR
> Summary: Implement a SQLite-backed JobStore (rusqlite + r2d2), add an axum admin POST /schedule endpoint protected by Bearer token, wire CLI send -t/-d to prefer admin -> DB -> disk, implement deterministic DB path precedence (env -> config file folder -> executable folder -> cwd), import scheduled_jobs.json into DB (rename to backup), add unit & integration tests, and update README (Chinese + English sections). 
> Deliverables: src/sqlite_job_store.rs, src/admin.rs, modifications to src/main.rs, src/config.rs, src/handler.rs, tests/, README updates, CI snippet. 
> Effort: Medium (3–6 dev days) 

## Context
- Current state: scheduler core types live in src/scheduler.rs (ScheduledJob, JobStore trait, InMemoryJobStore, persist/load helpers, scheduled_jobs_file_path()). CLI send -t persists jobs to scheduled_jobs.json; Scheduler::start() loads that file each minute. Cargo.toml already contains rusqlite, r2d2, r2d2-rusqlite and axum/reqwest. 
- New requirements from user: if SCHEDULED_JOBS_DB env is not set, derive DB path from (in order) config file folder (when user passed --config/-c), otherwise executable folder, otherwise working directory. Also update README (zh + en) for new usage and reminders.

## Work Objectives
### Core objective
- Replace (optionally) in-memory persistence with a production-ready SQLite adapter and admin endpoint while preserving existing disk fallback and ensuring no duplicate executions across possible concurrent schedulers.

### Deliverables
- src/sqlite_job_store.rs — SqliteJobStore implementing JobStore trait (r2d2 pool). 
- src/admin.rs — axum server starter + POST /schedule handler. 
- src/config.rs — add helper to return resolved config PathBuf. 
- src/main.rs — compute DB path with explicit precedence; import scheduled_jobs.json into DB when DB enabled; start admin server when SCHEDULED_ADMIN_ADDR/SCHEDULED_ADMIN_URL set. 
- src/handler.rs — accept Arc<dyn JobStore> in schedule_send_job helper. 
- tests/sqlite_jobstore_tests.rs, tests/admin_handler_tests.rs — unit & integration tests. 
- README.md — add Scheduling section (Chinese + English subsections). 
- CI snippet (GitHub Actions) instructions to ensure sqlite build deps exist. 

### Definition of Done
- All tests pass: cargo test
- Formatting and lints: cargo fmt -- --check && cargo clippy --all-targets -- -D warnings
- Admin endpoint POST /schedule returns 201 and job stored in DB (automated test). 
- CLI send -t prefers admin -> DB -> disk and prints created id. 

## Verification Strategy
- Test decision: tests-after with TDD-critical tests added before feature commit (unit tests for SqliteJobStore + handler-level integration tests). 
- QA policy: Every task below includes agent-executable QA scenarios (curl, cargo commands). 

## Execution Strategy
Implement in waves: Wave 1 (foundation) -> Wave 2 (admin + CLI) -> Wave 3 (tests & CI) -> Final Verification.

Wave 1 (foundation, parallelizable):
- Task 1: Add SQL schema & pragmas; implement SqliteJobStore skeleton (r2d2 pool). 
- Task 2: Add helper in config.rs to return resolved config file PathBuf.

Wave 2 (dependent):
- Task 3: Wire DB path selection logic in main.rs (env -> config folder -> exe folder -> cwd). 
- Task 4: Implement import routine: scheduled_jobs.json -> insert into DB in single transaction, then rename scheduled_jobs.json.imported-<ts>. 
- Task 5: Modify handler.rs schedule_send_job to accept Arc<dyn JobStore>. 

Wave 3 (integration & docs):
- Task 6: Implement src/admin.rs / POST /schedule with Bearer token auth. 
- Task 7: Modify CLI send flow to prefer admin -> DB -> disk. 
- Task 8: Add unit & integration tests; update README zh + en. 

Dependency Matrix (high-level)
- src/sqlite_job_store.rs → used by main.rs, admin.rs, handler.rs, tests
- config.rs helper → main.rs DB path computation
- main.rs → imports, wiring, import routine, start admin
- admin.rs → depends on JobStore trait + SqliteJobStore

## TODOs (decision-complete tasks)

- [ ] 1. Add SQL schema & pragmas — src/sqlite_job_store.rs

  What to do: Create file src/sqlite_job_store.rs and implement the following exact constants and code outline (copy/paste-ready):

  - SQL schema (exact):
    const SCHEDULED_JOBS_SCHEMA: &str = r#"
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA busy_timeout = 5000;

    CREATE TABLE IF NOT EXISTS scheduled_jobs (
      id TEXT PRIMARY KEY,
      job_type TEXT NOT NULL,
      message TEXT NOT NULL,
      run_at_iso TEXT NOT NULL,
      run_at_local_minute TEXT NOT NULL,
      run_at_unix_ms INTEGER NOT NULL,
      status TEXT NOT NULL DEFAULT 'scheduled',
      attempts INTEGER NOT NULL DEFAULT 0,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      meta TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_status_runat ON scheduled_jobs(status, run_at_unix_ms);
    "#;

  - Struct & factory (exact signatures):
    pub struct SqliteJobStore { pool: r2d2::Pool<r2d2_rusqlite::SqliteConnectionManager> }

    impl SqliteJobStore {
      pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = r2d2_rusqlite::SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::builder().max_size(5).build(manager)?;
        {
          let conn = pool.get()?;
          conn.execute_batch(SCHEDULED_JOBS_SCHEMA)?;
        }
        Ok(Self { pool })
      }
    }

  - JobStore impl (exact behaviors & SQL):
    impl JobStore for SqliteJobStore {
      fn add_job(&self, job: ScheduledJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute("INSERT OR IGNORE INTO scheduled_jobs (id,job_type,message,run_at_iso,run_at_local_minute,run_at_unix_ms,created_at,updated_at,meta) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", rusqlite::params![job.id, job.job_type, job.message, job.run_at_iso, job.run_at_local_minute, job.run_at_unix_ms, job.created_at, now, job.meta.as_ref().map(|m| m.to_string())])?;
        Ok(())
      }

      fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob> {
        // Decision-complete pattern (transactional claim):
        // 1) begin transaction
        // 2) SELECT id, ... FROM scheduled_jobs WHERE status='scheduled' AND run_at_local_minute = ? ORDER BY run_at_unix_ms
        // 3) For each row attempt to UPDATE scheduled_jobs SET status='in_progress', updated_at=? WHERE id=? AND status='scheduled'
        // 4) Collect rows where update affected 1 row (claimed)
        // 5) Commit transaction
        // 6) Return collected ScheduledJob vec
        
        let mut claimed = Vec::new();
        if let Ok(conn) = self.pool.get() {
          let tx = match conn.transaction() { Ok(t) => t, Err(_) => return vec![] };
          let mut stmt = match tx.prepare("SELECT id,job_type,message,run_at_iso,run_at_local_minute,run_at_unix_ms,created_at,meta FROM scheduled_jobs WHERE status='scheduled' AND run_at_local_minute = ?1 ORDER BY run_at_unix_ms") { Ok(s) => s, Err(_) => return vec![] };
          let rows = stmt.query_map([local_minute], |row| {
            Ok(ScheduledJob { id: row.get(0)?, job_type: row.get(1)?, message: row.get(2)?, run_at_iso: row.get(3)?, run_at_local_minute: row.get(4)?, run_at_unix_ms: row.get(5)?, created_at: row.get(6)?, meta: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()) })
          }).unwrap();
          for r in rows { if let Ok(job) = r {
            // attempt claim
            let now = chrono::Utc::now().to_rfc3339();
            let res = tx.execute("UPDATE scheduled_jobs SET status='in_progress', updated_at=?1 WHERE id=?2 AND status='scheduled'", rusqlite::params![now, &job.id]);
            if let Ok(1) = res { claimed.push(job); }
          }}
          let _ = tx.commit();
        }
        claimed
      }
    }

  Must NOT do: Use non-parameterized SQL or string interpolation. Use transaction for claiming.

  Recommended Agent Profile: category=deep; skills=["rust","sql","r2d2","rusqlite"]

  Parallelization: NO (this is foundational)

  References: scheduler.rs:ScheduledJob struct, JobStore trait — implement to match fields exactly.

  Acceptance Criteria:
  - [ ] cargo test includes sqlite unit tests (see tests below) and passes.
  - [ ] Manual check: sqlite3 <dbpath> '.schema' shows scheduled_jobs table.

- [ ] 2. Add helper to resolve config file path — src/config.rs

  What to do: Add public helper with exact signature:
    pub fn resolve_config_path(config_path: Option<&str>) -> Result<std::path::PathBuf, Box<dyn std::error::Error>>
  Behavior (decision-complete):
    - If config_path.is_some(): let p = Path::new(config_path.unwrap()); if p.is_relative() then return current_dir().join(p).canonicalize()? else return p.canonicalize()?;
    - If None: return current_dir().join("config.toml").canonicalize().or_else(|_| Ok(current_dir().join("config.toml")))
  Rationale: main.rs will call this to derive the config parent folder for DB default.

  Acceptance: compile and unit test that resolve_config_path handles relative, absolute, missing file paths.

- [ ] 3. Compute DB path and wire into main.rs (decision-complete)

  What to do (exact sequence in main.rs, before scheduler init and before any scheduled_jobs_file_path calls):
  1) Read env var: let env_db = std::env::var("SCHEDULED_JOBS_DB").ok();
  2) let resolved_db = if let Some(db) = env_db { db } else {
       // try config path from CLI: cli.config.as_deref()
       if let Some(cfg_arg) = cli.config.as_deref() {
         let cfg_path = config::resolve_config_path(Some(cfg_arg))?;
         let db_path = cfg_path.parent().unwrap_or_else(|| std::path::Path::new(".")).join("scheduled_jobs.db");
         db_path.to_string_lossy().to_string()
       } else if let Ok(exe) = std::env::current_exe() {
         exe.parent().map(|p| p.join("scheduled_jobs.db").to_string_lossy().to_string()).unwrap_or_else(|| "scheduled_jobs.db".to_string())
       } else {
         std::env::current_dir()?.join("scheduled_jobs.db").to_string_lossy().to_string()
       }
     };

  3) Set std::env::set_var("SCHEDULED_JOBS_DB", &resolved_db); // makes it visible for other code that might read env
  4) Instantiate store: let store: Arc<dyn JobStore> = Arc::new(SqliteJobStore::new(&resolved_db)?);

  Note: The scheduled_jobs.json path remains controlled by SCHEDULED_JOBS_PATH env (unchanged).

  Acceptance: printed startup log: "Using scheduled jobs DB at: {resolved_db}" and store initializes.

- [ ] 4. Import scheduled_jobs.json into DB at startup (decision-complete)

  What to do (main.rs, immediately after store creation and before scheduler.start()):
  1) let json_path = scheduler::scheduled_jobs_file_path();
  2) if std::path::Path::new(&json_path).exists() && std::env::var("SCHEDULED_JOBS_IMPORT_SKIP").unwrap_or_default() != "true" {
       let jobs = scheduler::load_jobs_from_disk(&json_path)?; // this already reads & deletes file; change to read-only
       // Instead of deleting, modify load_jobs_from_disk to return jobs WITHOUT deleting file; do not alter user-visible behavior here.
       // Decision: We'll implement new helper scheduler::read_jobs_from_disk_no_delete(path) that reads and returns Vec<ScheduledJob] without deleting.
       for job in jobs.iter() { store.add_job(job.clone())?; }
       let backup = format!("{}.imported-{}", json_path, chrono::Local::now().format("%Y%m%d-%H%M%S"));
       std::fs::rename(&json_path, &backup)?;
       log::info!("Imported {} jobs from {} into DB and backed up to {}", jobs.len(), json_path, backup);
     }

  Rationale: preserve original file as forensic artifact; ensure idempotency by renaming.

  Acceptance: after startup, DB contains imported jobs and scheduled_jobs.json.imported-<ts> exists.

- [ ] 5. Modify handler.rs schedule_send_job signature & wiring

  What to do: change places that used Arc<InMemoryJobStore> to Arc<dyn JobStore + Send + Sync> and update schedule_send_job to accept a trait object. Replace concrete types in ServeHandler struct to use Arc<dyn JobStore>.

  Acceptance: compile without trait object errors; schedule_send_job calls store.add_job(job).

- [ ] 6. Implement admin server — src/admin.rs

  What to do: new file src/admin.rs with these decision-complete elements:
  - types:
    #[derive(Deserialize)] struct AdminScheduleRequest { message: String, time: String, date: Option<String> }
    #[derive(Serialize)] struct AdminScheduleResponse { id: String, run_at_local_minute: String }
  - pub fn start_admin_server(bind_addr: &str, token: Option<String>, store: Arc<dyn JobStore + Send + Sync>) -> tokio::task::JoinHandle<()> {
      // create Router::new().route("/schedule", post(handler)).layer(Extension(store)).layer(Extension(token))
      // run axum::Server::bind(&addr.parse()?).serve(app.into_make_service()) in spawn
    }
  - handler behavior (exact):
    1) Validate Authorization header if token.is_some(): header must equal "Bearer {token}" (use constant-time compare via subtle::ConstantTimeEq or compare fixed strings but do not log token).
    2) Validate time format HH:MM via regex r"^\d{2}:\d{2}$" and optional date YYYY-MM-DD via chrono::NaiveDate::parse_from_str.
    3) Build job via scheduler::build_job(req.message, req.date, &req.time) (?) or use internal builder that sets id=Uuid::new_v4().to_string().
    4) store.add_job(job)
    5) return (StatusCode::CREATED, Json(AdminScheduleResponse{ id: job.id.clone(), run_at_local_minute: job.run_at_local_minute.clone() }))

  Acceptance: unit test that posts valid JSON returns 201 and store has one recorded job.

- [ ] 7. Modify CLI send flow (main.rs) — decision-complete

  What to do (exact sequence when send -t present):
  1) Build job locally (id = uuid::Uuid::new_v4().to_string())
  2) If let Ok(admin_url) = std::env::var("SCHEDULED_ADMIN_URL") {
       let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
       let mut req = client.post(format!("{}/schedule", admin_url)).json(&serde_json::json!({"message": job.message, "time": job.run_at_local_minute.split('T').nth(1).unwrap_or("00:00"), "date": job.run_at_local_minute.split('T').nth(0).unwrap_or("") }));
       if let Ok(token) = std::env::var("SCHEDULED_ADMIN_TOKEN") { req = req.bearer_auth(token); }
       match req.send().await {
         Ok(resp) if resp.status() == reqwest::StatusCode::CREATED => { println!("✅ Scheduled job {} at {} (via admin)", job.id, job.run_at_local_minute); return Ok(()) }
         _ => { log::warn!("Admin POST failed, falling back to DB/disk"); }
       }
     }
  3) Else if let Ok(db_path) = std::env::var("SCHEDULED_JOBS_DB") { let store = SqliteJobStore::new(&db_path)?; store.add_job(job)?; println!("✅ Scheduled job {} at {} (persisted to DB: {})", job.id, job.run_at_local_minute, db_path); return Ok(()) }
  4) Else fallback: let path = scheduler::scheduled_jobs_file_path(); scheduler::persist_job_to_disk(&path, &job)?; println!("✅ Scheduled job {} persisted to {}", job.id, path); return Ok(())

  Note: CLI code runs in async main; use tokio runtime. Use reqwest crate (already added). For fallbacks, always print exact path and id.

  Acceptance: manual smoke test: set SCHEDULED_ADMIN_URL/SCHEDULED_ADMIN_TOKEN -> curl post to admin -> verify; unset admin but set SCHEDULED_JOBS_DB -> cli send writes DB; else writes scheduled_jobs.json.

- [ ] 8. Tests (decision-complete)

  tests/sqlite_jobstore_tests.rs
  - test_add_and_fetch_due_now:
    * Create tempdir, db_path = tempdir.join("test.db"); let store = SqliteJobStore::new(db_path.to_str().unwrap()).unwrap();
    * let job = build_job("hello", None, &now_time_str) with run_at_local_minute == now minute
    * store.add_job(job.clone()).unwrap();
    * let found = store.fetch_and_remove_due_jobs(&job.run_at_local_minute);
    * assert_eq!(found.len(), 1);

  - test_claiming_is_atomic:
    * Insert multiple jobs with same minute; spawn 2 threads each calling fetch_and_remove_due_jobs; assert total popped equals inserted and no duplicates.

  tests/admin_handler_tests.rs
  - test_admin_handler_returns_201_on_valid_post:
    * Create MockStore { called: Mutex<Vec<ScheduledJob>> } impl JobStore; start handler via Router::new() in background bound to 127.0.0.1:0 (random port); POST with Authorization: Bearer testtoken to /schedule; assert 201 and MockStore.called.len()==1.
  - test_admin_rejects_missing_token: assert 401.

  Acceptance: cargo test covers these and passes in CI.

- [ ] 9. README updates (decision-complete)

  Files to change: README.md (top-level). If repo later contains English README file, mirror changes there — for now update README.md with two added subsections under a new header "Scheduling (SQLite + admin)".

  Exact content to add (copy-paste):
  ----
  ## Scheduling (SQLite & Admin)

  Overview
  - The scheduler supports three storage backends in order of preference:
    1. SCHEDULED_JOBS_DB (environment variable pointing to SQLite DB file)
    2. If not set and you provided `--config /path/to/config.toml` when running `serve`, the DB will default to the same folder as the config file: `/path/to/scheduled_jobs.db`
    3. If no config path provided, the DB defaults to the executable folder: `<exe>/scheduled_jobs.db` or the current working directory if exe parent is not resolvable.
  - If the DB is not used, CLI `send -t` will persist to `scheduled_jobs.json` (unchanged behavior).

  Environment variables
  - SCHEDULED_JOBS_DB: optional. Absolute path to SQLite DB file. If set, it is used directly.
  - SCHEDULED_ADMIN_URL: optional. If set, CLI `send -t` will POST to `$SCHEDULED_ADMIN_URL/schedule` first.
  - SCHEDULED_ADMIN_TOKEN: optional. Bearer token used to authenticate admin POST requests.
  - SCHEDULED_JOBS_PATH: path to scheduled_jobs.json file (unchanged behavior for disk fallback).
  - SCHEDULED_JOBS_IMPORT_SKIP: set to "true" to skip importing scheduled_jobs.json into DB at startup.

  Usage examples
  - Start server using config file (DB default will be next to config):
    opencb --config /etc/opencb/config.toml serve

  - Schedule from CLI via admin endpoint (preferred):
    export SCHEDULED_ADMIN_URL="http://127.0.0.1:9000"
    export SCHEDULED_ADMIN_TOKEN="token"
    opencb send "hello" -t "16:22"

  - Schedule directly to DB (no admin):
    export SCHEDULED_JOBS_DB="/var/lib/opencb/scheduled_jobs.db"
    opencb send "hello" -t "16:22"

  Migration note
  - On startup, if DB is enabled and scheduled_jobs.json exists, the file will be imported into DB and renamed to scheduled_jobs.json.imported-<timestamp> (for backup). Use SCHEDULED_JOBS_IMPORT_SKIP=true to prevent import.

  ----

  Also add a short Chinese paragraph (translation) — put both in README.md; keep wording concise.

  Acceptance: README.md contains the new Scheduling section and examples.

- [ ] 10. CI: ensure sqlite is available in GitHub Actions

  What to do: Add this step to CI workflow (YAML snippet to include before cargo build):
    - name: Install sqlite dev
      run: |
        sudo apt-get update
        sudo apt-get install -y libsqlite3-dev

  Alternative: set rusqlite feature "bundled" in Cargo.toml. Decision: use apt-get in CI to keep compiled size small.

  Acceptance: CI runners run cargo test successfully.

## Final Verification Wave (MANDATORY)
- F1. Plan Compliance Audit — verify all exact files added/modified listed above exist in branch.
- F2. Code Quality Review — run clippy, ensure no warnings.
- F3. Real Manual QA — start server with DB, POST admin, confirm job runs at scheduled minute (or simulate by setting run_at to minute soon). Use Playwright only if UI — not needed.
- F4. Scope Fidelity Check — ensure no extra endpoints (GET/DELETE) were added.

## Commit Strategy (exact)
1) git checkout -b feature/scheduler-sqlite-admin-impl
2) git add Cargo.toml (already updated) src/sqlite_job_store.rs src/admin.rs src/config.rs src/main.rs src/handler.rs tests/ README.md
3) git commit -m "feat(scheduler): sqlite job store + admin POST /schedule; DB path precedence and import"
4) git push -u origin feature/scheduler-sqlite-admin-impl

## Success Criteria
- Automated:
  - cargo fmt -- --check => exit 0
  - cargo clippy --all-targets -- -D warnings => exit 0
  - cargo test => all tests pass
- Behavioural:
  - CLI send -t with SCHEDULED_ADMIN_URL set posts to admin and returns 201
  - With SCHEDULED_JOBS_DB set CLI writes to DB
  - Without SCHEDULED_JOBS_DB and no admin URL CLI writes scheduled_jobs.json
- On serve startup with DB enabled scheduled_jobs.json is imported and renamed as backup

## Detailed Task List (decision-complete — each task includes QA scenarios)

- [ ] A. Implement SqliteJobStore file: src/sqlite_job_store.rs

  What to do:
  - Create src/sqlite_job_store.rs and implement the SqliteJobStore exactly as described in the plan above (SCHEDULED_JOBS_SCHEMA, struct, new(), JobStore impl with parameterized SQL and transactional claim pattern).
  - Export the type as pub use crate::sqlite_job_store::SqliteJobStore; from src/lib.rs or main.rs top-level module area as needed.

  Must NOT do:
  - Do not perform DB schema changes outside the provided SCHEDULED_JOBS_SCHEMA constant.
  - Do not use string interpolation for SQL.

  Recommended Agent Profile:
  - Category: deep
  - Skills: ["rust", "rusqlite", "r2d2", "sql"]

  Parallelization: NO (foundational)

  References:
  - src/scheduler.rs: ScheduledJob fields and JobStore trait (use for column mapping)
  - Cargo.toml: ensure rusqlite + r2d2 dependencies present

  Acceptance Criteria (agent-executable):
  - [ ] src/sqlite_job_store.rs exists and compiles: cargo build
  - [ ] sqlite schema present in DB: run `sqlite3 {db} ".schema"` shows scheduled_jobs table

  QA Scenarios:
  ```
  Scenario: Add job and fetch due job
    Tool: Bash
    Steps:
      - Run a small test program or unit test that creates SqliteJobStore::new(":memory:") and adds a ScheduledJob with run_at_local_minute equal to now minute
      - Call fetch_and_remove_due_jobs(now_minute)
    Expected:
      - fetch returns the job
    Evidence: .sisyphus/evidence/task-A-sqlite-add-fetch.txt
  ```

  Commit: YES | Message: "feat(scheduler): add SqliteJobStore (rusqlite + r2d2)"

- [ ] B. Add resolve_config_path helper to src/config.rs

  What to do:
  - Add pub fn resolve_config_path(config_path: Option<&str>) -> Result<PathBuf, Box<dyn Error>> implemented exactly as specified in plan.
  - Add unit tests verifying relative/absolute behavior under tests/config_resolve_tests.rs.

  Acceptance Criteria:
  - [ ] Function exists and unit tests pass (cargo test)

  QA Scenarios:
  ```
  Scenario: Resolve relative path
    Tool: Bash
    Steps:
      - cargo test --test config_resolve_tests
    Expected:
      - tests pass
    Evidence: .sisyphus/evidence/task-B-config-resolve.txt
  ```

  Commit: YES | Message: "feat(config): add resolve_config_path helper"

- [ ] C. Wire DB path computation & instantiate SqliteJobStore in src/main.rs

  What to do (exact edits):
  - At top imports: mod sqlite_job_store; use crate::sqlite_job_store::SqliteJobStore;
  - After load_config and before scheduler init, compute resolved_db exactly per plan and call std::env::set_var("SCHEDULED_JOBS_DB", &resolved_db);
  - Instantiate store: let store: Arc<dyn JobStore + Send + Sync> = Arc::new(SqliteJobStore::new(&resolved_db)?);
  - Log: info!("Using scheduled jobs DB at {}", resolved_db);

  Acceptance Criteria:
  - [ ] On startup (cargo run -- serve) log prints the resolved DB path
  - [ ] store initializes without error

  QA Scenarios:
  ```
  Scenario: Compute DB path from config arg
    Tool: Bash
    Steps:
      - run: `opencb --config ./examples/config.toml serve` (in test harness)
    Expected:
      - boot log: "Using scheduled jobs DB at: /full/path/scheduled_jobs.db"
    Evidence: .sisyphus/evidence/task-C-dbpath.txt
  ```

  Commit: YES | Message: "feat(main): compute DB path precedence and instantiate SqliteJobStore"

- [ ] D. Implement import routine for scheduled_jobs.json into DB (main.rs)

  What to do (exact):
  - Implement scheduler::read_jobs_from_disk_no_delete(path) or modify load_jobs_from_disk to accept a flag. Use read-only reader to parse JSON into Vec<ScheduledJob> without deleting the file.
  - After store creation, if json exists and SCHEDULED_JOBS_IMPORT_SKIP != "true": read jobs, insert each via store.add_job(job.clone()) within a DB transaction if possible, then rename original file to backup: scheduled_jobs.json.imported-<YYYYMMDD-HHMMSS>.

  Acceptance Criteria:
  - [ ] After startup with DB enabled and scheduled_jobs.json present, DB contains imported jobs and backup file exists.

  QA Scenarios:
  ```
  Scenario: Import jobs on startup
    Tool: Bash
    Steps:
      - create scheduled_jobs.json with 2 jobs in repo root
      - run `RUST_LOG=info cargo run -- serve` with SCHEDULED_JOBS_DB unset but --config provided to derive DB
    Expected:
      - startup log: "Imported 2 jobs from scheduled_jobs.json into DB and backed up to scheduled_jobs.json.imported-..."
      - DB now has 2 jobs according to a small query test
    Evidence: .sisyphus/evidence/task-D-import.txt
  ```

  Commit: YES | Message: "feat(scheduler): import scheduled_jobs.json into DB on startup and backup original"

- [ ] E. Change handler.rs to use Arc<dyn JobStore + Send + Sync>

  What to do:
  - Modify ServeHandler struct and schedule_send_job signature to accept Arc<dyn JobStore + Send + Sync> instead of Arc<InMemoryJobStore>.
  - Update code paths that construct ServeHandler in main.rs to pass store.clone().

  Acceptance:
  - [ ] compile and handler tests pass

  QA Scenarios:
  ```
  Scenario: schedule_send_job from bot event
    Tool: Bash
    Steps:
      - run unit test or simulated event that calls schedule_send_job with a mock store
    Expected:
      - mock store recorded add_job call
    Evidence: .sisyphus/evidence/task-E-handler.txt
  ```

  Commit: YES | Message: "refactor(handler): accept trait object JobStore for scheduling"

- [ ] F. Implement admin server src/admin.rs and start it from main.rs

  What to do:
  - Implement the axum router with POST /schedule handler, Authorization Bearer token validation, input validation, call to build_job or internal builder, and store.add_job(job).
  - Start server in a tokio::spawn background task when SCHEDULED_ADMIN_ADDR or SCHEDULED_ADMIN_URL is set.

  Acceptance Criteria:
  - [ ] POST /schedule with valid token returns 201 and DB contains job
  - [ ] Missing/invalid token returns 401

  QA Scenarios:
  ```
  Scenario: Admin post success
    Tool: Bash
    Steps:
      - Start server with SCHEDULED_ADMIN_ADDR=127.0.0.1:9001 and SCHEDULED_ADMIN_TOKEN=testtoken
      - curl -s -X POST "http://127.0.0.1:9001/schedule" -H "Authorization: Bearer testtoken" -H "Content-Type: application/json" -d '{"message":"hi","time":"00:54"}' -w "%{http_code}"
    Expected:
      - HTTP 201
      - DB contains the new job
    Evidence: .sisyphus/evidence/task-F-admin.txt
  ```

  Commit: YES | Message: "feat(admin): add admin HTTP POST /schedule (axum)"

- [ ] G. Modify CLI send path to prefer admin -> DB -> disk

  What to do (exact):
  - Update main.rs send branch to attempt admin POST when SCHEDULED_ADMIN_URL set (use reqwest with 5s timeout), then DB insert if SCHEDULED_JOBS_DB set, else persist_job_to_disk.
  - Ensure CLI generates UUID id client-side and includes it in POST body.

  Acceptance Criteria:
  - [ ] CLI `opencb send "msg" -t "HH:MM"` with SCHEDULED_ADMIN_URL set posts and prints created id
  - [ ] CLI with SCHEDULED_JOBS_DB set writes to DB
  - [ ] Fallback writes scheduled_jobs.json

  QA Scenarios:
  ```
  Scenario: CLI admin preference
    Tool: Bash
    Steps:
      - export SCHEDULED_ADMIN_URL and SCHEDULED_ADMIN_TOKEN
      - run `opencb send "hello" -t "16:22"`
    Expected:
      - prints: "✅ Scheduled job <id> at <run_at_local_minute> (via admin)"
    Evidence: .sisyphus/evidence/task-G-cli.txt
  ```

  Commit: YES | Message: "feat(cli): prefer admin then DB then disk for scheduled jobs"

- [ ] H. Tests & CI adjustments

  What to do:
  - Add tests described above under tests/; ensure in CI the sqlite dev libs are installed (update .github/workflows/ci.yml with apt-get step). Use :memory: where possible.

  Acceptance Criteria:
  - [ ] `cargo test` passes on local runner
  - [ ] CI workflow passes

  QA Scenarios:
  ```
  Scenario: Run full test suite
    Tool: Bash
    Steps:
      - cargo fmt -- --check
      - cargo clippy --all-targets -- -D warnings
      - cargo test
    Expected:
      - all commands exit 0
    Evidence: .sisyphus/evidence/task-H-tests.txt
  ```

  Commit: YES | Message: "test(ci): add sqlite tests and CI sqlite install step"


## Notes & Guardrails
- Use SCHEDULED_JOBS_DB for SQLite file path and keep SCHEDULED_JOBS_PATH for JSON path to avoid confusion. Update docs accordingly.
- Do not delete scheduled_jobs.json silently — rename to imported backup.
- Use parameterized SQL and transactions; claim jobs by UPDATE status='in_progress' WHERE id=? AND status='scheduled' to avoid duplicates.
- Keep JobStore trait synchronous to match rest of codebase.

Plan saved to: .sisyphus/plans/scheduler-sqlite-admin-impl.md
