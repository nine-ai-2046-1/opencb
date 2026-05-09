use crate::scheduler::{JobStore, ScheduledJob};
use r2d2::{ManageConnection, Pool, PooledConnection};
use rusqlite::{params, Connection, OpenFlags, TransactionBehavior};
use std::error::Error;

pub const SCHEDULED_JOBS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS scheduled_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    message TEXT NOT NULL,
    run_at_iso TEXT NOT NULL,
    run_at_local_minute TEXT NOT NULL,
    run_at_unix_ms INTEGER NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    meta TEXT
);
"#;

/// Lightweight connection manager for rusqlite::Connection so we don't depend on an external crate
pub struct RusqliteConnectionManager {
    path: String,
    flags: OpenFlags,
}

impl RusqliteConnectionManager {
    pub fn file(path: &str) -> Self {
        Self {
            path: path.to_string(),
            flags: OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        }
    }
}

impl ManageConnection for RusqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = Connection::open_with_flags(&self.path, self.flags)?;
        // set a busy timeout for each new connection to reduce transient SQLITE_BUSY errors
        let _ = conn.busy_timeout(std::time::Duration::from_secs(2));
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1")?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub struct SqliteJobStore {
    pool: Pool<RusqliteConnectionManager>,
}

impl SqliteJobStore {
    /// Create a new SqliteJobStore backed by the provided database path.
    /// For in-memory tests use: "file::memory:?cache=shared"
    pub fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        let manager = RusqliteConnectionManager::file(db_path);
        let pool = Pool::builder().max_size(5).build(manager)?;

        // Ensure schema exists
        let conn = pool.get()?;
        // set a busy timeout to reduce transient SQLITE_BUSY errors under contention
        let _ = conn.busy_timeout(std::time::Duration::from_secs(2));
        conn.execute_batch(SCHEDULED_JOBS_SCHEMA)?;

        Ok(Self { pool })
    }

    fn conn(&self) -> Result<PooledConnection<RusqliteConnectionManager>, r2d2::Error> {
        self.pool.get()
    }

    /// Import multiple jobs inside a single DB transaction.
    pub fn import_jobs(&self, jobs: &[ScheduledJob]) -> Result<(), Box<dyn Error>> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        for job in jobs {
            tx.execute(
                "INSERT OR IGNORE INTO scheduled_jobs (id, job_type, message, run_at_iso, run_at_local_minute, run_at_unix_ms, status, attempts, created_at, updated_at, meta) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    job.id,
                    job.job_type,
                    job.message,
                    job.run_at_iso,
                    job.run_at_local_minute,
                    job.run_at_unix_ms,
                    job.status,
                    job.attempts,
                    job.created_at,
                    job.updated_at,
                    job.meta.as_ref().map(|m| serde_json::to_string(m).unwrap())
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

impl JobStore for SqliteJobStore {
    fn add_job(&self, job: &ScheduledJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO scheduled_jobs (id, job_type, message, run_at_iso, run_at_local_minute, run_at_unix_ms, status, attempts, created_at, updated_at, meta) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                job.id,
                job.job_type,
                job.message,
                job.run_at_iso,
                job.run_at_local_minute,
                job.run_at_unix_ms,
                job.status,
                job.attempts,
                job.created_at,
                job.updated_at,
                job.meta.as_ref().map(|m| serde_json::to_string(m).unwrap())
            ],
        )?;
        Ok(())
    }

    fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob> {
        // Transaction: select then delete matching rows
        let mut jobs = Vec::new();
        if let Ok(mut conn) = self.conn() {
            // Try to begin an IMMEDIATE transaction with a small retry/backoff to
            // avoid transient SQLITE_BUSY failures under contention.
            for attempt in 0..3 {
                match conn.transaction_with_behavior(TransactionBehavior::Immediate) {
                    Ok(mut tx) => {
                        // Collect rows into an owned Vec first to avoid borrow/lifetime issues
                        let mut ids = Vec::new();
                        let mut collector: Vec<ScheduledJob> = Vec::new();

                        // Limit stmt lifetime to this block so it drops before commit
                        {
                            let mut stmt = match tx.prepare("SELECT id, job_type, message, run_at_iso, run_at_local_minute, run_at_unix_ms, status, attempts, created_at, updated_at, meta FROM scheduled_jobs WHERE run_at_local_minute = ?1") {
                                Ok(s) => s,
                                Err(_) => return Vec::new(),
                            };

                            let iter = match stmt.query_map(params![local_minute], |r| {
                                let meta_text: Option<String> = r.get(10)?;
                                let meta = match meta_text {
                                    Some(t) => serde_json::from_str(&t).ok(),
                                    None => None,
                                };
                                Ok(ScheduledJob {
                                    id: r.get(0)?,
                                    job_type: r.get(1)?,
                                    message: r.get(2)?,
                                    run_at_iso: r.get(3)?,
                                    run_at_local_minute: r.get(4)?,
                                    run_at_unix_ms: r.get(5)?,
                                    status: r.get(6)?,
                                    attempts: r.get(7)?,
                                    created_at: r.get(8)?,
                                    updated_at: r.get(9)?,
                                    meta,
                                })
                            }) {
                                Ok(it) => it,
                                Err(_) => return Vec::new(),
                            };

                            for row_res in iter {
                                if let Ok(job) = row_res {
                                    ids.push(job.id.clone());
                                    collector.push(job);
                                }
                            }
                        }

                        // Delete all fetched ids and commit only on success. If delete
                        // or commit fails, rollback and retry the outer transaction.
                        if !ids.is_empty() {
                            // Build statement with variable placeholders
                            let mut q = String::from("DELETE FROM scheduled_jobs WHERE id IN (");
                            for (i, _) in ids.iter().enumerate() {
                                if i > 0 {
                                    q.push(',');
                                }
                                q.push_str(&format!("?{}", i + 1));
                            }
                            q.push(')');
                            let params_vec: Vec<&dyn rusqlite::ToSql> =
                                ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                            match tx.execute(&q, params_vec.as_slice()) {
                                Ok(deleted) => {
                                    if deleted as usize != ids.len() {
                                        let _ = tx.rollback();
                                        // mismatch: retry
                                        continue;
                                    }
                                    // attempt commit - commit consumes tx, so do it into a temp
                                    match tx.commit() {
                                        Ok(()) => {
                                            // commit succeeded -> extend returned jobs and exit
                                            jobs.extend(collector.into_iter());
                                            break;
                                        }
                                        Err(_) => {
                                            // commit failed, cannot rollback because tx was moved; just continue retry
                                            continue;
                                        }
                                    }
                                }
                                Err(_) => {
                                    let _ = tx.rollback();
                                    continue;
                                }
                            }
                        } else {
                            // nothing to delete, just commit and exit
                            match tx.commit() {
                                Ok(()) => break,
                                Err(_) => continue,
                            }
                        }
                    }
                    Err(_) => {
                        // small backoff then retry
                        std::thread::sleep(std::time::Duration::from_millis(50 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::ScheduledJob;
    use chrono::Local;

    fn make_store() -> SqliteJobStore {
        // Use shared in-memory so r2d2 connections see the same DB
        SqliteJobStore::new("file::memory:?cache=shared").expect("create store")
    }

    #[test]
    fn insert_and_claim() {
        let store = make_store();
        let now = Local::now();
        let local_minute = now.format("%Y-%m-%dT%H:%M").to_string();
        let job = ScheduledJob::new("id1".to_string(), "hey".to_string(), now);
        store.add_job(&job).expect("add job");

        let picked = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].id, job.id);

        // subsequent fetch should be empty
        let picked2 = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked2.len(), 0);
    }

    #[test]
    fn file_db_insert_and_claim() {
        // create a temp file path
        let mut p = std::env::temp_dir();
        let fname = format!("opencb_test_{}.db", uuid::Uuid::new_v4());
        p.push(fname);
        let path = p.to_string_lossy().to_string();

        // ensure no leftover
        let _ = std::fs::remove_file(&path);

        let store = SqliteJobStore::new(&path).expect("create file-backed store");
        let now = Local::now();
        let local_minute = now.format("%Y-%m-%dT%H:%M").to_string();
        let job = ScheduledJob::new("fid1".to_string(), "filey".to_string(), now);
        store.add_job(&job).expect("add job");

        // fetch once
        let picked = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].id, job.id);

        // subsequent fetch should be empty
        let picked2 = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked2.len(), 0);

        // cleanup
        let _ = std::fs::remove_file(&path);
    }
}
