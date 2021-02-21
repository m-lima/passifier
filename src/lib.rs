#![deny(warnings, rust_2018_idioms, missing_docs, clippy::pedantic)]

//! Handles secrets in a secret store

mod crypter;
mod store;

macro_rules! impl_store {
    ($name:ty) => {
        impl Store for $name {
            fn create(&mut self, name: String, value: String) -> Result<()> {
                self.store.create(name, value)
            }

            fn read<K: AsRef<str>>(&self, name: K) -> Option<&String> {
                self.store.read(name)
            }

            fn update<K: AsRef<str>>(&mut self, name: K, value: String) -> Result<()> {
                self.store.update(name, value)
            }

            fn delete<K: AsRef<str>>(&mut self, name: K) -> Result<String> {
                self.store.delete(name)
            }

            fn secrets(&self) -> SecretNames<'_> {
                self.store.secrets()
            }

            fn iter(&self) -> Secrets<'_> {
                self.store.iter()
            }
        }
    };
}

/// Conveniene alias for operations that may return an [`Error`](enum.Error.html)
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that may happen
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// IO error while loading/saving secret store
    #[error("IO error: {0}")]
    IO(std::io::Error),

    /// Failed to encrypt/decrypt secret store
    #[error("Failed to perform crypto for secret store: {0}")]
    Crypto(crypter::Error),

    /// Secret already exists
    #[error("Secret already exists")]
    SecretAlreadyExists,

    /// Secret not found
    #[error("Secret not found")]
    SecretNotFound,
}

/// Trait for all implementaions of a secrets store
pub trait Store {
    /// Creates a new secret in the store
    ///
    /// # Errors
    /// If the secret name already exists, a
    /// [`SecretAlreadyExists`](enum.Error.html#variant.SecretAlreadyExists) error will be returned
    fn create(&mut self, name: String, value: String) -> Result<()>;

    /// Reads a secret from the store, if it exists
    fn read<S: AsRef<str>>(&self, name: S) -> Option<&String>;

    /// Updates a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.Error.html#variant.SecretNotFound) error will be returned
    fn update<S: AsRef<str>>(&mut self, name: S, value: String) -> Result<()>;

    /// Deletes a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.Error.html#variant.SecretNotFound) error will be returned
    fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<String>;

    /// An iterator over all the secret names stored
    fn secrets(&self) -> SecretNames<'_>;

    /// An iterator over all the secret name/value pairs
    fn iter(&self) -> Secrets<'_>;
}

/// Iterator over secret names
pub struct SecretNames<'a>(std::collections::hash_map::Keys<'a, String, String>);

impl<'a> Iterator for SecretNames<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator over secret name/value pais
pub struct Secrets<'a>(std::collections::hash_map::Iter<'a, String, String>);

impl<'a> Iterator for Secrets<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Implementation of a secret store that is file-based
pub struct FileStore {
    path: std::path::PathBuf,
    store: store::Store,
}

impl_store!(FileStore);

impl FileStore {
    /// Loads a secret store from file
    ///
    /// # Errors
    /// Any IO failure will result in an error. After reading the file, any decryption,
    /// deserialization, and decrompression failures will result in an error.
    pub fn load<P: AsRef<std::path::Path>, S: AsRef<str>>(path: P, pass: S) -> Result<Self> {
        use std::io::Read;

        let mut file = std::fs::File::open(path.as_ref()).map_err(Error::IO)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(Error::IO)?;

        let store = crypter::Crypter::new(pass)
            .decrypt(&buffer)
            .map_err(Error::Crypto)?;
        Ok(Self {
            path: std::path::PathBuf::from(path.as_ref()),
            store,
        })
    }

    /// Save the secret store back into file
    ///
    /// # Errors
    /// Any encryption, serialization, and crompression failures will result in an error. After
    /// preparing the payload, any IO failure will result in an error.
    pub fn save<S: AsRef<str>>(self, pass: S) -> Result<()> {
        use std::io::Write;

        let buffer = crypter::Crypter::new(pass)
            .encrypt(&self.store)
            .map_err(Error::Crypto)?;

        std::fs::File::create(self.path)
            .map_err(Error::IO)?
            .write_all(&buffer)
            .map_err(Error::IO)
            .map(|_| ())
    }
}
