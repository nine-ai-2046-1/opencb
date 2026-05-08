Scheduler & Adapter Guide
=========================

這個文件解釋 opencb 裡面嘅排程模組設計、資料結構，仲有點樣喺未來換成 SQLite adaptor。

核心概念
-----------
- ScheduledJob: in-memory job 資料結構，包含 id / job_type / message / run_at_iso / run_at_local_minute / created_at / meta。
- JobStore trait: 抽象化存儲層，提供 add_job() 同 fetch_and_remove_due_jobs(local_minute) 兩個方法。任何 adapter（in-memory, sqlite, redis）只要 implement 呢個 trait 就可以被 scheduler 使用。
- Scheduler: 每分鐘對齊後每 60s 檢查一次，fetch_and_remove_due_jobs() 回傳嘅 job 會被交俾 JobExecutor 執行。
- JobExecutor: 抽象執行器，例如現時有 HttpJobExecutor（用 serenity HTTP 發 message 到 config.channel_id）。

資料結構（ScheduledJob）
-------------------------
- id: String
- job_type: String (e.g. "send")
- message: String
- run_at_iso: String (由機器本地時間建立嘅 ISO 字串)
- run_at_local_minute: String ("YYYY-MM-DDTHH:mm"，用作比較 Key)
- created_at: String
- meta: Option<Json>

如何切換到 SQLite Adapter
---------------------------
1. 新增一個 struct SqliteJobStore 並 impl JobStore。建議使用 rusqlite 或 sqlx：

   - schema: 建議 table jobs (id TEXT PRIMARY KEY, job_type TEXT, message TEXT, run_at_iso TEXT, run_at_local_minute TEXT, created_at TEXT, meta TEXT)
   - add_job: INSERT INTO jobs (...)
   - fetch_and_remove_due_jobs(local_minute): 在 transaction 裡做
       a) SELECT * FROM jobs WHERE run_at_local_minute = ?
       b) DELETE FROM jobs WHERE run_at_local_minute = ?
       c) commit
     以上需要在 transaction 裡做以確保 atomic（避免 race condition）。

2. 在 main.rs 裡將 InMemoryJobStore 換成 SqliteJobStore 的實例，例如：

   let store = Arc::new(SqliteJobStore::new("./opencb_jobs.sqlite"));

3. Scheduler 不需要改動，因為它以 JobStore trait 作為 interface。

注意事項
---------
- 喺多 worker / 多 instance 架構中，SQLite 仍然有鎖競爭風險；更好的方案係 Redis 或資料庫支援的分布式鎖。
- run_at_local_minute 用機器本地時區進行格式化與比較；如果你會變更機器時區或跨 region 部署，請把 timezone 與原始 local datetime 一起存入 meta。

現有 behaviour
--------------
- 當你喺非 serve 一次性執行 opencb send "msg" -t "HH:MM" -d "YYYY-MM-DD"，程式會把排程 job 寫入 scheduled_jobs.json（若無 -d 則使用機器本地日期）。serve 在啟動時會嘗試 load scheduled_jobs.json 並把 jobs 加入 store。
- 若需要即時把 job 加入已在運行嘅 serve，建議加一個 admin HTTP endpoint 或啟用 SqliteJobStore 並令 CLI 直接插入 SQLite（兩者我都可以幫你 implement）。
