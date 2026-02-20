use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

pub struct Crypt {
    cipher: Aes256Gcm,
}

impl Crypt {
    /// Create a new Crypt instance with the given key (32 bytes, possibly base64 encoded with "base64:" prefix)
    pub fn new(key_str: &str) -> Result<Self> {
        let key_bytes = if let Some(stripped) = key_str.strip_prefix("base64:") {
            general_purpose::STANDARD
                .decode(stripped)
                .context("Failed to decode base64 APP_KEY")?
        } else {
            key_str.as_bytes().to_vec()
        };

        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "APP_KEY must be exactly 32 bytes (got {})",
                key_bytes.len()
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        Ok(Self { cipher })
    }

    /// Encrypt plaintext to a base64 encoded string containing (Nonce + Ciphertext + Tag)
    /// AES-GCM output is Ciphertext + Tag automatically by the crate `encrypt` function usually?
    /// Actually `aes-gcm` crate's `encrypt` returns `Vec<u8>` which is Ciphertext + Tag.
    /// We need to prepend Nonce.
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12]; // 96-bit nonce
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Format: Nonce (12 bytes) + Ciphertext + Tag
        let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&combined))
    }

    /// Decrypt a base64 encoded string
    pub fn decrypt(&self, encrypted_base64: &str) -> Result<String> {
        let decoded = general_purpose::STANDARD
            .decode(encrypted_base64)
            .context("Invalid base64 in decrypt")?;

        if decoded.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext length"));
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed (MAC mismatch)"))?;

        String::from_utf8(plaintext_bytes).context("Decrypted data is not valid UTF-8")
    }
}
