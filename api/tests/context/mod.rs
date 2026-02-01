use api::config::{CheckContentConfig, CommonConfig, Config, Environment, JwtConfig, SmtpConfig};
use api::{App, app::AppBuilder};
use axum_test::TestServer;
use plannify_driver_api_core::application::{DriverRepositories, create_repositories};
use test_context::AsyncTestContext;
use uuid::Uuid;

use super::helpers::auth::generate_mock_token;

pub struct TestContext {
    pub app: App,
    // Router without auth token (will get 401 unauthorized)
    pub unauthenticated_router: TestServer,
    // Router with auth token (authenticated as test user)
    pub authenticated_router: TestServer,
    pub repositories: DriverRepositories,
    pub authenticated_user_id: Uuid,
    pub test_token: String,
}

impl AsyncTestContext for TestContext {
    async fn setup() -> Self {
        let database_url: String =
            "postgres://plannify_user:plannify_password@localhost:5432/plannify_db".to_string();

        let redis_url: String = "redis://localhost:6379/0".to_string();

        let smtp_config = SmtpConfig {
            default_sender: "Plannify <contact@plannify.be>".to_string(),
            default_sender_reply_to: "contact@plannify.be".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
            domain: "localhost".to_string(),
        };

        let jwt: JwtConfig = JwtConfig {
            secret_key: "test-secret-key".to_string(),
            access_ttl: 3600,
            refresh_ttl: 86400,
        };

        let common = CommonConfig {
            api_port: 8080,
            health_port: 8081,
            origins: vec!["0.0.0.0/0".to_string()],
            frontend_url: "http://localhost:3000".to_string(),
            pdf_service_endpoint: "http://localhost:4000".to_string(),
        };

        let check_content = CheckContentConfig {
            email_domain_denylist: vec!["example.fr".to_string(), "example.com".to_string()],
        };

        let config = Config {
            database_url,
            redis_url,
            smtp: smtp_config,
            jwt,
            common,
            check_content,
            environment: Environment::Test,
        };

        let repositories = create_repositories(
            &config.database_url,
            &config.redis_url,
            config.smtp.to_client(),
            config.smtp.to_transport(),
            config.common.frontend_url.clone(),
            true,
            &config.common.pdf_service_endpoint,
        )
        .await
        .expect("Failed to create repositories");

        let app = App::build(config.clone())
            .await
            .expect("Failed to build app")
            .with_state(repositories.clone().into())
            .await
            .expect("Failed to set state");

        // Generate fallback user ID (will be overridden by actual Keycloak user ID if available)
        let fallback_user_id =
            Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").expect("Invalid UUID");

        let test_token = generate_mock_token(&fallback_user_id);

        // Build unauthenticated router (without auth token)
        let unauthenticated_router = TestServer::new(app.app_router()).unwrap();

        // Build authenticated router (with auth token)
        let mut authenticated_router = TestServer::new(app.app_router()).unwrap();
        authenticated_router.add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", test_token),
        );

        TestContext {
            app,
            unauthenticated_router,
            authenticated_router,
            repositories,
            authenticated_user_id: fallback_user_id,
            test_token,
        }
    }

    async fn teardown(self) {
        self.app.shutdown().await;
    }
}

impl TestContext {
    /// Create a new authenticated router with a different user ID
    /// This is useful for testing access control between different users
    /// Note: When using real Keycloak, you'd need to create additional test users
    pub async fn create_authenticated_router_with_different_user(&self) -> TestServer {
        // Try to get token for a different user, or use mock token
        let test_token = generate_mock_token(
            &Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").expect("Invalid UUID"),
        );

        let mut router = TestServer::new(self.app.app_router()).unwrap();
        router.add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", test_token),
        );
        router
    }

    /// Get the current authenticated user's ID
    pub fn authenticated_user_id(&self) -> Uuid {
        self.authenticated_user_id
    }

    /// Get the current test token
    pub fn test_token(&self) -> &str {
        &self.test_token
    }
}
