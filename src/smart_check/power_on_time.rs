use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerOnTimeResult {
    pub hours: u32,
}
