use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

// 12-byte nonce for AES-GCM, prepended to ciphertext
const NONCE_LEN: usize = 12;
// Deterministic key derivation — production would use Argon2/scrypt,
// but for MVP we hash the passphrase with a simple expansion
const KEY_LEN: usize = 32;

fn derive_key(passphrase: &str) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    let bytes = passphrase.as_bytes();
    for (i, slot) in key.iter_mut().enumerate() {
        *slot = bytes[i % bytes.len()].wrapping_add(i as u8);
    }
    key
}

pub fn encrypt(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| e.to_string())?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt(passphrase: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < NONCE_LEN {
        return Err("Corrupted vault data".into());
    }

    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
    let plaintext = cipher
        .decrypt(nonce, &data[NONCE_LEN..])
        .map_err(|_| "Decryption failed — wrong passphrase or corrupted data".to_string())?;

    Ok(plaintext)
}
