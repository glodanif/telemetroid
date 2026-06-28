pub mod info_collector;
pub mod smart_check;
mod maintenance;
mod system_update;
mod telegram_interface;
pub mod text_utils;
pub mod command_runner;
pub mod btrfs;

use crate::telegram_interface::start_bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    log::info!("Starting telemetroid...");
    start_bot().await;
}
