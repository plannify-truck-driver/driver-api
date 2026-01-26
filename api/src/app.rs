use axum::{
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware::{from_extractor_with_state, from_fn},
};
use plannify_driver_api_core::{application::create_repositories, domain::common::CoreError};
use tower_http::cors::CorsLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    ApiError, AppState, AuthMiddleware, AuthValidator,
    config::{Config, Environment},
    health_routes,
    http::{
        authentication::routes::authentication_routes,
        common::middleware::tracing::tracing_middleware, driver::routes::driver_routes,
        workday::routes::workday_routes,
    },
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Plannify Driver openapi",
        contact(name = "contact@plannify.be"),
        description = "API documentation for the Plannify Driver API",
        version = "0.1.0"
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api-dev.plannify.be/driver/v1", description = "Development server"),
        (url = "https://api.plannify.be/driver/v1", description = "Production server")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;
pub struct App {
    config: Config,
    pub state: AppState,
    pub auth_validator: AuthValidator,
    app_router: axum::Router,
    health_router: axum::Router,
}

impl App {
    pub async fn new(config: Config) -> Result<Self, ApiError> {
        let mut state: AppState = create_repositories(
            &config.database_url,
            &config.redis_url,
            config.smtp.to_client(),
            config.smtp.to_transport(),
            config.common.frontend_url.clone(),
            matches!(config.environment, Environment::Test),
        )
        .await
        .map_err(|e| ApiError::StartupError {
            msg: format!("Failed to create repositories: {}", e),
        })?
        .into();
        state.config = config.clone();

        let cors_origins = config
            .common
            .origins
            .iter()
            .map(|origin| {
                origin
                    .parse::<HeaderValue>()
                    .map_err(|e| CoreError::CorsBindingError {
                        message: format!("{}", e),
                    })
            })
            .collect::<Result<Vec<HeaderValue>, CoreError>>()?;

        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_origin(cors_origins)
            .allow_credentials(true)
            .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

        let auth_validator = AuthValidator::new(&config.clone().jwt);
        state.auth_validator = auth_validator.clone();

        let (app_router, mut api) = OpenApiRouter::<AppState>::new()
            .merge(driver_routes())
            .merge(workday_routes())
            .route_layer(from_extractor_with_state::<AuthMiddleware, AuthValidator>(
                auth_validator.clone(),
            ))
            .merge(authentication_routes())
            .layer(cors)
            .split_for_parts();

        // Override API documentation info
        let custom_info = ApiDoc::openapi();
        api.info = custom_info.info;

        let openapi_json = api.to_pretty_json().map_err(|e| ApiError::StartupError {
            msg: format!("Failed to generate OpenAPI spec: {}", e),
        })?;

        let jwt_secret = config.jwt.secret_key.clone();
        let jwt_secret_for_app = jwt_secret.clone();
        let app_router = app_router
            .with_state(state.clone())
            .merge(Scalar::with_url("/doc", api))
            .layer(from_fn(move |request, next| {
                let secret = jwt_secret_for_app.clone();
                tracing_middleware(request, next, secret)
            }));

        // Write OpenAPI spec to file in development environment
        if matches!(config.environment, crate::config::Environment::Development) {
            std::fs::write("openapi.json", &openapi_json).map_err(|e| ApiError::StartupError {
                msg: format!("Failed to write OpenAPI spec to file: {}", e),
            })?;
        }

        let health_router = axum::Router::new()
            .merge(health_routes())
            .with_state(state.clone())
            .layer(from_fn(move |request, next| {
                let secret = jwt_secret.clone();
                tracing_middleware(request, next, secret)
            }));

        Ok(Self {
            config,
            state,
            auth_validator,
            app_router,
            health_router,
        })
    }

    pub fn app_router(&self) -> axum::Router {
        self.app_router.clone()
    }

    pub async fn start(&self) -> Result<(), ApiError> {
        let health_addr = format!("0.0.0.0:{}", self.config.clone().common.health_port);
        let api_addr = format!("0.0.0.0:{}", self.config.clone().common.api_port);
        // Create TCP listeners for both servers
        let health_listener = tokio::net::TcpListener::bind(&health_addr)
            .await
            .map_err(|_| ApiError::StartupError {
                msg: format!("Failed to bind health server: {}", health_addr),
            })?;
        let api_listener = tokio::net::TcpListener::bind(&api_addr)
            .await
            .map_err(|_| ApiError::StartupError {
                msg: format!("Failed to bind API server: {}", api_addr),
            })?;

        info!(
            "Starting driver API server ({}) and health server ({})",
            api_addr, health_addr
        );

        // Run both servers concurrently
        tokio::try_join!(
            axum::serve(health_listener, self.health_router.clone()),
            axum::serve(api_listener, self.app_router.clone())
        )
        .expect("Failed to start servers");

        Ok(())
    }

    pub async fn shutdown(&self) {
        self.state.shutdown().await;
    }
}

pub trait AppBuilder {
    fn build(config: Config) -> impl Future<Output = Result<App, ApiError>>;
    fn with_state(self, state: AppState) -> impl Future<Output = Result<App, ApiError>>;
}

impl AppBuilder for App {
    async fn build(config: Config) -> Result<App, ApiError> {
        App::new(config).await
    }

    async fn with_state(mut self, state: AppState) -> Result<App, ApiError> {
        self.state = state;
        Ok(self)
    }
}

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT Bearer token"))
                        .build(),
                ),
            )
        }
    }
}
