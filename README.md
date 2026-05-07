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
# 執行一次，會自動創建 config.toml
opencb

# 跟住編輯 config.toml，填入你嘅 Bot Token
vim config.toml
```

### 配置檔說明（config.toml）
```toml
bot_token = "你嘅_Discord_Bot_Token"
channel_id = 123456789012345678  # 頻道 ID
owner_id = None                   # 可選：擁有者 ID
debug = true                       # 可選：是否開啟 debug 日誌
```

## 🎯 新功能：外部 CLI 目標（chat-with-cli）

你可以喺 `config.toml` 定義一個 target CLI（例如 `[opencode]`），當 Bot 收到訊息時會呼叫該 CLI 並把執行結果回覆到頻道。

範例（新增到 config.toml）：
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


> ⚠️ **重要**：去 https://discord.com/developers/applications 創建應用，添加 Bot，複製 Token 填到 config.toml

### 命令一：發送訊息（Send 模式）
```bash
# 發送一條訊息到配置檔入面指定嘅頻道
opencb send "Hello World 🎉"

# 發送多個單詞
opencb send "呢係一條測試訊息" "第二部份" "第三部份"
```

特點：
- ✅ 用 HTTP API，唔使連接 Gateway
- ✅ 發完即退出，適合腳本調用
- ✅ 唔使長期運行

### 命令二：啟動 Bot（Serve 模式）
```bash
# 方式一：直接執行（預設 serve）
opencb

# 方式二：明確指定 serve
opencb serve

# 可選：指定配置檔路徑
opencb --config /path/to/config.toml serve

# 可選：設置 CHANNEL_ID 環境變數，Bot 就緒時會發測試訊息
export CHANNEL_ID=123456789012345678
opencb serve
```

特點：
- 🔄 持續運行，監聽所有訊息
- 📊 將訊息元數據以 JSON 格式輸出到 stdout
- 📝 適合管道到其他工具或日誌系統

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
delivery/dev/
├── 📄 Cargo.toml              # 項目配置、依賴定義
├── 🔒 Cargo.lock              # 依賴版本鎖定
├── 📖 README.md               # 項目說明（本檔案）
├── ⚖️ LICENSE                # 許可證
├── 🚫 .gitignore              # Git 忽略規則
├── ⚙️ config.toml            # 配置文件（運行時自動生成）
├── 📂 src/
│   ├── 🚀 main.rs            # 主程序入口（159行）
│   ├── 📊 types.rs           # 訊息元數據類型定義（66行）
│   ├── ⚙️ config.rs          # 配置處理模組（80行）
│   ├── 🎯 cli.rs             # 命令行參數解析（31行）
│   ├── 🚨 error.rs           # Discord 錯誤處理（36行）
│   ├── 📤 outbound.rs        # 出站訊息發送（20行）
│   ├── 📥 inbound.rs         # 入站訊息處理（75行）
│   └── 🤖 handler.rs        # Discord 事件處理（57行）
└── 📂 docs/
    ├── 📖 README.md          # 呢個檔案
    └── 🔬 TECH.md            # 技術細節文檔
```

### 模組說明
| 模組 | 行數 | 職責 |
|------|------|------|
| `main.rs` | 159 | 🚀 程序入口，組裝各模組 |
| `types.rs` | 66 | 📊 定義 MessageMetadata 等結構化類型 |
| `config.rs` | 80 | ⚙️ 讀取、驗證、生成 config.toml |
| `cli.rs` | 31 | 🎯 用 clap 解析 CLI 參數 |
| `error.rs` | 36 | 🚨 處理 Discord 錯誤，提供用戶友好提示 |
| `outbound.rs` | 20 | 📤 通過 Context 發送訊息 |
| `inbound.rs` | 75 | 📥 從 Discord Message 提取元數據 |
| `handler.rs` | 57 | 🤖 實作 EventHandler，處理訊息事件 |

## 🧪 測試

```bash
# 運行所有測試
cargo test

# 預期輸出：
# running 4 tests
# test tests::test_cli_parsing_serve ... ok
# test tests::test_cli_parsing_send ... ok
# test tests::test_cli_parsing_default ... ok
# test tests::test_message_metadata_serialization ... ok
# test result: ok. 4 passed; 0 failed;
```

## 📝 常見問題

### Q: Bot Token 點獲取？
A: 去 https://discord.com/developers/applications → 創建應用 → Bot → Reset Token → 複製 Token 填到 config.toml

### Q: 點樣獲取頻道 ID？
A: Discord 用戶設置 → 高級 → 開啟「開發者模式」→ 右鍵點擊頻道 →「複製 ID」

### Q: Bot 點樣加入伺服器？
A: Discord Developer Portal → OAuth2 → URL Generator → 勾選 `bot` → 複製生成的 URL → 喺瀏覽器打開 → 選擇伺服器 → 授權

### Q: 點解我嘅 Bot 收唔到訊息？
A: 確保：
1. Bot 有 `MESSAGE CONTENT INTENT` 權限（Discord Developer Portal → Bot → Privileged Gateway Intents）
2. Bot 在線（運行 serve 模式）
3. config.toml 入面嘅 token 正確

## 📄 許可證

本項目採用 LICENSE 檔案中指定的許可證。

---

🎉 **Happy Coding!** 有咩問題隨時提 issue 或者聯繫維護者 😊
