# 🔬 OpenCB 技術文檔

> 深入探討 OpenCB 嘅技術實現、依賴包、安全性同邏輯流程 ✨

## 📦 依賴包說明

### 核心依賴

#### 🤖 serenity (v0.12)
**用途**：Discord API 包裝庫，提供 Gateway、HTTP、Model 等核心功能

**使用功能**：
- `client` — Discord Client 創建同管理
- `gateway` — 連接 Discord Gateway，接收實時事件
- `model` — Discord 數據模型（Message, User, Channel 等）
- `http` — HTTP API 調用（發送訊息、讀取頻道等）
- `utils` — 工具函數（權限計算、格式轉換等）
- `cache` — 緩存 Guild、Channel 等資訊，減少 API 調用
- `rustls_backend` — 純 Rust 實現嘅 TLS，唔使 OpenSSL

**為咩揀佢**：
- ✅ 成熟穩定，社區活躍
- ✅ 純 Rust 實現，安全性高
- ✅ 文檔齊全，例子多
- ✅ 支持 async/await

**官方文檔**：https://docs.rs/serenity/latest/serenity/

#### 🎮 poise (v0.6)
**用途**：基於 serenity 嘅現代命令框架

**現狀**：項目有引入但未大量使用，目前主要用 clap 做 CLI

**未來計劃**：可以考慮用 poise 替代 clap，提供更豐富嘅 Discord 命令功能

**官方文檔**：https://docs.rs/poise/latest/poise/

#### 🎯 clap (v4.5)
**用途**：命令行參數解析庫

**使用功能**：
- `derive` — 用 derive macro 定義 CLI 結構
- `Parser` — 自動解析命令行參數
- `Subcommand` — 支持子命令（serve, send）

**為咩揀佢**：
- ✅ 現代 Rust CLI 標準
- ✅ derive macro 簡潔易讀
- ✅ 自動生成 --help 和 --version

**官方文檔**：https://docs.rs/clap/latest/clap/

#### 📄 toml (v0.8)
**用途**：TOML 格式解析同序列化

**使用場景**：
- 讀取 config.toml
- 生成預設配置檔

**官方文檔**：https://docs.rs/toml/latest/toml/

#### 🔄 serde (v1.0) + serde_json (v1.0)
**用途**：序列化/反序列化框架

**使用場景**：
- `Config` struct 反序列化（從 TOML）
- `MessageMetadata` 序列化（到 JSON）
- `#[derive(Serialize, Deserialize)]` 自動實作

**官方文檔**：
- serde: https://docs.rs/serde/latest/serde/
- serde_json: https://docs.rs/serde_json/latest/serde_json/

#### ⚡ tokio (v1.0)
**用途**：Rust 異步運行時

**使用功能**：
- `macros` — #[tokio::main] 宏
- `rt-multi-thread` — 多線程異步運行時

**為咩需要**：serenity 同 Discord Gateway 需要異步環境

**官方文檔**：https://docs.rs/tokio/latest/tokio/

#### 📝 tracing (v0.1) + tracing-subscriber (v0.3)
**用途**：結構化日誌記錄

**使用場景**：
- 記錄 Bot 就緒、訊息發送等事件
- 根據 config.debug 設置日誌級別（DEBUG/INFO）
- 統一嘅日誌輸出格式

**官方文檔**：
- tracing: https://docs.rs/tracing/latest/tracing/
- tracing-subscriber: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/

### 開發依賴

#### 🧪 mockall (v0.12)
**用途**：Rust Mock 對象生成庫

**使用場景**：單元測試中用嚟 mock Discord API

#### ✅ predicates (v3.0)
**用途**：斷言庫，用於測試

### 依賴關係圖

```
opencb
├── serenity (Discord API)
│   └── tokio (異步運行時)
├── poise (命令框架，基於 serenity)
├── clap (CLI 解析)
├── toml (配置解析)
├── serde + serde_json (序列化)
└── tracing + tracing-subscriber (日誌)
```

## 🔒 安全性分析

### 🛡️ TLS 實現
**選用 rustls_backend 而唔係 openssl**
- ✅ **純 Rust 實現**：冇 C 代碼，減少內存安全漏洞
- ✅ **可審計**：所有代碼都係 Rust，易於安全審計
- ✅ **無 OpenSSL 依賴**：避開 OpenSSL 嘅歷史安全問題

### 🔐 Token 處理
**Bot Token 管理**：
- ⚙️ Token 存儲喺 `config.toml`，唔應該提交到 Git
- 🚫 `.gitignore` 應該忽略 `config.toml`（如果未有，要加）
- ⚠️ **警告**：千祈唔好將真實 Token 寫到代碼或提交到公開倉庫
- ✅ `load_config()` 會檢查 Token 係咪仲係預設值 `"YOUR_BOT_TOKEN_HERE"`

**建議**：
1. 將 `config.toml` 加到 `.gitignore`
2. 用環境變數覆蓋敏感配置（未實現，可考慮）
3. 定期輪換 Bot Token

### 🔍 輸入驗證
**訊息處理**：
- ✅ `extract_message_metadata()` 只讀取唔修改，冇注入風險
- ✅ JSON 輸出用 `serde_json::to_string_pretty()`，自動轉義

### ⚙️ 新功能風險：執行外部 CLI

- **功能**：透過 `config.toml` 定義 target CLI（例如 `[opencode]`）並可在 `argv` 用 `#INPUT#` 佔位符傳入訊息內容，Bot 收到訊息會執行外部命令並把 stdout 回覆到 channel
- **風險**：執行外部命令可能導致命令注入、資料洩露或系統被利用，特別係當 CLI 會觸發網絡請求或存取本地資源時

**緩解建議**：
1. 只允許受信任嘅 target name（例如白名單），避免任意 table 被當成可執行 target（可作為未來改進）
2. 若可能，用容器或 sandbox（例如 chroot、docker）執行外部 CLI，將風險隔離
3. 對輸入加嚴格長度限制（現時截斷為 1900 字元），並對生成檔案路徑做額外檢查
4. 在生產環境，避免使用帶有 shell expansion 嘅 cmd（直接執行二進制比透過 shell 字串更安全）
5. 日誌不要記錄敏感資訊（如完整訊息內容、token）到公開日誌


**CLI 輸入**：
- ✅ `clap` 自動驗證參數格式
- ✅ `send` 命令嘅訊息內容冇特殊處理（直接發送到 Discord）

### 🚫 權限控制
**Discord Intents**：
```rust
let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;
```
- ⚠️ **注意**：`MESSAGE_CONTENT` 係特權 Intent，需要喺 Discord Developer Portal 開啟
- ✅ 只申請必要嘅 Intents，遵循最小權限原則

### 📊 數據處理
**元數據提取**：
- ✅ 只提取公開訊息資訊（id, content, author, channel 等）
- ⚠️ **隱私提醒**：輸出嘅 JSON 包含用戶名、訊息內容，唔好隨便分享
- ✅ 冇保存訊息到本地數據庫，減少數據洩露風險

## 🔄 邏輯流程

### 主程序流程（main.rs）

```
用戶執行 opencb
    │
    ├─→ 解析 CLI 參數（clap）
    │   ├─→ 讀取 --config 參數（可選）
    │   └─→ 判斷命令：serve / send / 默認（serve）
    │
    ├─→ 載入配置（config::load_config）
    │   ├─→ 檢查 config.toml 存在？
    │   │   ├─→ 否：生成預設配置，退出
    │   │   └─→ 是：讀取並解析 TOML
    │   ├─→ 驗證 bot_token 唔係預設值
    │   └─→ 返回 Config struct
    │
    ├─→ 設置日誌（tracing_subscriber）
    │   └─→ 根據 config.debug 設置級別
    │
    └─→ 判斷命令模式
        │
        ├─→ [Send 模式]
        │   ├─→ 用 HTTP API 創建 Http client
        │   ├─→ 調用 ChannelId::send_message()
        │   ├─→ 輸出結果（成功/失敗）
        │   └─→ 退出（process::exit）
        │
        └─→ [Serve 模式]（默認）
            ├─→ 創建 Client builder
            │   ├─→ 設置 bot_token
            │   ├─→ 設置 GatewayIntents
            │   └─→ 綁定 ServeHandler 事件處理器
            ├─→ 啟動 Gateway 連接（client.start()）
            └─→ 進入事件循環（唔會主動退出）
```

### Serve 模式事件處理（handler.rs）

```
Discord Gateway 連接成功
    │
    ├─→ [ready 事件]
    │   ├─→ 輸出 Bot 就緒資訊（tracing::info）
    │   ├─→ 檢查 CHANNEL_ID 環境變數
    │   │   ├─→ 有：調用 send_message_to_channel() 發測試訊息
    │   │   └─→ 無：跳過
    │   └─→ 返回，等待其他事件
    │
    └─→ [message 事件]
        ├─→ 過濾：訊息作者係咪係 Bot 自己？
        │   └─→ 是：忽略（return）
        │
        ├─→ 提取元數據（inbound::extract_message_metadata）
        │   ├─→ 提取作者資訊（id, name, bot）
        │   ├─→ 提取頻道資訊（id, name, type）
        │   ├─→ 提取伺服器資訊（id, name）
        │   ├─→ 提取提及資訊（users, everyone）
        │   ├─→ 提取附件資訊（id, filename, size, url）
        │   └─→ 組裝 MessageMetadata struct
        │
        ├─→ 序列化到 JSON（serde_json::to_string_pretty）
        │   ├─→ 成功：輸出到 stdout
        │   └─→ 失敗：記錄錯誤（tracing::error）
        │
        └─→ 返回，等待下條訊息
```

### Send 模式流程（outbound.rs + main.rs）

```
用戶執行：opencb send "Hello World"
    │
    ├─→ CLI 解析：Commands::Send { message: ["Hello", "World"] }
    ├─→ 組裝訊息內容：message.join(" ") → "Hello World"
    ├─→ 創建 HTTP client（serenity::http::Http::new）
    ├─→ 調用 ChannelId::send_message()
    │   ├─→ 成功：輸出訊息 ID（tracing::info）
    │   └─→ 失敗：輸出錯誤提示（eprintln）
    └─→ 退出（process::exit）
```

### 配置載入流程（config.rs）

```
load_config(config_path)
    │
    ├─→ 判斷 config_path 有冇值？
    │   ├─→ 有：轉換為絕對路徑
    │   └─→ 無：用當前目錄 + "config.toml"
    │
    ├─→ 檢查文件存在？
    │   ├─→ 不存在：
    │   │   ├─→ config_path 係 None？
    │   │   │   ├─→ 是：生成預設 config.toml，返回錯誤
    │   │   │   └─→ 否：返回「文件不存在」錯誤
    │   │   └─→ 退出
    │   └─→ 存在：繼續
    │
    ├─→ 檢查文件權限（唔係 read-only）
    ├─→ 讀取文件內容（fs::read_to_string）
    ├─→ 解析 TOML（toml::from_str）
    ├─→ 驗證 bot_token != "YOUR_BOT_TOKEN_HERE"
    └─→ 返回 Config struct
```

## 🧩 模組交互圖

```
                    ┌─────────────┐
                    │   main.rs   │ (入口)
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
    ┌─────▼─────┐  ┌─────▼─────┐  ┌─────▼─────┐
    │  cli.rs   │  │ config.rs │  │ error.rs  │
    │ (CLI解析) │  │ (配置載入)│  │ (錯誤處理)│
    └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
          │                │                │
          │                │                │
          └────────────────┼────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼─────┐ ┌──▼──┐ ┌─────▼─────┐
        │ handler.rs │ │types│ │ outbound.rs│
        │ (事件處理) │ │.rs  │ │ (發送訊息)│
        └─────┬─────┘ └─────┘ └─────┬─────┘
              │                      │
              │         ┌────────────┘
              │         │
        ┌─────▼─────┐ │
        │ inbound.rs │◄┘
        │ (提取元數據)│
        └────────────┘
```

**交互說明**：
1. `main.rs` 調用 `cli.rs` 解析參數，調用 `config.rs` 載入配置
2. Serve 模式下，`main.rs` 創建 `handler.rs` 嘅 `ServeHandler`
3. `handler.rs` 收到訊息時，調用 `inbound.rs` 提取元數據
4. `handler.rs` 可以調用 `outbound.rs` 發送訊息（例如就緒測試訊息）
5. `types.rs` 提供共用類型，畀 `inbound.rs`、`handler.rs`、`outbound.rs` 使用
6. `error.rs` 提供錯誤處理，畀 `handler.rs` 和 `main.rs` 調用

## 🚀 性能考慮

### 異步運行時
- ✅ tokio 多線程運行時，充分利用多核 CPU
- ✅ 所有 I/O 操作（Discord API、檔案讀寫）都係異步嘅

### 緩存策略
- ✅ serenity 嘅 cache 功能，減少 API 調用
- ✅ `extract_message_metadata()` 會從 cache 讀取頻道/伺服器名稱

### 記憶體使用
- ⚠️ **注意**：長期運行（serve 模式）可能會累積 cache
- 💡 **建議**：如果發現記憶體洩露，可以考慮定期清理 cache 或者重啟 Bot

## 🔧 擴展建議

### 短期改進
1. 📝 將 `config.toml` 加到 `.gitignore`
2. 🔐 支持環境變數覆蓋配置（例如 `export OPENCB_BOT_TOKEN=...`）
3. 🧪 增加更多單元測試同集成測試
4. 📊 添加 Prometheus metrics（可選）

### 長期規劃
1. 🎮 用 `poise` 替代 `clap`，提供豐富嘅 Discord 命令
2. 🗄️ 添加數據庫支持，保存歷史訊息
3. 🤖 集成 AI 功能（例如用 OpenAI API 回覆訊息）
4. 🌐 提供 Web UI 查看統計資訊

---

🔬 **技術細節更新日期**：2026-05-06  
📧 **聯繫方式**：[你的聯繫方式]  
🎉 **Happy Coding!** 😊
