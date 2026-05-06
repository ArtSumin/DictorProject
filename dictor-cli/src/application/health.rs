// application/health.rs — Use Case: Server health check

use crate::domain::models::HealthStatus;
use crate::domain::traits::{HealthCheck, SttError};

/// Use Case: Check STT server status.
pub struct HealthCheckUseCase {
    checker: Box<dyn HealthCheck>,
}

impl HealthCheckUseCase {
    /// Creates new Use Case with injected `HealthCheck` dependency.
    pub fn new(checker: Box<dyn HealthCheck>) -> Self {
        Self { checker }
    }

    /// Performs server health check.
    /// Returns Ok(HealthStatus) if the request succeeds.
    pub fn execute(&self) -> Result<HealthStatus, SttError> {
        self.checker.check_health()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::MockHealthCheck;

    // ============================================================
    // Test: Successful health check
    // ============================================================
    #[test]
    fn test_execute_success() {
        let mut mock = MockHealthCheck::new();
        
        mock.expect_check_health()
            .times(1)
            .returning(|| {
                Ok(HealthStatus {
                    status: "ok".to_string(),
                    model_loaded: true,
                })
            });

        let use_case = HealthCheckUseCase::new(Box::new(mock));
        let result = use_case.execute();

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.is_ready());
    }

    // ============================================================
    // Test: Health check failure
    // ============================================================
    #[test]
    fn test_execute_failure() {
        let mut mock = MockHealthCheck::new();
        
        mock.expect_check_health()
            .times(1)
            .returning(|| Err(SttError::ServerError(500)));

        let use_case = HealthCheckUseCase::new(Box::new(mock));
        let result = use_case.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ServerError(code) => assert_eq!(code, 500),
            _ => panic!("Expected ServerError 500"),
        }
    }
}
