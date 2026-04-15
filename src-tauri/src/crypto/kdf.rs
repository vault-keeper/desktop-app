use argon2::{Argon2, Algorithm, Version, Params};
use rand::RngCore;

/// Generate a random salt for key derivation
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derive a 256-bit key from a password using Argon2id
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<Vec<u8>, String> {
    let params = Params::new(
        65536,     // 64 MB memory
        3,         // 3 iterations
        4,         // 4 parallelism
        Some(32),  // 32 bytes output (256-bit)
    ).map_err(|e| format!("Invalid Argon2 params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = vec![0u8; 32];
    argon2
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| format!("Argon2id hashing failed: {}", e))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_consistency() {
        let password = b"test_password_123";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(key1, key2, "Same password and salt should produce same key");
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = generate_salt();

        let key1 = derive_key(b"password1", &salt).unwrap();
        let key2 = derive_key(b"password2", &salt).unwrap();

        assert_ne!(key1, key2, "Different passwords should produce different keys");
    }

    #[test]
    fn test_derive_key_different_salts() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        let key1 = derive_key(b"same_password", &salt1).unwrap();
        let key2 = derive_key(b"same_password", &salt2).unwrap();

        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    #[test]
    fn test_key_length() {
        let salt = generate_salt();
        let key = derive_key(b"test", &salt).unwrap();
        assert_eq!(key.len(), 32, "Key should be 32 bytes (256-bit)");
    }
}