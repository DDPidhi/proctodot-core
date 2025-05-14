use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce}; // Or `Aes128Gcm`
use rand::RngCore;
use rand::rngs::OsRng;
use base64;
use std::env;

pub struct Encryptor;

impl Encryptor {
    /// Retrieve the encryption key from the environment variable
    fn get_key() -> Result<[u8; 32], &'static str> {
        let key_str = env::var("APP_KEY").map_err(|_| "APP_KEY not set")?;
        let key_bytes = base64::decode(&key_str).map_err(|_| "Invalid APP_KEY")?;
        if key_bytes.len() != 32 {
            return Err("Invalid APP_KEY length");
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt_data(plain_text: &str) -> Result<String, &'static str> {
        let key_bytes = Self::get_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| "Invalid key")?;

        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = cipher.encrypt(nonce, plain_text.as_bytes())
            .map_err(|_| "Encryption failed")?;

        // Concatenate nonce and ciphertext
        let mut nonce_and_ciphertext = Vec::new();
        nonce_and_ciphertext.extend_from_slice(&nonce_bytes);
        nonce_and_ciphertext.extend_from_slice(&ciphertext);

        // Base64 encode
        let encoded = base64::encode(&nonce_and_ciphertext);
        Ok(encoded)
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt_data(cipher_text: &str) -> Result<String, &'static str> {
        let key_bytes = Self::get_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| "Invalid key")?;

        // Base64 decode
        let nonce_and_ciphertext = base64::decode(cipher_text).map_err(|_| "Base64 decoding failed")?;
        if nonce_and_ciphertext.len() < 12 {
            return Err("Ciphertext too short");
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = nonce_and_ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed")?;

        let plaintext = String::from_utf8(plaintext_bytes).map_err(|_| "Invalid UTF-8")?;
        Ok(plaintext)
    }
}
