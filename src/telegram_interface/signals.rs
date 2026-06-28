use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::signal::unix::{signal, SignalKind};

use crate::smart_check::self_test_task::SmartTestKind;

/// Spawns a task that listens for the maintenance signals and reports to `admin_chat`:
///   * `SIGUSR1` → weekly short SMART self-test on all drives
///   * `SIGUSR2` → monthly btrfs scrub followed by a long SMART self-test
///
/// Both handlers reserve their slot and detach the actual (long-running) work, so the loop stays
/// responsive and a repeated signal during a run is answered with "already in progress".
pub(super) fn spawn(bot: Bot, admin_chat: ChatId) {
    tokio::spawn(async move {
        let mut sigusr1 = match signal(SignalKind::user_defined1()) {
            Ok(s) => s,
            Err(err) => {
                log::error!("Failed to register SIGUSR1 handler: {}", err);
                return;
            }
        };
        let mut sigusr2 = match signal(SignalKind::user_defined2()) {
            Ok(s) => s,
            Err(err) => {
                log::error!("Failed to register SIGUSR2 handler: {}", err);
                return;
            }
        };

        log::info!("Signal-triggered maintenance enabled (SIGUSR1: short test, SIGUSR2: scrub + long test)");

        loop {
            tokio::select! {
                _ = sigusr1.recv() => {
                    log::info!("Received SIGUSR1: starting short SMART self-test");
                    super::handle_self_test(&bot, admin_chat, SmartTestKind::Short, "short").await;
                }
                _ = sigusr2.recv() => {
                    log::info!("Received SIGUSR2: starting monthly maintenance");
                    super::handle_maintenance(&bot, admin_chat).await;
                }
            }
        }
    });
}
