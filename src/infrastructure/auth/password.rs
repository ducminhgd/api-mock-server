use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::application::services::users::PasswordHasher;
use crate::domain::errors::DomainError;

pub struct BcryptHasher {
    cost: u32,
}

impl BcryptHasher {
    pub fn new(cost: u32) -> Self {
        Self { cost }
    }
}

impl Default for BcryptHasher {
    fn default() -> Self {
        Self { cost: bcrypt::DEFAULT_COST }
    }
}

impl PasswordHasher for BcryptHasher {
    fn hash(&self, password: &str) -> Result<String, DomainError> {
        bcrypt::hash(password, self.cost)
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError> {
        bcrypt::verify(password, hash)
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn generate_random(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect()
    }
}
