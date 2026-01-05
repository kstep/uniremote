use axum::{
    extract::{FromRef, Query},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

/// Authentication token generated on server start
#[derive(Clone, Debug)]
pub struct AuthToken(String);

impl AuthToken {
    /// Generate a new random authentication token (64 bytes)
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        Self(token)
    }

    /// Get the token string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Query parameter extractor for token validation
#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

/// Validate the authentication token from query parameters
pub fn validate_token<S>(query: &Query<TokenQuery>, state: &S) -> Result<(), StatusCode>
where
    Arc<crate::AppState>: FromRef<S>,
{
    let app_state: Arc<crate::AppState> = Arc::<crate::AppState>::from_ref(state);
    
    if query.token.as_ref().is_some_and(|token| token == app_state.auth_token.as_str()) {
        Ok(())
    } else {
        tracing::warn!("unauthorized access attempt");
        Err(StatusCode::FORBIDDEN)
    }
}
