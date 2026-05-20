#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub workday_garbage_retention_days: i64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            workday_garbage_retention_days: 30,
        }
    }
}
