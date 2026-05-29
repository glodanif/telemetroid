use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Status info")]
    Status,
    #[command(description = "Check for available package updates")]
    Updates,
    #[command(description = "Smartools check")]
    SmartCheck,
}
