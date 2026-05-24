use crate::info_collector::linux_info_source;

mod telegram_interface;
pub mod info_collector;

//use crate::telegram_interface::start_bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    //start_bot().await;
    let info = linux_info_source::collect_info();
    println!("{}", info);
}
