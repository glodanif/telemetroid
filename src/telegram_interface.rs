mod command;

use crate::info_collector;
use crate::smart_check::check_task::{prepare_smart_check, smart_check};
use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_result::SmartCheckFailure;
use crate::system_update;
use crate::telegram_interface::command::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::{ChatId, Message, ParseMode, Update};
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, dptree};

pub async fn start_bot() {
    let bot = Bot::from_env();
    let is_smart_check_running = Arc::new(AtomicBool::new(false));

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(answer),
    )
    .dependencies(dptree::deps![is_smart_check_running])
    .build()
    .dispatch()
    .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    is_smart_check_running: Arc<AtomicBool>,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Status => {
            let info = info_collector::collect();
            bot.send_message(msg.chat.id, info.to_string())
                .parse_mode(ParseMode::Html)
                .await?
        }
        Command::Updates => {
            let text = match system_update::check_updates() {
                Ok(updates) if updates.is_empty() => "System is up to date".to_string(),
                Ok(updates) => format!(
                    "<b>Available updates: {}</b>\n\n{}",
                    updates.len(),
                    updates
                        .iter()
                        .map(|u| u.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                Err(e) => format!("Failed to check updates: {}", e),
            };
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?
        }
        Command::SmartCheck => {
            if is_smart_check_running.load(Ordering::Relaxed) {
                bot.send_message(msg.chat.id, "Smart check is already running")
                    .await?;
                return Ok(());
            }
            start_smart_check(bot.clone(), msg.chat.id, is_smart_check_running.clone());
            bot.send_message(
                msg.chat.id,
                "Smart check has started, it may take a couple of minutes per drive",
            )
            .parse_mode(ParseMode::Html)
            .await?
        }
    };
    Ok(())
}

fn start_smart_check(bot: Bot, chat_id: ChatId, is_smart_check_running: Arc<AtomicBool>) {
    tokio::spawn(async move {
        is_smart_check_running.store(true, Ordering::Relaxed);
        let result = prepare_smart_check().await;
        match result {
            Ok(drives) => {
                send_prepare_message(&bot, chat_id, &drives.drives_info).await;
                if !drives.can_proceed {
                    send_message(
                        &bot,
                        chat_id.clone(),
                        "All drives failed to prepare for smart check, no test will be performed",
                    )
                    .await;
                    return;
                } else {
                    let total_duration: f32 = drives
                        .drives_info
                        .iter()
                        .filter_map(|d| d.as_ref().ok())
                        .map(|d| d.ata_smart_data.self_test.polling_minutes.short as f32 * 1.5)
                        .sum();
                    send_message(
                        &bot,
                        chat_id.clone(),
                        format!("Results will be ready in {:.1} minutes", total_duration).as_str(),
                    )
                    .await;
                }
            }
            Err(e) => {
                log::error!("Failed to prepare smart check: {}", e);
                send_message(
                    &bot,
                    chat_id.clone(),
                    format!("Failed to prepare smart check: {}", e).as_str(),
                )
                .await;
                return;
            }
        }

        let check_results = smart_check().await;
        let text = match check_results {
            Ok(results) => format!(
                "<b>Smart check results:</b>\n\n{}",
                results
                    .iter()
                    .map(|r| {
                        match r {
                            Ok(result) => {
                                format!("{}\n", result)
                            }
                            Err(error) => {
                                format!("{}\n", error)
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Err(e) => format!("Failed to perform smart check: {}", e),
        };
        send_message(&bot, chat_id.clone(), text.as_str()).await;
        is_smart_check_running.store(false, Ordering::Relaxed);
    });
}

async fn send_prepare_message(
    bot: &Bot,
    chat_id: ChatId,
    drives_info: &Vec<Result<DriveInfo, SmartCheckFailure>>,
) {
    let mut message = String::new();
    for (i, info) in drives_info.into_iter().enumerate() {
        let text = match info {
            Ok(info) => info.to_string(),
            Err(error) => error.to_string(),
        };
        message.push_str(format!("{}. {}", i + 1, text).as_str());
        message.push('\n');
    }
    send_message(bot, chat_id, message.as_str()).await;
}

async fn send_message(bot: &Bot, chat_id: ChatId, text: &str) {
    if let Err(e) = bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await
    {
        log::error!("Failed to send message: {}", e);
    }
}
