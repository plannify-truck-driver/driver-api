use chrono::Utc;
use plannify_driver_api_core::domain::driver::entities::DriverRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tracing::error;

use crate::{config::JwtConfig, http::common::api_error::ApiError};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverClaims {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub driver: DriverClaims,
    pub exp: i64,
    pub iat: i64,
}

impl AccessClaims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

impl RefreshClaims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}

#[derive(Clone)]
pub struct AuthValidator {
    secret_key: String,
    access_ttl: u64,
    refresh_ttl: u64,
}

impl AuthValidator {
    pub fn new(jwt_config: &JwtConfig) -> Self {
        Self {
            secret_key: jwt_config.secret_key.clone(),
            access_ttl: jwt_config.access_ttl,
            refresh_ttl: jwt_config.refresh_ttl,
        }
    }
}

pub trait TokenValidator: Send + Sync {
    fn create_tokens(&self, driver: &DriverRow) -> Result<(String, String), ApiError>;
    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError>;
}

impl TokenValidator for AuthValidator {
    fn create_tokens(&self, driver: &DriverRow) -> Result<(String, String), ApiError> {
        let now = Utc::now().timestamp();

        let access_exp = now + self.access_ttl as i64;
        let refresh_exp = now + self.refresh_ttl as i64;

        let access_claims = AccessClaims {
            sub: driver.pk_driver_id,
            driver: DriverClaims {
                id: driver.pk_driver_id,
                first_name: driver.firstname.clone(),
                last_name: driver.lastname.clone(),
                email: driver.email.clone(),
                verified: driver.verified_at.is_some(),
            },
            exp: access_exp,
            iat: now,
        };

        let refresh_claims = RefreshClaims {
            sub: driver.pk_driver_id,
            exp: refresh_exp,
            iat: now,
        };

        let access_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &access_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|e| {
            error!(
                "Failed to create access token for user {}: {:?}",
                driver.pk_driver_id, e
            );
            ApiError::InternalServerError
        })?;

        let refresh_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &refresh_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|e| {
            error!(
                "Failed to create refresh token for user {}: {:?}",
                driver.pk_driver_id, e
            );
            ApiError::InternalServerError
        })?;

        Ok((access_token, refresh_token))
    }

    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError> {
        let token_data = decode::<AccessClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| {
            error!("Failed to decode token: {:?}", e);
            ApiError::Unauthorized {
                error_code: "UNAUTHORIZED".to_string(),
            }
        })?;

        let claims = token_data.claims;

        if claims.is_expired() {
            return Err(ApiError::Unauthorized {
                error_code: "TOKEN_EXPIRED".to_string(),
            });
        }

        if !claims.driver.verified {
            return Err(ApiError::Unauthorized {
                error_code: "DRIVER_NOT_VERIFIED".to_string(),
            });
        }

        Ok(UserIdentity {
            user_id: claims.sub,
        })
    }
}
