use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize)]
pub struct VapidPublicKeyResponse {
    pub public_key: String,
}
