//! 🚨 Discord 錯誤處理模組
//! 專門處理 Discord Gateway 同 HTTP 錯誤，提供用戶友好提示 ✨

use serenity::Error;

pub fn handle_discord_error(e: serenity::Error) -> ! {
    match &e {
        Error::Gateway(gateway_err) => {
            match gateway_err {
                serenity::all::GatewayError::InvalidAuthentication => {
                    eprintln!("❌ Discord Token 認證失敗喇！😱");
                    eprintln!("可能原因：");
                    eprintln!("  1. config.toml 入面嘅 bot_token 無效或者過咗期");
                    eprintln!("  2. Token 格式唔啱（有多餘嘅空格或者引號）");
                    eprintln!("  3. Token 已經畀 Discord 撤銷咗");
                    eprintln!();
                    eprintln!("解決方法 💡：");
                    eprintln!("  1. 檢查 config.toml 入面嘅 bot_token 啱唔啱");
                    eprintln!("  2. 去 https://discord.com/developers/applications");
                    eprintln!("     揀你個 app → Bot → Reset Token");
                    eprintln!("  3. 更新 config.toml 之後再試過 🔄");
                }
                _ => {
                    eprintln!("❌ Discord 網關出錯喇 😵：{:?}", gateway_err);
                }
            }
        }
        Error::Http(http_err) => {
            eprintln!("❌ Discord HTTP 錯誤 🌐：{}", http_err);
        }
        _ => {
            eprintln!("❌ Discord 出錯喇 ⚠️：{:?}", e);
        }
    }
    std::process::exit(1);
}
