use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Password {
    pub(crate) hashed_password: String,
}

impl Password {
    pub fn new(password: &str) -> Password {
        let hashed_password = Password::hash_password(password);
        Password { hashed_password }
    }

    pub fn set_password(&mut self, new_password: &str) {
        self.hashed_password = Password::hash_password(new_password);
    }

    pub fn check_password(&self, password: &str) -> bool {
        Password::hash_password(password) == self.hashed_password
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }
}
