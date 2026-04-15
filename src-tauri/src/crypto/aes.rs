use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

/// Encrypt data using AES-256-GCM
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Invalid key length: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Invalid key length: {}", e))?;

    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes
        let plaintext = b"Hello, VaultKeeper! This is a secret message.";

        let (ciphertext, nonce) = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&ciphertext, &nonce, key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let key = b"0123456789abcdef0123456789abcdef";
        let plaintext = b"Same message";

        let (ct1, _) = encrypt(plaintext, key).unwrap();
        let (ct2, _) = encrypt(plaintext, key).unwrap();

        assert_ne!(ct1, ct2, "Different nonces should produce different ciphertext");
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key = b"0123456789abcdef0123456789abcdef";
        let wrong_key = b"fedcba9876543210fedcba9876543210";
        let plaintext = b"Secret message";

        let (ciphertext, nonce) = encrypt(plaintext, key).unwrap();
        let result = decrypt(&ciphertext, &nonce, wrong_key);

        assert!(result.is_err(), "Decryption with wrong key should fail");
    }
}