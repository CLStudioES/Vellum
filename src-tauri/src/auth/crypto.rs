use argon2::{self, Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

const VAULT_KEY_LEN: usize = 32;

/// Hash a master password for storage (verification on login).
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Verify a password against a stored hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = argon2::PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}

/// Derive a 32-byte vault encryption key from the master password + user salt.
/// This key never touches disk — lives only in the session.
pub fn derive_vault_key(password: &str, salt: &[u8]) -> Result<[u8; VAULT_KEY_LEN], String> {
    let mut key = [0u8; VAULT_KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| e.to_string())?;
    Ok(key)
}

/// Generate a random salt for vault key derivation (stored with user profile).
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);
    salt
}
