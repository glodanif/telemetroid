use crate::smart_check::smart_check_error::SmartCheckError;
use std::time::Duration;
use tokio::{task, time};

pub async fn smart_check() -> Result<String, SmartCheckError> {
    let result = task::spawn_blocking(|| {
        let _ = time::sleep(Duration::from_secs(10));
        Ok("Smart check completed".to_string())
    })
    .await
    .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?;
    result
}
