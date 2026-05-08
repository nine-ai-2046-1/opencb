//! ⏱️ 排程模組（抽象化 job store + scheduler）
//!
//! - 提供 ScheduledJob 結構
//! - JobStore trait（方便 later 換成 SQLite/Redis adaptor）
//! - InMemoryJobStore 實作（簡單 Mutex 保護的 Vec）
//! - JobExecutor trait + HttpJobExecutor（透過 HTTP API 發送訊息）
//! - Scheduler：每分鐘檢查到期 job，fetch & remove 後交給 executor 執行

use serde::{Deserialize, Serialize};
use serenity::builder::CreateMessage;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

/// Scheduled job (簡單版)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub job_type: String, // e.g. "send"
    pub message: String,
    /// ISO string created from local machine time (seconds zeroed)
    pub run_at_iso: String,
    /// local minute key for fast comparison: "YYYY-MM-DDTHH:mm"
    pub run_at_local_minute: String,
    pub created_at: String,
    pub meta: Option<serde_json::Value>,
}

/// Job store trait - sync methods so adapters can be simple to implement
pub trait JobStore: Send + Sync {
    /// Add a job to the store
    fn add_job(&self, job: ScheduledJob) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Atomically fetch and remove due jobs for the given local-minute key
    fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob>;
}

/// Simple in-memory store - Mutex<Vec<..>>
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

impl Default for InMemoryJobStore {
    fn default() -> Self {
        Self::new()
    }
}

impl JobStore for InMemoryJobStore {
    fn add_job(&self, job: ScheduledJob) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut g = self.inner.lock().map_err(|e| format!("mutex poisoned: {}", e))?;
        g.push(job);
        info!("InMemoryJobStore: job added, total={}", g.len());
        Ok(())
    }

    fn fetch_and_remove_due_jobs(&self, local_minute: &str) -> Vec<ScheduledJob> {
        let mut g = match self.inner.lock() {
            Ok(v) => v,
            Err(poisoned) => poisoned.into_inner(),
        };
        let (mut due, mut rest): (Vec<_>, Vec<_>) = g.drain(..).partition(|j| j.run_at_local_minute == local_minute);
        // put back rest
        *g = rest;
        if !due.is_empty() {
            info!("InMemoryJobStore: fetch_and_remove_due_jobs found {} jobs for {}", due.len(), local_minute);
            for j in &due {
                info!(" - job {} type={} message='{}'", j.id, j.job_type, j.message);
            }
        }
        due
    }
}

/// Job executor trait - returns a JoinHandle so scheduler can fire-and-forget
pub trait JobExecutor: Send + Sync {
    fn execute(&self, job: ScheduledJob) -> tokio::task::JoinHandle<()>;
}

/// HTTP-based executor that sends to configured channel via Serenity HTTP
pub struct HttpJobExecutor {
    bot_token: String,
    channel_id: u64,
}

impl HttpJobExecutor {
    pub fn new(bot_token: String, channel_id: u64) -> Self {
        Self { bot_token, channel_id }
    }
}

impl JobExecutor for HttpJobExecutor {
    fn execute(&self, job: ScheduledJob) -> tokio::task::JoinHandle<()> {
        let token = self.bot_token.clone();
        let channel_id = self.channel_id;
        tokio::spawn(async move {
            info!("Executing scheduled job {} (send) at {}", job.id, job.run_at_local_minute);
            let http = Http::new(&token);
            let ch = ChannelId::new(channel_id);
            match ch.send_message(&http, CreateMessage::new().content(&job.message)).await {
                Ok(_) => info!("Scheduled message sent: id={}", job.id),
                Err(e) => error!("Failed to send scheduled message {}: {:?}", job.id, e),
            }
        })
    }
}

/// Format local minute key "YYYY-MM-DDTHH:mm" using machine local time
pub fn format_local_minute(dt: &std::time::SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = dt.clone().into();
    format!("{:04}-{:02}-{:02}T{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute())
}

/// Scheduler: poll per-minute and hand off due jobs to executor
pub struct Scheduler {
    store: Arc<dyn JobStore>,
    executor: Arc<dyn JobExecutor>,
}

impl Scheduler {
    pub fn new(store: Arc<dyn JobStore>, executor: Arc<dyn JobExecutor>) -> Self {
        Self { store, executor }
    }

    /// Start the scheduler background task. It will align to next minute then run every 60s.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // align to next minute boundary
            let now = chrono::Local::now();
            let secs = now.second();
            let wait = 60 - secs;
            info!("Scheduler starting - waiting {}s to align to minute", wait);
            sleep(Duration::from_secs(wait as u64)).await;

            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = std::time::SystemTime::now();
                // build local minute key
                let local_minute = {
                    let dt: chrono::DateTime<chrono::Local> = now.into();
                    format!("{:04}-{:02}-{:02}T{:02}:{:02}", dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute())
                };
                info!("Scheduler tick for local_minute={}", local_minute);

                // load any persisted jobs from disk (CLI may have written scheduled_jobs.json)
                match load_jobs_from_disk(&scheduled_jobs_file_path()) {
                    Ok(jobs) => {
                        if !jobs.is_empty() {
                            info!("Loaded {} persisted jobs from disk", jobs.len());
                            for job in jobs {
                                if let Err(e) = self.store.add_job(job) {
                                    error!("Failed to add persisted job into store: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // non-fatal
                        error!("Failed to load persisted jobs from disk: {}", e);
                    }
                }

                let due = self.store.fetch_and_remove_due_jobs(&local_minute);
                for job in due {
                    // hand off
                    let _h = self.executor.execute(job);
                }
            }
        })
    }
}

// bring chrono helpers into scope for formatting
use chrono::prelude::*;
use regex;
use rand;

/// Persist a single job to disk file as JSON array (append-safe)
pub fn persist_job_to_disk(path: &str, job: &ScheduledJob) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut vec = Vec::new();
    if std::path::Path::new(path).exists() {
        let raw = std::fs::read_to_string(path)?;
        if !raw.trim().is_empty() {
            let parsed: Vec<ScheduledJob> = serde_json::from_str(&raw)?;
            vec = parsed;
        }
    }
    vec.push(job.clone());
    let s = serde_json::to_string_pretty(&vec)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// Load pending jobs from disk and remove the file
pub fn load_jobs_from_disk(path: &str) -> Result<Vec<ScheduledJob>, Box<dyn Error + Send + Sync>> {
    if !std::path::Path::new(path).exists() { return Ok(Vec::new()); }
    let raw = std::fs::read_to_string(path)?;
    if raw.trim().is_empty() { return Ok(Vec::new()); }
    let v: Vec<ScheduledJob> = serde_json::from_str(&raw)?;
    // remove file after loading
    let _ = std::fs::remove_file(path);
    Ok(v)
}

/// Resolve scheduled jobs file path. Priority:
/// 1) SCHEDULED_JOBS_PATH env var
/// 2) binary directory (same folder as executable) / scheduled_jobs.json
/// 3) fallback to ./scheduled_jobs.json
pub fn scheduled_jobs_file_path() -> String {
    if let Ok(p) = std::env::var("SCHEDULED_JOBS_PATH") {
        return p;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("scheduled_jobs.json");
            if let Some(s) = p.to_str() {
                return s.to_string();
            }
        }
    }
    // fallback
    "scheduled_jobs.json".to_string()
}

/// Build a ScheduledJob from message, optional date (YYYY-MM-DD) and HH:MM time using machine local timezone
pub fn build_job(message: String, date_opt: Option<String>, time_str: &str) -> Result<ScheduledJob, String> {
    // validate time
    let re = regex::Regex::new(r"^([01]?\d|2[0-3]):([0-5]\d)$").unwrap();
    let caps = re.captures(time_str).ok_or("invalid time format; expected HH:MM")?;
    let hour: u32 = caps.get(1).unwrap().as_str().parse().map_err(|_| "invalid hour")?;
    let minute: u32 = caps.get(2).unwrap().as_str().parse().map_err(|_| "invalid minute")?;

    let ymd = if let Some(d) = date_opt {
        // validate
        if !regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&d) {
            return Err("invalid date format; expected YYYY-MM-DD".into());
        }
        d
    } else {
        let now = chrono::Local::now();
        format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
    };

    let parts: Vec<u32> = ymd.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();
    if parts.len() != 3 { return Err("invalid date parts".into()); }
    let year = parts[0] as i32;
    let month = parts[1];
    let day = parts[2];

    let local_dt = chrono::Local.ymd(year, month, day).and_hms(hour, minute, 0);
    let run_at_iso = local_dt.to_rfc3339();
    let run_at_local_minute = format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hour, minute);

    Ok(ScheduledJob {
        id: format!("job-{}-{}", chrono::Local::now().timestamp(), rand::random::<u16>()),
        job_type: "send".to_string(),
        message,
        run_at_iso,
        run_at_local_minute,
        created_at: chrono::Local::now().to_rfc3339(),
        meta: None,
    })
}
