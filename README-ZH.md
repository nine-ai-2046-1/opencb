# 🚀 OpenCB（Open CLI Broker/Bridge）

> Discord Bot 實現，用嚟處理 Agent & CLI 頻道訊息嘅開源工具 ✨

## 📖 項目簡介

OpenCB（Open CLI Broker/Bridge）係一個用 Rust 寫嘅 Discord Bot，主要功能：
- 用嚟比 Agent & Non-Agent 經 Discord channel 溝通
- 📥 **Serve 模式**：連接 Discord Gateway，實時監聽訊息並輸出 JSON 元數據
- 📤 **Send 模式**：通過 HTTP API 發送單次訊息，唔使長期連接
- 📊 **元數據提取**：自動提取訊息內容、作者、頻道、提及、附件等資訊

（以下內容以現有 README.md 為基礎，並加入排程說明）

## ✅ 新功能：排程（SQLite & Admin）

概覽
- 排程功能支援三種儲存優先級：
  1. SCHEDULED_JOBS_DB 環境變數（SQLite DB 檔案絕對路徑）
  2. 如果冇設定且你用 `--config /path/to/config.toml` 啟動 `serve`，DB 會預設喺同一個資料夾：`/path/to/scheduled_jobs.db`
  3. 如果冇提供 config，DB 會預設到執行檔所在資料夾（`<exe>/scheduled_jobs.db`），如果無法解析則 fallback 到當前工作目錄。

Fallback
- 如果未啟用 DB，`opencb send -t "HH:MM"` 會 fallback 去把工作寫入 `scheduled_jobs.json`（維持既有行為）。

Admin 介面（可選）
- 可以啟動一個簡單嘅 admin HTTP server，接受 POST /schedule 用來新增排程。
- 環境變數：
  - `SCHEDULED_ADMIN_URL`：如果設定，CLI `send -t` 會先 POST 到 `$SCHEDULED_ADMIN_URL/schedule`。
  - `SCHEDULED_ADMIN_TOKEN`：保護 admin API 嘅 Bearer token。

匯入行為
- 當啟用 DB 並且發現 `scheduled_jobs.json`，啟動時會喺單一 transaction 內匯入 DB，並把原檔重新命名為 `scheduled_jobs.json.imported-<YYYYMMDD-HHMMSS>` 作為備份。若要跳過匯入，設 `SCHEDULED_JOBS_IMPORT_SKIP=true`。

環境變數小結
- `SCHEDULED_JOBS_DB` — 選填，SQLite DB 檔案路徑
- `SCHEDULED_JOBS_PATH` — 選填，scheduled_jobs.json 路徑
- `SCHEDULED_ADMIN_URL` — 選填，admin server 的 base URL
- `SCHEDULED_ADMIN_TOKEN` — 選填，admin endpoint 的 Bearer token
- `SCHEDULED_JOBS_IMPORT_SKIP` — 選填，設成 `true` 跳過匯入

Usage 範例
```bash
# admin（建議）
export SCHEDULED_ADMIN_URL="http://127.0.0.1:9000"
export SCHEDULED_ADMIN_TOKEN="secret"
opencb send "hello" -t "16:22"

# 直接寫 DB
export SCHEDULED_JOBS_DB="/var/lib/opencb/scheduled_jobs.db"
opencb send "hello" -t "16:22"
```

---

（保留原 README.md 其餘使用說明未改動）

🎉 **Happy Coding!** 有咩問題隨時提 issue 或者聯繫維護者 😊
