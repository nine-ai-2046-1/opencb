# Draft: Scheduler integration

## Requirements (confirmed)
- 用機器本地時區 (UTC+X) 做所有時間建立、儲存同比較（只精確到 minute）。
- opencb serve 啟動時要檢查 in-memory 變數 scheduled_jobs，同埋排程啟動時可處理已儲存嘅 job。
- CLI: `opencb send "msg" -t "HH:MM" [-d "YYYY-MM-DD"]`，若帶 -t 就唔即時發送，而係把 job 放入 scheduled_jobs（persist 或 runtime store）。
- 若 job 到達（compare YYYY-MM-DD HH:MM）且 type == "send"，先從 store 移除再執行發送，以免 duplicate。

## 技術決定
- Data structure (ScheduledJob):
  - id, job_type, message, run_at_iso, run_at_local_minute ("YYYY-MM-DDTHH:mm"), created_at, meta
- 抽象化 adapter: JobStore trait（add_job, fetch_and_remove_due_jobs）。預設實作為 InMemoryJobStore (Mutex<Vec<>>)，方便未來換成 SQLite/Redis。
- Scheduler: align to next minute boundary, interval 每 60s 檢查一次；先 load persisted jobs 檔案（scheduled_jobs.json），再 fetch_and_remove_due_jobs(local_minute)，hand off 給 JobExecutor。
- JobExecutor: HttpJobExecutor 用 serenity::http 發送到 config.channel_id；執行前 job 已從 store 移除。
- CLI 行為: 如果 serve 未提供即時 IPC，one-shot send -t 會 persist job 到 scheduled_jobs.json（預設路徑支援 env 覆蓋 SCHEDULED_JOBS_PATH），serve 每分鐘會 load 並加入 in-memory store。這個選擇簡單可靠，易於 switch 到 DB-backed。

## 實作位置（現時 repo）
- src/scheduler.rs — ScheduledJob, JobStore trait, InMemoryJobStore, Scheduler, JobExecutor, persist/load/build helpers
- src/main.rs — CLI send -t 呼叫 build_job -> persist_job_to_disk；serve 啟動時 load_jobs_from_disk 並把 job 加入 store
- src/handler.rs — schedule_send_job helper（runtime push 到 store）
- docs/SCHEDULER.md — 設計說明同換 adapter 指南

## Open questions (需要你決定)
1. 要即時把 CLI 的排程 job push 到已運行嘅 serve 嗎？（可用 local admin HTTP endpoint 或 Unix socket）
2. 定要持久化到 SQLite DB 嚟避免 disk fallback 嗎？（如果要可直接實作 SqliteJobStore）

## Scope Boundaries
- 包含：in-memory store 實作、disk-persist fallback、scheduler 每分鐘檢查、HTTP 發送執行路徑、switch 到 DB 的設計說明。
- 不包含（目前排除）：多機分布式鎖、enterprise-level job retry queue、完整 web UI。

## Next actions
- 已加日誌幫助觀察（InMemoryJobStore add/fetch、Scheduler load）。
- 請揀下一步：我會幫你做「新增 admin HTTP endpoint」或「實作 SqliteJobStore」或「保持現狀（只做 doc）」。
