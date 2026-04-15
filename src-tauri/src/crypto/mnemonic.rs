use bip39::Mnemonic;
use rand::RngCore;

/// Generate a random 256-bit seed for BIP39 mnemonic
pub fn generate_seed() -> Vec<u8> {
    let mut seed = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    seed
}

/// Convert a seed to a BIP39 mnemonic phrase (24 words)
pub fn seed_to_mnemonic(seed: &[u8]) -> Result<Vec<String>, String> {
    let mnemonic = Mnemonic::from_entropy(seed)
        .map_err(|e| format!("Failed to create mnemonic: {}", e))?;

    Ok(mnemonic.words().map(|s| s.to_string()).collect())
}

/// Convert a BIP39 mnemonic phrase back to seed
pub fn mnemonic_to_seed(words: &[String]) -> Result<Vec<u8>, String> {
    let phrase = words.join(" ");
    let mnemonic: Mnemonic = phrase.parse()
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;

    Ok(mnemonic.to_entropy())
}

/// Validate a BIP39 mnemonic phrase
pub fn validate_mnemonic(words: &[String]) -> bool {
    let phrase = words.join(" ");
    phrase.parse::<Mnemonic>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_roundtrip() {
        let seed = generate_seed();
        let words = seed_to_mnemonic(&seed).unwrap();
        assert_eq!(words.len(), 24, "Should produce 24 words");

        let recovered_seed = mnemonic_to_seed(&words).unwrap();
        assert_eq!(seed, recovered_seed, "Roundtrip should recover original seed");
    }

    #[test]
    fn test_validate_mnemonic() {
        let seed = generate_seed();
        let words = seed_to_mnemonic(&seed).unwrap();
        assert!(validate_mnemonic(&words));

        let invalid_words = vec!["invalid".to_string(); 24];
        assert!(!validate_mnemonic(&invalid_words));
    }

    #[test]
    fn test_mnemonic_uniqueness() {
        let seed1 = generate_seed();
        let seed2 = generate_seed();
        let words1 = seed_to_mnemonic(&seed1).unwrap();
        let words2 = seed_to_mnemonic(&seed2).unwrap();

        assert_ne!(words1, words2, "Different seeds should produce different mnemonics");
    }
}
