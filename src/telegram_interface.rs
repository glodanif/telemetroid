mod command;
use crate::info_collector;
use crate::system_update;
use crate::telegram_interface::command::Command;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::repls::CommandReplExt;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::{Message, ParseMode};
use teloxide::utils::command::BotCommands;

pub async fn start_bot() {
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
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
                Ok(updates) => updates
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => format!("Failed to check updates: {}", e),
            };
            bot.send_message(msg.chat.id, text).await?
        }
    };
    Ok(())
}
