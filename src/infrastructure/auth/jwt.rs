use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::application::services::auth::TokenIssuer;
use crate::domain::errors::DomainError;

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

const TOKEN_TTL_SECS: u64 = 86_400; // 24 h

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
}

pub struct JwtIssuer {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtIssuer {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn verify(&self, token: &str) -> Result<Claims, DomainError> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::new(Algorithm::HS256))
            .map_err(|_| DomainError::InvalidCredentials)?;
        Ok(data.claims)
    }
}

impl TokenIssuer for JwtIssuer {
    fn issue(&self, user_id: &str, role: &str) -> Result<String, DomainError> {
        let exp = current_timestamp() + TOKEN_TTL_SECS;
        let claims = Claims {
            sub: user_id.to_owned(),
            role: role.to_owned(),
            exp,
        };
        encode(&Header::default(), &claims, &self.encoding)
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-key";

    #[test]
    fn issue_and_verify_roundtrip() {
        let issuer = JwtIssuer::new(SECRET);
        let token = issuer.issue("user-123", "admin").unwrap();
        let claims = issuer.verify(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn claims_exp_is_approximately_24h_from_now() {
        let issuer = JwtIssuer::new(SECRET);
        let token = issuer.issue("u", "r").unwrap();
        let claims = issuer.verify(&token).unwrap();
        let now = current_timestamp();
        let diff = claims.exp.saturating_sub(now);
        assert!(diff > TOKEN_TTL_SECS - 5 && diff <= TOKEN_TTL_SECS);
    }

    #[test]
    fn verify_rejects_invalid_token() {
        let issuer = JwtIssuer::new(SECRET);
        let err = issuer.verify("not.a.jwt.token").unwrap_err();
        assert!(matches!(err, DomainError::InvalidCredentials));
    }

    #[test]
    fn verify_rejects_token_signed_with_different_secret() {
        let issuer_a = JwtIssuer::new("secret-a");
        let issuer_b = JwtIssuer::new("secret-b");
        let token = issuer_a.issue("user-1", "regular").unwrap();
        let err = issuer_b.verify(&token).unwrap_err();
        assert!(matches!(err, DomainError::InvalidCredentials));
    }
}
