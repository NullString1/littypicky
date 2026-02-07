use base64::{engine::general_purpose, Engine};
use rand::Rng;

/// Generate a secure random token for email verification or password reset
#[must_use]
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes)
}

/// Hash a token for secure storage (simple SHA256)
#[must_use]
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
