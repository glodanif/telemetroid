mod telegram_interface;
pub mod os;

use crate::telegram_interface::start_bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    start_bot().await;
}
