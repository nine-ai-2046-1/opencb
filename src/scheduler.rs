use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Canonical scheduled job representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledJob {
    pub id: String,
    pub job_type: String,
    pub message: String,
    pub run_at_iso: String,
    pub run_at_local_minute: String,
    pub run_at_unix_ms: i64,
    pub status: String,
    pub attempts: i32,
    pub created_at: String,
    pub updated_at: String,
    pub meta: Option<serde_json::Value>,
}

impl ScheduledJob {
    pub fn new(id: String, message: String, run_at: chrono::DateTime<Local>) -> Self {
        let run_at_iso = run_at.to_rfc3339();
        let run_at_local_minute = run_at.format("%Y-%m-%dT%H:%M").to_string();
        let run_at_unix_ms = run_at.timestamp_millis();
        let now = Local::now().to_rfc3339();
        Self {
            id,
            job_type: "send".to_string(),
            message,
            run_at_iso,
            run_at_local_minute,
            run_at_unix_ms,
            status: "scheduled".to_string(),
            attempts: 0,
            created_at: now.clone(),
            updated_at: now,
            meta: None,
        }
    }
}

/// JobStore trait — synchronous for simplicity
pub trait JobStore: Send + Sync {
    fn add_job(&self, job: &ScheduledJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob>;
}

/// Simple in-memory job store (for testing / default)
pub struct InMemoryJobStore {
    inner: Mutex<Vec<ScheduledJob>>,
}

impl InMemoryJobStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }
}

impl JobStore for InMemoryJobStore {
    fn add_job(&self, job: &ScheduledJob) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("mutex poisoned: {}", e))?;
        guard.push(job.clone());
        Ok(())
    }

    fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob> {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        for job in guard.drain(..) {
            if job.run_at_local_minute == local_minute {
                due.push(job);
            } else {
                remaining.push(job);
            }
        }
        *guard = remaining;
        due
    }
}

// Removed file/DB persistence helpers — scheduler is pure in-memory

/// Build job from message, optional date (YYYY-MM-DD) and time (HH:MM)
pub fn build_job(
    message: String,
    date: Option<String>,
    time: &str,
) -> Result<ScheduledJob, Box<dyn std::error::Error>> {
    // validate time HH:MM
    if !time.chars().all(|c| c.is_digit(10) || c == ':') || time.split(':').count() != 2 {
        return Err("invalid time format, expected HH:MM".into());
    }
    let date_str = match date {
        Some(d) => d,
        None => Local::now().format("%Y-%m-%d").to_string(),
    };
    let dt_str = format!("{}T{}:00", date_str, time);
    // parse as local
    let naive = chrono::NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%dT%H:%M:%S")?;
    let local_dt = Local
        .from_local_datetime(&naive)
        .single()
        .ok_or("ambiguous local datetime")?;
    let id = uuid::Uuid::new_v4().to_string();
    Ok(ScheduledJob::new(id, message, local_dt))
}
