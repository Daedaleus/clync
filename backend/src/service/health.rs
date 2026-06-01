use crate::dto::health::HealthResponse;

pub fn get_health() -> HealthResponse {
    HealthResponse { status: "ok" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_health_returns_ok_status() {
        assert_eq!(get_health().status, "ok");
    }
}
