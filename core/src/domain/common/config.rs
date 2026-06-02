#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub workday_garbage_retention_days: i64,
    pub account_deactivation_days: i64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            workday_garbage_retention_days: 30,
            account_deactivation_days: 30,
        }
    }
}
