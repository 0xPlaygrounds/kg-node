use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "$type")]
pub struct Cursor {
    pub cursor: String,
    pub block_number: u64,
}

#[derive(Default)]
pub struct BlockMetadata {
    pub cursor: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}