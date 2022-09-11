use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct TodoResponse {
    pub content: String,
    pub owner: String,
    pub created_at: DateTime<Utc>,
}
