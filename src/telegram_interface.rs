mod command;

use crate::info_collector;
use crate::smart_check::check_task::{prepare_smart_check, smart_check};
use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_result::SmartCheckFailure;
use crate::system_update;
use crate::telegram_interface::command::Command;
use crate::text_utils::format_number;
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

            let result = prepare_smart_check().await;
            match result {
                Ok(drives_info) => {
                    let drives_number = drives_info.len();
                    let mut failed_preparations = 0;
                    for r in &drives_info {
                        if r.is_err() {
                            failed_preparations += 1;
                        }
                    }
                    let can_proceed = failed_preparations < drives_number;
                    send_prepare_message(&bot, msg.chat.id, &drives_info, can_proceed).await;
                    if can_proceed {
                        let drives: Vec<DriveInfo> =
                            drives_info.into_iter().filter_map(Result::ok).collect();
                        start_smart_check(
                            bot.clone(),
                            msg.chat.id,
                            is_smart_check_running.clone(),
                            drives,
                        );
                    }
                }
                Err(err) => {
                    send_message(
                        &bot,
                        msg.chat.id,
                        format!("Failed to prepare smart check: {}", err).as_str(),
                    )
                    .await;
                }
            }
            return Ok(());
        }
    };
    Ok(())
}

fn start_smart_check(
    bot: Bot,
    chat_id: ChatId,
    is_smart_check_running: Arc<AtomicBool>,
    drives: Vec<DriveInfo>,
) {
    tokio::spawn(async move {
        is_smart_check_running.store(true, Ordering::Relaxed);
        let check_results = smart_check(drives).await;
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
    can_proceed: bool,
) {
    let mut message = String::new();
    for (i, info) in drives_info.into_iter().enumerate() {
        let text = match info {
            Ok(info) => info.to_string(),
            Err(error) => error.to_string(),
        };
        message.push_str(format!("{}. {}", i + 1, text).as_str());
        message.push_str("\n\n");
    }
    if !can_proceed {
        message.push_str("All drives failed to prepare for smart check, no test will be performed");
    } else {
        let total_duration: f32 = drives_info
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|info| info.get_time_to_test())
            .sum();
        message.push_str(
            format!(
                "Smart check has started, results will be ready in {:.1} min",
                format_number(total_duration)
            )
            .as_str(),
        );
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
