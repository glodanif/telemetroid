mod command;
use crate::telegram_interface::command::Command;
use teloxide::Bot;
use teloxide::repls::CommandReplExt;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::Message;
use teloxide::utils::command::BotCommands;
use crate::os;

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
            let os_info = os::collect_os_info();
            bot.send_message(msg.chat.id, os_info.to_string()).await?
        }
    };
    Ok(())
}
