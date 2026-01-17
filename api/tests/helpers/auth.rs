use api::http::common::middleware::auth::entities::{AccessClaims, DriverClaims};
use jsonwebtoken::Algorithm;
use uuid::Uuid;

/// Generate a simple test JWT token for local testing without Keycloak
/// WARNING: This will NOT work with real Keycloak validation
/// Use this only for unit tests that mock the auth repository
pub fn generate_mock_token(user_id: &Uuid) -> String {
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};

    let now = Utc::now().timestamp();
    let claims = AccessClaims {
        sub: *user_id,
        driver: DriverClaims {
            id: *user_id,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test.user@example.com".to_string(),
            verified: true,
        },
        exp: now + 3600, // Token expires in 1 hour
        iat: now,
    };

    encode(
        &Header {
            alg: Algorithm::HS256,
            ..Default::default()
        },
        &claims,
        &EncodingKey::from_secret("test-secret-key".as_ref()),
    )
    .expect("Failed to generate mock token")
}
