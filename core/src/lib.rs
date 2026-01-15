pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types for convenience
pub use domain::common::services::Service;
pub use infrastructure::health::repositories::postgres::PostgresHealthRepository;
