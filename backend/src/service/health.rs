use crate::dto::health::HealthResponse;

pub fn get_health() -> HealthResponse {
    HealthResponse { status: "ok" }
}
