use std::sync::Arc;

use axum::{
    extract::FromRef,
    http::StatusCode,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::AppState;

/// Authentication token generated on server start
#[derive(Clone, Debug)]
pub struct AuthToken(String);

impl AuthToken {
    const AUTH_TOKEN_LENGTH: usize = 16;

    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; Self::AUTH_TOKEN_LENGTH];
        rand::rng().fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        Self(token)
    }

    /// Get the token string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validate the authentication token from Authorization Bearer header
pub fn validate_token<S>(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    state: &S,
) -> Result<(), StatusCode>
where
    Arc<AppState>: FromRef<S>,
{
    let app_state: Arc<AppState> = Arc::<AppState>::from_ref(state);

    if let Some(TypedHeader(Authorization(bearer))) = auth_header {
        if bearer.token() == app_state.auth_token.as_str() {
            return Ok(());
        }
    }

    tracing::warn!("unauthorized access attempt");
    Err(StatusCode::UNAUTHORIZED)
}
