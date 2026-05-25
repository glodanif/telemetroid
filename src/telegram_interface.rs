mod command;
use crate::telegram_interface::command::Command;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::repls::CommandReplExt;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::{Message, ParseMode};
use teloxide::utils::command::BotCommands;
use crate::info_collector;

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
        Command::Os => {
            let info = info_collector::collect();
            bot.send_message(msg.chat.id, info.to_string())
                .parse_mode(ParseMode::Html)
                .await?
        }
    };
    Ok(())
}
