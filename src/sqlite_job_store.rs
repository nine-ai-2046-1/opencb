use crate::scheduler::{JobStore, ScheduledJob};
use r2d2::{ManageConnection, Pool, PooledConnection};
use rusqlite::{params, Connection, OpenFlags};
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
        Connection::open_with_flags(&self.path, self.flags)
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
        let mut conn = pool.get()?;
        conn.execute_batch(SCHEDULED_JOBS_SCHEMA)?;

        Ok(Self { pool })
    }

    fn conn(&self) -> Result<PooledConnection<RusqliteConnectionManager>, r2d2::Error> {
        self.pool.get()
    }
}

impl JobStore for SqliteJobStore {
    fn add_job(&self, job: ScheduledJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        if let Ok(conn) = self.conn() {
            if let Ok(tx) = conn.transaction() {
                let mut stmt = match tx.prepare("SELECT id, job_type, message, run_at_iso, run_at_local_minute, run_at_unix_ms, status, attempts, created_at, updated_at, meta FROM scheduled_jobs WHERE run_at_local_minute = ?1") {
                    Ok(s) => s,
                    Err(_) => return Vec::new(),
                };
                let rows = stmt.query_map(params![local_minute], |r| {
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
                });

                let mut ids = Vec::new();
                if let Ok(iter) = rows {
                    for row_res in iter {
                        if let Ok(job) = row_res {
                            ids.push(job.id.clone());
                            jobs.push(job);
                        }
                    }
                }

                // Delete all fetched ids
                if !ids.is_empty() {
                    // Build statement with variable placeholders
                    let mut q = String::from("DELETE FROM scheduled_jobs WHERE id IN (");
                    for (i, _) in ids.iter().enumerate() {
                        if i > 0 { q.push(','); }
                        q.push_str(&format!("?{}", i+1));
                    }
                    q.push(')');
                    let params_vec: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                    let _ = tx.execute(&q, params_vec.as_slice());
                }

                let _ = tx.commit();
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
        store.add_job(job.clone()).expect("add job");

        let picked = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].id, job.id);

        // subsequent fetch should be empty
        let picked2 = store.fetch_and_remove_due_jobs(&local_minute);
        assert_eq!(picked2.len(), 0);
    }
}
