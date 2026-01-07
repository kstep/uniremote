use core::fmt;

use axum::http::StatusCode;

/// Authentication token generated on server start
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Validate a token string against this token
    pub fn validate(&self, token: &str) -> Result<(), StatusCode> {
        if token == self.0 {
            Ok(())
        } else {
            tracing::warn!("unauthorized access attempt with invalid token");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_generation() {
        let token1 = AuthToken::generate();
        let token2 = AuthToken::generate();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Token should be hex-encoded (32 chars for 16 bytes)
        assert_eq!(token1.0.len(), 32);
        assert!(token1.0.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
