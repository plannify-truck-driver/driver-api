use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::JwtConfig, http::common::api_error::ApiError};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn is_expired(&self) -> bool {
        self.exp < Utc::now().timestamp()
    }
}

#[derive(Clone)]
pub struct AuthValidator {
    secret_key: String,
    access_ttl: String,
    refresh_ttl: String,
}

impl AuthValidator {
    pub fn new(jwt_config: &JwtConfig) -> Self {
        Self {
            secret_key: jwt_config.secret_key.clone(),
            access_ttl: jwt_config.access_ttl.clone(),
            refresh_ttl: jwt_config.refresh_ttl.clone(),
        }
    }
}

pub trait TokenValidator: Send + Sync {
    fn create_tokens(&self, driver_id: Uuid) -> Result<(String, String), ApiError>;
    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError>;
}

impl TokenValidator for AuthValidator {
    fn create_tokens(&self, driver_id: Uuid) -> Result<(String, String), ApiError> {
        let now = Utc::now().timestamp();

        let access_exp = now + self.access_ttl.parse::<i64>().map_err(|_| ApiError::InternalServerError)?;
        let refresh_exp = now + self.refresh_ttl.parse::<i64>().map_err(|_| ApiError::InternalServerError)?;

        let access_claims = Claims {
            sub: driver_id,
            exp: access_exp,
            iat: now,
        };

        let refresh_claims = Claims {
            sub: driver_id,
            exp: refresh_exp,
            iat: now,
        };

        let access_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &access_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|_| ApiError::InternalServerError)?;

        let refresh_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &refresh_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|_| ApiError::InternalServerError)?;

        Ok((access_token, refresh_token))
    }

    fn validate_token(&self, token: &str) -> Result<UserIdentity, ApiError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ApiError::Unauthorized { error_code: "UNAUTHORIZED".to_string() })?;

        let claims = token_data.claims;

        if claims.is_expired() {
            return Err(ApiError::Unauthorized { error_code: "TOKEN_EXPIRED".to_string() });
        }

        Ok(UserIdentity {
            user_id: claims.sub,
        })
    }
}
