#![deny(
    warnings,
    rust_2018_idioms,
    // missing_docs,
    clippy::pedantic,
)]

mod crypter;
mod store;

macro_rules! impl_store {
    ($name:ty) => {
        impl Store for $name {
            fn create(&mut self, key: String, value: String) -> Result<()> {
                self.store.create(key, value)
            }

            fn read<K: AsRef<str>>(&self, key: K) -> Option<&String> {
                self.store.read(key)
            }

            fn update<K: AsRef<str>>(&mut self, key: K, value: String) -> Result<()> {
                self.store.update(key, value)
            }

            fn delete<K: AsRef<str>>(&mut self, key: K) -> Result<String> {
                self.store.delete(key)
            }

            fn list(&self) -> Secrets<'_> {
                self.store.list()
            }
        }
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// IO error while loading/saving secret store
    #[error("IO error: {0}")]
    IO(std::io::Error),

    /// Failed to encrypt/decrypt secret store
    #[error("Failed to perform crypto for secret store: {0}")]
    Crypto(crypter::Error),

    /// Key already exists
    #[error("Key already exists")]
    KeyAlreadyExists,

    /// Key not found
    #[error("Key not found")]
    NotFound,
}

pub trait Store {
    fn create(&mut self, key: String, value: String) -> Result<()>;
    fn read<K: AsRef<str>>(&self, key: K) -> Option<&String>;
    fn update<K: AsRef<str>>(&mut self, key: K, value: String) -> Result<()>;
    fn delete<K: AsRef<str>>(&mut self, key: K) -> Result<String>;
    fn list(&self) -> Secrets<'_>;
}

pub struct Secrets<'a>(std::collections::hash_map::Keys<'a, String, String>);

impl<'a> Iterator for Secrets<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct FileStore {
    path: std::path::PathBuf,
    store: store::Store,
}

impl_store!(FileStore);

impl FileStore {
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
