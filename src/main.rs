pub mod info_collector;
pub mod smart_check;
mod system_update;
mod telegram_interface;

use crate::telegram_interface::start_bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    log::info!("Starting telemetroid...");
    start_bot().await;
}
