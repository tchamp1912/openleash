use async_trait::async_trait;
use leash_ai_backend::ApprovalBackend;
use leash_ai_core::models::ApprovalRequest;
use leash_ai_core::{Result, LeashError};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ChatId};

pub struct TelegramApprovalBackend {
    token: String,
    chat_id: i64,
}

impl TelegramApprovalBackend {
    pub fn new(token: String, chat_id: i64) -> Self {
        Self { token, chat_id }
    }

    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

#[async_trait]
impl ApprovalBackend for TelegramApprovalBackend {
    async fn notify_approval(&self, req: &ApprovalRequest) -> Result<()> {
        let bot = Bot::new(&self.token);
        
        let resource_id = Self::escape_html(&req.resource_id);
        let reason = Self::escape_html(&req.reason);

        let (icon, title) = match req.resource_type {
            leash_ai_core::models::ResourceType::Secret => ("🔑", "Secret Request"),
            leash_ai_core::models::ResourceType::Package => ("📦", "Package Request"),
            leash_ai_core::models::ResourceType::Command => ("💻", "Command Request"),
            leash_ai_core::models::ResourceType::System => {
                if req.resource_id == "keychain-unlock" {
                    ("🔓", "Keychain Unlock Request")
                } else {
                    ("⚙️", "System Request")
                }
            }
        };

        let message = format!(
            "{} <b>{}</b>

<b>Resource</b>: <code>{}</code>
<b>Rationale</b>: <i>{}</i>
<b>Expires</b>: {}",
            icon, title, resource_id, reason, req.expires_at.format("%H:%M:%S UTC")
        );

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("✅ Approve", format!("approve:{}", req.id)),
            InlineKeyboardButton::callback("❌ Deny", format!("deny:{}", req.id)),
        ]]);

        bot.send_message(ChatId(self.chat_id), message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await
            .map_err(|e| LeashError::Backend(format!("Telegram error: {}", e)))?;

        Ok(())
    }
}
