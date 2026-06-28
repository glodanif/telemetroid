mod command;

use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::scrub_task::ScrubRun;
use crate::info_collector;
use crate::smart_check::self_test_task::{SelfTestRun, SmartTestKind};
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
            handle_self_test(&bot, msg.chat.id, SmartTestKind::Short, "short").await;
            return Ok(());
        }
        Command::SmartTestLong => {
            handle_self_test(&bot, msg.chat.id, SmartTestKind::Long, "long").await;
            return Ok(());
        }
        Command::Scrub => {
            handle_scrub(&bot, msg.chat.id).await;
            return Ok(());
        }
    };
    Ok(())
}

async fn handle_self_test(bot: &Bot, chat_id: ChatId, kind: SmartTestKind, label: &'static str) {
    // Reserve the slot up front so we can give immediate, accurate feedback.
    let run = match SelfTestRun::begin() {
        Ok(run) => run,
        Err(SmartCheckError::AlreadyRunningError()) => {
            send_message(bot, chat_id, "⏳ A SMART self-test is already in progress").await;
            return;
        }
        Err(err) => {
            send_message(
                bot,
                chat_id,
                format!("Failed to start {} self-test: {}", label, err).as_str(),
            )
            .await;
            return;
        }
    };

    send_message(
        bot,
        chat_id,
        format!(
            "▶️ Started {} SMART self-test on all drives. I'll report back when it finishes.",
            label
        )
        .as_str(),
    )
    .await;

    // Detach the (potentially hours-long) run so the bot stays responsive to other commands.
    let bot = bot.clone();
    tokio::spawn(async move {
        let text = match run.execute(kind).await {
            Ok(report) => format!("<b>SMART {} self-test result:</b>\n\n{}", label, report),
            Err(err) => format!("Failed to run {} self-test: {}", label, err),
        };
        send_message(&bot, chat_id, text.as_str()).await;
    });
}

async fn handle_scrub(bot: &Bot, chat_id: ChatId) {
    // Reserve the slot up front so we can give immediate, accurate feedback.
    let run = match ScrubRun::begin() {
        Ok(run) => run,
        Err(BtrfsError::AlreadyRunningError()) => {
            send_message(bot, chat_id, "⏳ A btrfs scrub is already in progress").await;
            return;
        }
        Err(err) => {
            send_message(bot, chat_id, format!("Failed to start scrub: {}", err).as_str()).await;
            return;
        }
    };

    send_message(
        bot,
        chat_id,
        "▶️ Started btrfs scrub on all mounted filesystems. I'll report back when it finishes.",
    )
    .await;

    // Detach the (potentially hours-long) run so the bot stays responsive to other commands.
    let bot = bot.clone();
    tokio::spawn(async move {
        let text = match run.execute().await {
            Ok(report) => format!("<b>Btrfs scrub result:</b>\n\n{}", report),
            Err(err) => format!("Failed to run scrub: {}", err),
        };
        send_message(&bot, chat_id, text.as_str()).await;
    });
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
