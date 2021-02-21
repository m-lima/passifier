#![deny(warnings, rust_2018_idioms, missing_docs, clippy::pedantic)]

//! Allows encrypting and decrypting serde payloads with AES/GCM encryption

/// Errors that can happen while serialization/deserialization, inflation/compression, and
/// decryption/encryption
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to serialize/deserialize payload
    #[error("Could not serialize/deserialize payload: {0}")]
    Serde(Box<bincode::ErrorKind>),

    /// Failed to encrypt/decrypt payload
    #[error("Failed to encrypt/decrypt payload: {0}")]
    Crypto(aes_gcm::Error),

    /// Failed to inflate payload
    #[error("Failed to inflate payload")]
    Inflation,
}

/// Struct holding the cipher that can be used to encrypt and decrypt payloads
pub struct Crypter {
    cipher: aes_gcm::Aes256Gcm,
    nonce:
        aes_gcm::aead::generic_array::GenericArray<u8, aes_gcm::aead::generic_array::typenum::U12>,
}

impl Crypter {
    /// Creates a new cipher with the given passphrase
    pub fn new<S: AsRef<str>>(passphrase: S) -> Self {
        use aes_gcm::aead::generic_array::GenericArray;
        use aes_gcm::aead::NewAead;
        use sha2::Digest;

        let secret = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(passphrase.as_ref().as_bytes());
            hasher.finalize()
        };

        let mut nonce = [0; 12];
        nonce.copy_from_slice(&secret[..12]);

        Self {
            cipher: aes_gcm::Aes256Gcm::new(GenericArray::from_slice(&secret[..32])),
            nonce: GenericArray::from(nonce),
        }
    }

    /// Encrypts the payload
    ///
    /// # Errors
    /// Can fail at any of these points:
    /// * Serialization: [`Serde`](enum.Error.html#variant.Serde)
    /// * Encryption: [`Crypto`](enum.Error.html#variant.Crypto)
    pub fn encrypt<T: serde::Serialize>(&self, payload: &T) -> Result<Vec<u8>, Error> {
        use aes_gcm::aead::Aead;

        let binary = bincode::serialize(&payload).map_err(Error::Serde)?;

        #[cfg(feature = "miniz_oxide")]
        let binary = miniz_oxide::deflate::compress_to_vec(&binary, 8);

        self.cipher
            .encrypt(&self.nonce, binary.as_slice())
            .map_err(Error::Crypto)
    }

    /// Decrypts into the payload
    ///
    /// # Errors
    /// Can fail at any of these points:
    /// * Deserialization: [`Serde`](enum.Error.html#variant.Serde)
    /// * Decryption: [`Crypto`](enum.Error.html#variant.Crypto)
    /// * Inflation: [`Inflation`](enum.Error.html#variant.Inflation)
    ///   * Only if feature `compression` is enabled
    pub fn decrypt<T: serde::de::DeserializeOwned>(&self, payload: &[u8]) -> Result<T, Error> {
        use aes_gcm::aead::Aead;

        let decrypted: Vec<u8> = self
            .cipher
            .decrypt(&self.nonce, payload)
            .map_err(Error::Crypto)?;

        // Allowed because the returned error is quite useless, just a number
        #[allow(clippy::map_err_ignore)]
        #[cfg(feature = "miniz_oxide")]
        let decrypted =
            miniz_oxide::inflate::decompress_to_vec(&decrypted).map_err(|_| Error::Inflation)?;

        bincode::deserialize(&decrypted).map_err(Error::Serde)
    }
}

#[cfg(test)]
mod tests {
    use super::Crypter;

    #[test]
    fn can_contruct() {
        let crypter = Crypter::new("foobar");
        assert!(!crypter.nonce.as_slice().iter().all(|b| b == &0));
    }

    #[test]
    fn empty_key() {
        let crypter = Crypter::new("");
        assert!(!crypter.nonce.as_slice().iter().all(|b| b == &0));
    }

    #[test]
    fn round_trip() {
        let mut map = std::collections::HashMap::new();
        map.insert(String::from("foo"), 123_i32);
        map.insert(String::from("bar"), 321_i32);

        let crypter = Crypter::new("foo\u{1f1e7}\u{1f1f7}\u{1f1f3}\u{1f1f4}bar");
        let encrypted = crypter.encrypt(&map).unwrap();
        let decrypted = crypter
            .decrypt::<std::collections::HashMap<String, i32>>(&encrypted)
            .unwrap();

        assert_eq!(map, decrypted);
    }
}
