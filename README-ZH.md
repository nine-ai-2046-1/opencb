# 🚀 OpenCB（Open CLI Broker/Bridge）

> Discord Bot 實現，用嚟處理 Agent & CLI 頻道訊息嘅開源工具 ✨

## 📖 項目簡介

OpenCB（Open CLI Broker/Bridge）係一個用 Rust 寫嘅 Discord Bot，主要功能：
- 用嚟比 Agent & Non-Agent 經 Discord channel 溝通
- 📥 **Serve 模式**：連接 Discord Gateway，實時監聽訊息並輸出 JSON 元數據
- 📤 **Send 模式**：通過 HTTP API 發送單次訊息，唔使長期連接
- 📊 **元數據提取**：自動提取訊息內容、作者、頻道、提及、附件等資訊

## 🛠️ 構建方法

### 前置要求
- 🦀 Rust (建議用 rustup 安裝：https://rustup.rs/)
- 📦 Cargo (Rust 包管理器，隨 Rust 一齊安裝)

### 構建命令
```bash
# 克隆項目（如果仲未克隆）
git clone https://github.com/nine-ai-2046-1/opencb
cd opencb

# 構建 debug 版本（開發用）
cargo build

# 構建 release 版本（優化過，速度快）
cargo build --release
```

構建完成後：
- Debug 版本：target/debug/opencb
- Release 版本：target/release/opencb

## 🌍 安裝到全局路徑

### 方法一：用 Cargo install（推薦）
```bash
# 喺項目根目錄執行
cargo install --path .

# 之後可以喺任何地方直接用 opencb 命令
opencb --help
```

### 方法二：手動複製到 PATH
```bash
# 複製到 /usr/local/bin（需要 sudo）
sudo cp target/release/opencb /usr/local/bin/

# 或者複製到 ~/.local/bin（唔使 sudo，要確保 ~/.local/bin 在 PATH 入面）
mkdir -p ~/.local/bin
cp target/release/opencb ~/.local/bin/
```

### 驗證安裝
```bash
opencb --version
# 應該輸出：opencb 0.1.0
```

## 🎮 使用方法

### 第一次使用（建立配置檔）
```bash
# 執行一次，會自動喺 ~/.config/opencb/config.toml 創建配置檔
opencb

# 跟住編輯配置檔，填入你嘅 Bot Token
vim ~/.config/opencb/config.toml
```

預設配置路徑係 `~/.config/opencb/config.toml`，如果檔案唔存在，OpenCB 會自動創建預設配置並提示你填入 Bot Token。

你亦可以用 `-c` / `--config` 指定自定義路徑：
```bash
opencb -c /path/to/your/config.toml serve
opencb --config /path/to/your/config.toml send "Hello"
```

### 配置檔說明（~/.config/opencb/config.toml）

OpenCB 使用 **Profile（設定檔）** 格式。每個 profile 有自己的 token、頻道篩選、同發送目標。

```toml
debug = false

[profiles.default]
profile_id = "default"
channel_type = "discord"

# channel_ids：serve 模式監聽哪些頻道。
# 用 ["*"] 表示接受任何頻道嘅訊息。
channel_ids = ["*"]

bot_token = "YOUR_BOT_TOKEN_HERE"

# default_send_to_channel_ids：send 命令發送到哪些頻道。
# 必須填具體 ID，唔可以用萬用字元 "*"。
default_send_to_channel_ids = ["123456789012345678"]

[profiles.default.targets.opencode]
cmd = "opencode"
argv = ["run", "#INPUT#"]
# work_dir = "/path/to/workdir"  # 可選
```

> **注意：** `channel_ids = ["*"]` 喺 serve 模式表示「接受任何頻道」。
> `send` 命令必須透過 `default_send_to_channel_ids` 或 `--rc` 指定具體頻道 ID。

> **向後兼容：** 舊版平坦格式（頂層 `bot_token` / `channel_id`）仍然有效，但建議使用上面嘅 profiles 格式。

## 🎯 新功能：外部 CLI 目標（chat-with-cli）

你可以喺 `~/.config/opencb/config.toml` 定義一個 target CLI（例如 `[opencode]`），當 Bot 收到訊息時會呼叫該 CLI 並把執行結果回覆到頻道。

範例（新增到 ~/.config/opencb/config.toml）：
```toml
[opencode]
cmd = "opencode"
argv = ["run", "#INPUT#"]
# work_dir = "/path/to/workdir"  # 可選
```

使用方法：
- 啟動 Bot 並指定 target：
```bash
opencb opencode
```
- 當 Discord channel 收到訊息（例如 `hello`），Bot 會執行： `opencode run "hello"`，然後把 CLI stdout 作為回覆發返上去

注意事項：
- Bot 只會對非 Bot 自己發嘅訊息觸發外部 CLI，避免無限迴圈
- CLI 執行結果若太長會被截斷（預設 1900 字元）
- 執行外部命令存在風險，請只用受信任嘅 CLI 並注意 work_dir 設定


> ⚠️ **重要**：去 https://discord.com/developers/applications 創建應用，添加 Bot，複製 Token 填到 `~/.config/opencb/config.toml`

### 命令一：發送訊息（Send 模式）

```bash
# 發送到 default profile 嘅 default_send_to_channel_ids 所指定嘅頻道
opencb send "Hello World 🎉"

# 多個單詞自動拼接（唔需要引號）
opencb send 你好 世界 測試訊息

# 覆蓋目標頻道（呢次只用 --rc 指定嘅頻道）
opencb send "Hello" --rc "123456789012345678"

# 使用指定 profile（唔係 default）
opencb send "Hello" --profile myprofile

# 以 DM 方式發送畀指定用戶
opencb send "Hello" --ru "111222333444555666"

# 喺訊息尾部加入 @mention
opencb send "上線咗！" --mu "111222333444555666,999888777666555444"

# 組合使用 —— 覆蓋頻道同加 mention
opencb send "完成部署" --rc "123456789012345678" --mu "111222333444555666"
```

特點：
- ✅ 用 HTTP API，唔使連接 Gateway
- ✅ 發完即退出，適合腳本調用
- ✅ `--rc` 覆蓋 `default_send_to_channel_ids`，只影響呢次發送
- ✅ `--ru` 以 DM 方式發送
- ✅ `--mu` 喺訊息尾部加入 `<@id>` mention

### 命令二：啟動 Bot（Serve 模式）

```bash
# 用 default profile 啟動
opencb serve

# 用指定 profile 啟動
opencb serve --profile myprofile

# 可選：指定自定義配置檔路徑
opencb -c /path/to/config.toml serve
```

特點：
- 🔄 持續運行，監聽所有訊息
- 📊 將訊息元數據以 JSON 格式輸出到 stdout
- 📝 適合管道到其他工具或日誌系統
- 🎯 根據 profile 嘅 `channel_ids` 篩選頻道（`["*"]` = 所有頻道）

### 原生 Slash 命令

Bot 啟動時會自動向 Discord 註冊 slash 命令：

| 命令 | 說明 |
|------|------|
| `/echo <文字>` | 原樣回覆輸入嘅文字，保留格式 |
| `/cli <參數>` | 用指定參數調用 `nine-cli`。支援引號包圍嘅參數（例如 `"hello world"`）。即時串流 stdout 到 Discord 並滾動更新；10 分鐘後逾時 |

Slash 命令喺 Bot 啟動後即時出現喺 Discord 嘅 `/` 自動補全選單，唔需要手動註冊。

### 完整命令列表
```bash
opencb --help
# 輸出：
# Usage: opencb [OPTIONS] [COMMAND]
#
# Commands:
#   serve   啟動 Discord Bot，監聽訊息（預設）
#   send    發送訊息到指定頻道
#
# Options:
#   -c, --config <FILE>  指定配置檔路徑
#   -h, --help           顯示說明
#   -V, --version        顯示版本
```

## 📁 項目結構

```
opencb/
├── 📄 Cargo.toml              # 項目配置、依賴定義
├── 🔒 Cargo.lock              # 依賴版本鎖定
├── 📖 README.md               # 項目說明（英文）
├── 📖 README-ZH.md            # 項目說明（本檔案）
├── ⚖️  LICENSE                # 許可證
├── 🚫 .gitignore              # Git 忽略規則
├── ⚙️  config.sample.toml     # 配置範例檔
├── 📂 libs/
│   └── 📂 argv-parser/
│       └── mod.rs             # 支援引號嘅 argv tokenizer（狀態機）
├── 📂 src/
│   ├── 🚀 main.rs             # 主程序入口
│   ├── 📊 types.rs            # 訊息元數據類型定義
│   ├── ⚙️  config.rs          # 配置處理模組
│   ├── 🎯 cli.rs              # 命令行參數解析
│   ├── 🚨 error.rs            # Discord 錯誤處理
│   ├── 📤 outbound.rs         # 出站訊息發送
│   ├── 📥 inbound.rs          # 入站訊息元數據提取
│   ├── 🤖 handler.rs          # Discord 事件處理（訊息 + interaction）
│   ├── ✂️  splitter.rs        # 長訊息分割
│   ├── 🕐 scheduler.rs        # 排程訊息 job store
│   └── 📂 slash_commands/
│       ├── mod.rs             # SlashCommand trait（async）、ResponseHandle、CommandDispatch enum、Discord 註冊
│       ├── echo.rs            # /echo 命令實作
│       └── cli.rs             # /cli 命令 — nine-cli 串流實作
└── 📂 openspec/               # 變更管理 artifacts
```

### 模組說明

| 模組 | 職責 |
|------|------|
| `main.rs` | 🚀 程序入口，組裝各模組 |
| `types.rs` | 📊 定義 `MessageMetadata` 等結構化類型 |
| `config.rs` | ⚙️ 讀取、驗證、生成 `~/.config/opencb/config.toml` |
| `cli.rs` | 🎯 用 clap 解析 CLI 參數（`serve`、`send` 子命令） |
| `error.rs` | 🚨 處理 Discord 錯誤，提供用戶友好提示 |
| `outbound.rs` | 📤 通過 serenity HTTP 發送訊息 |
| `inbound.rs` | 📥 從 Discord `Message` 提取結構化元數據 |
| `handler.rs` | 🤖 實作 `EventHandler`：訊息篩選、slash 命令路由、interaction 處理 |
| `splitter.rs` | ✂️ 將長訊息分割成 ≤2000 字元嘅 Discord 安全區塊 |
| `scheduler.rs` | 🕐 `send -t` 排程功能嘅 in-memory job store |
| `slash_commands/mod.rs` | 🎯 Async `SlashCommand` trait、`ResponseHandle`、`CommandDispatch` enum、命令登記、Discord API 註冊 |
| `slash_commands/echo.rs` | 💬 `/echo` 命令 — 原樣回覆 args |
| `slash_commands/cli.rs` | 🖥️ `/cli` 命令 — tokenize 參數、spawn `nine-cli`、串流 stdout 到 Discord（限速即時編輯，10 分鐘逾時） |
| `libs/argv-parser/mod.rs` | 🔤 支援引號嘅 argv tokenizer（`tokenize_argv`）— 處理單雙引號及反斜線跳脫 |

## 🧪 測試

```bash
# 運行所有測試
cargo test

# 預期輸出：
# test result: ok. 72 passed; 0 failed; 0 ignored
```

## 📝 常見問題

### Q: Bot Token 點獲取？
A: 去 https://discord.com/developers/applications → 創建應用 → Bot → Reset Token → 複製 Token 填到 `~/.config/opencb/config.toml`

### Q: 點樣獲取頻道 ID？
A: Discord 用戶設置 → 高級 → 開啟「開發者模式」→ 右鍵點擊頻道 →「複製 ID」

### Q: Bot 點樣加入伺服器？
A: Discord Developer Portal → OAuth2 → URL Generator → 勾選 `bot` → 複製生成的 URL → 喺瀏覽器打開 → 選擇伺服器 → 授權

### Q: 點解我嘅 Bot 收唔到訊息？
A: 確保：
1. Bot 有 `MESSAGE CONTENT INTENT` 權限（Discord Developer Portal → Bot → Privileged Gateway Intents）
2. Bot 在線（運行 serve 模式）
3. `~/.config/opencb/config.toml` 入面嘅 token 正確

## 📄 許可證

本項目採用 LICENSE 檔案中指定的許可證。

---

🎉 **Happy Coding!** 有咩問題隨時提 issue 或者聯繫維護者 😊
