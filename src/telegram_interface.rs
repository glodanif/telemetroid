mod command;

use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::scrub_task::run_btrfs_scrub;
use crate::info_collector;
use crate::smart_check::self_test_task::{run_long_self_test, run_short_self_test};
use crate::smart_check::smart_check::get_drives_info;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::system_update;
use crate::telegram_interface::command::Command;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::{ChatId, Message, ParseMode, Update};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

pub async fn start_bot() {
    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(answer),
    )
    .build()
    .dispatch()
    .await;
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
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
            match get_drives_info() {
                Ok(drives) => {
                    send_message(&bot, msg.chat.id.clone(), drives.to_string().as_str()).await;
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
        Command::SmartTestShort => {
            run_self_test(&bot, msg.chat.id, "short", run_short_self_test().await).await;
            return Ok(());
        }
        Command::SmartTestLong => {
            run_self_test(&bot, msg.chat.id, "long", run_long_self_test().await).await;
            return Ok(());
        }
        Command::Scrub => {
            match run_btrfs_scrub().await {
                Ok(report) => {
                    send_message(
                        &bot,
                        msg.chat.id,
                        format!("<b>Btrfs scrub result:</b>\n\n{}", report).as_str(),
                    )
                    .await;
                }
                Err(BtrfsError::AlreadyRunningError()) => {
                    send_message(&bot, msg.chat.id, "⏳ A btrfs scrub is already in progress").await;
                }
                Err(err) => {
                    send_message(
                        &bot,
                        msg.chat.id,
                        format!("Failed to run scrub: {}", err).as_str(),
                    )
                    .await;
                }
            }
            return Ok(());
        }
    };
    Ok(())
}

async fn run_self_test(
    bot: &Bot,
    chat_id: ChatId,
    kind: &str,
    result: Result<crate::smart_check::self_test_report::SelfTestReport, SmartCheckError>,
) {
    match result {
        Ok(report) => {
            send_message(
                bot,
                chat_id,
                format!("<b>SMART {} self-test result:</b>\n\n{}", kind, report).as_str(),
            )
            .await;
        }
        Err(SmartCheckError::AlreadyRunningError()) => {
            send_message(bot, chat_id, "⏳ A SMART self-test is already in progress").await;
        }
        Err(err) => {
            send_message(
                bot,
                chat_id,
                format!("Failed to run {} self-test: {}", kind, err).as_str(),
            )
            .await;
        }
    }
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
