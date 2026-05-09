# Draft: scheduler-sqlite-admin

## Requirements (confirmed)
- Implement Sqlite-backed JobStore (SqliteJobStore) implementing JobStore trait.
- Add admin HTTP POST /schedule endpoint (axum) protected by Bearer token (SCHEDULED_ADMIN_TOKEN).
- Modify CLI send -t/-d flow: prefer SCHEDULED_ADMIN_URL -> SCHEDULED_JOBS_DB -> disk fallback.
- Keep existing scheduled_jobs.json disk fallback; import to DB at startup when DB configured.
- Check interval: per minute (already implemented in Scheduler::start()).

## Technical Decisions
- DB library: rusqlite + r2d2 (Cargo.toml already contains rusqlite, r2d2, r2d2-rusqlite).
- Admin auth: Bearer token (user selected). Default: require SCHEDULED_ADMIN_TOKEN if set.
- Default DB path: no hardcoded default; require SCHEDULED_JOBS_DB env to enable DB backend (user selected).
- JobStore trait remains sync; SqliteJobStore will use transactions to atomically SELECT and DELETE due rows.

## Research Findings
- All scheduler core types and helpers are in src/scheduler.rs (ScheduledJob, JobStore trait, InMemoryJobStore, Scheduler::start, build_job, persist/load helpers, scheduled_jobs_file_path).
- CLI send persistence lives in src/main.rs (send branch) and uses scheduler::persist_job_to_disk().
- schedule_send_job helper in src/handler.rs schedules into the in-memory store; will be updated to accept Arc<dyn JobStore>.
- Cargo.toml already includes axum, rusqlite, r2d2 and r2d2-rusqlite — no new deps needed except reqwest (already added) for CLI POST.

## Open Questions
- Test workflow: TDD (write tests first) or tests-after (write code then tests)?

## Scope Boundaries
IN:
- src/sqlite_job_store.rs (new), src/admin.rs (new), modifications to src/main.rs and src/handler.rs, tests in tests/*.rs, README/docs updates.
OUT:
- GET/DELETE endpoints, UI, cron-style expressions, RBAC beyond simple bearer token.
