use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};
use teloxide::prelude::*;
use uuid::Uuid;
use openleash_core::models::{TaskStatus, ResourceType};
use openleash_backend::SecretBackend;
use crate::OpenLeashDaemon;

pub async fn run_background_worker(daemon: Arc<OpenLeashDaemon>) {
    loop {
        sleep(TokioDuration::from_secs(30)).await;
        
        match daemon.db.get_expired_tasks().await {
            Ok(expired_tasks) => {
                for task in expired_tasks {
                    tracing::info!(task_id = %task.id, name = %task.name, "Auto-cleaning expired task");
                    if let Err(e) = daemon.cleanup_task(&task.id, TaskStatus::Expired).await {
                        tracing::error!(task_id = %task.id, error = %e, "Failed to auto-cleanup task");
                    }
                }
            }
            Err(e) => tracing::error!(error = %e, "Failed to query expired tasks"),
        }

        match daemon.db.get_expired_leases().await {
            Ok(expired_leases) => {
                for lease in expired_leases {
                    if let Err(e) = daemon.cleanup_lease(&lease).await {
                        tracing::error!(lease_id = %lease.id, error = %e, "Failed to auto-cleanup lease");
                    }
                }
            }
            Err(e) => tracing::error!(error = %e, "Failed to query expired leases"),
        }
    }
}

pub async fn run_telegram_bot(token: String, daemon: Arc<OpenLeashDaemon>) {
    let bot = Bot::new(token);

    let handler = Update::filter_callback_query().endpoint(
        |bot: Bot, q: CallbackQuery, daemon: Arc<OpenLeashDaemon>| async move {
            if let Some(data) = q.data {
                let parts: Vec<&str> = data.split(':').collect();
                if parts.len() == 2 {
                    let action = parts[0];
                    let approval_id = match Uuid::parse_str(parts[1]) {
                        Ok(id) => id,
                        Err(_) => return respond(()),
                    };

                    let status = match action {
                        "approve" => "Approved",
                        "deny" => "Denied",
                        _ => return respond(()),
                    };

                    if let Err(e) = daemon.db.update_approval_status(&approval_id, status, None).await {
                        tracing::error!(error = %e, "Failed to update approval status from telegram");
                        let _ = bot.answer_callback_query(q.id).text("Error updating status").await;
                    } else {
                        // Proactive unlock if this was a keychain unlock request
                        if status == "Approved" {
                            if let Ok(Some(req)) = daemon.db.get_approval_by_id(&approval_id).await {
                                if req.resource_type == ResourceType::System && req.resource_id == "keychain-unlock" {
                                    let password = std::env::var("OPENLEASH_KEYCHAIN_PASSWORD").ok();
                                    if let Err(e) = daemon.keychain_backend.unlock(password.as_deref()).await {
                                        tracing::error!(error = %e, "Failed to unlock keychain after approval");
                                        let _ = bot.send_message(q.from.id, format!("❌ Approved, but unlock failed: {}", e)).await;
                                    } else {
                                        tracing::info!("Keychain unlocked via Telegram approval");
                                    }
                                }
                            }
                        }

                        let _ = bot.answer_callback_query(q.id).text(format!("Request {}", status)).await;
                        if let Some(msg) = q.message {
                            let _ = bot.edit_message_text(msg.chat.id, msg.id, format!("✅ Request {}", status)).await;
                        }
                    }
                }
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![daemon])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
