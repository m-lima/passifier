#![deny(warnings, rust_2018_idioms, missing_docs, clippy::pedantic)]

//! Handles secrets in a secret store

mod crypter;

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

/// Possible backends for the secret store
pub enum Backend {
    /// File based secret store
    File(std::path::PathBuf),
}

impl Backend {
    fn load(&self) -> Result<Vec<u8>> {
        match self {
            Self::File(path) => {
                use std::io::Read;

                let mut file = std::fs::File::open(path).map_err(Error::IO)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).map_err(Error::IO)?;
                Ok(buffer)
            }
        }
    }

    fn save(&self, buffer: &[u8]) -> Result<()> {
        match self {
            Self::File(path) => {
                use std::io::Write;

                std::fs::File::create(path)
                    .map_err(Error::IO)?
                    .write_all(buffer)
                    .map_err(Error::IO)
                    .map(|_| ())
            }
        }
    }
}

/// Yo
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Store(std::collections::HashMap<String, String>);

impl Store {
    /// Load a secret store from the backend with the given passphrase
    ///
    /// # Errors
    /// Any IO failure will result in an error. After loading the bytes, any decryption,
    /// deserialization, and decrompression failures will result in an error.
    pub fn load<S: AsRef<str>>(backend: &Backend, pass: S) -> Result<Self> {
        let buffer = backend.load()?;
        crypter::Crypter::new(pass)
            .decrypt(&buffer)
            .map_err(Error::Crypto)
    }

    /// Save the secret store back into the backend with the given passphrase
    ///
    /// # Errors
    /// Any encryption, serialization, and crompression failures will result in an error. After
    /// preparing the payload, any IO failure will result in an error.
    pub fn save<S: AsRef<str>>(&self, backend: &Backend, pass: S) -> Result<()> {
        let buffer = crypter::Crypter::new(pass)
            .encrypt(self)
            .map_err(Error::Crypto)?;

        backend.save(&buffer)
    }

    /// Creates a new secret in the store
    ///
    /// # Errors
    /// If the secret name already exists, a
    /// [`SecretAlreadyExists`](enum.Error.html#variant.SecretAlreadyExists) error will be returned
    pub fn create(&mut self, name: String, value: String) -> Result<()> {
        match self.0.entry(name) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(Error::SecretAlreadyExists),
        }
    }

    /// Reads a secret from the store, if it exists
    pub fn read<S: AsRef<str>>(&self, name: S) -> Option<&String> {
        self.0.get(name.as_ref())
    }

    /// Updates a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.Error.html#variant.SecretNotFound) error will be returned
    pub fn update<S: AsRef<str>>(&mut self, name: S, value: String) -> Result<()> {
        let entry = self.0.get_mut(name.as_ref()).ok_or(Error::SecretNotFound)?;
        *entry = value;
        Ok(())
    }

    /// Deletes a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.Error.html#variant.SecretNotFound) error will be returned
    pub fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<String> {
        self.0.remove(name.as_ref()).ok_or(Error::SecretNotFound)
    }

    /// An iterator over all the secret names stored
    pub fn secrets(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// An iterator over all the secret name/value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {

    mod backend {}

    mod store {
        use super::super::Error;
        use super::super::Store;

        macro_rules! own {
            ($string:literal) => {
                String::from($string)
            };
        }

        fn setup() -> (Store, std::collections::HashMap<String, String>) {
            let mut reference = std::collections::HashMap::new();
            reference.insert(own!("existing"), own!("existing_value"));

            let store = Store(reference.clone());

            (store, reference)
        }

        #[test]
        fn create() {
            let (mut store, mut reference) = setup();

            assert!(matches!(
                store
                    .create(own!("existing"), own!("existing_new_value"))
                    .unwrap_err(),
                Error::SecretAlreadyExists
            ));
            assert_eq!(store.0, reference);

            assert!(store.create(own!("new"), own!("new_value")).is_ok());
            reference.insert(own!("new"), own!("new_value"));
            assert_eq!(store.0, reference);

            assert!(matches!(
                store
                    .create(own!("new"), own!("new_new_value"))
                    .unwrap_err(),
                Error::SecretAlreadyExists
            ));
            assert_eq!(store.0, reference);
        }

        #[test]
        fn read() {
            let (store, reference) = setup();
            assert!(store.read("new").is_none());
            assert_eq!(store.0, reference);
            assert_eq!(store.read("existing").unwrap(), "existing_value");
            assert_eq!(store.0, reference);
        }

        #[test]
        fn update() {
            let (mut store, mut reference) = setup();

            assert!(matches!(
                store.update("new", own!("new_value")).unwrap_err(),
                Error::SecretNotFound
            ));
            assert_eq!(store.0, reference);

            assert!(store.update("existing", own!("new_value")).is_ok());
            reference.insert(own!("existing"), own!("new_value"));
            assert_eq!(store.0, reference);
        }

        #[test]
        fn delete() {
            let (mut store, mut reference) = setup();

            assert!(matches!(
                store.delete("new").unwrap_err(),
                Error::SecretNotFound
            ));
            assert_eq!(store.0, reference);

            assert!(store.delete("existing").is_ok());
            reference.clear();
            assert_eq!(store.0, reference);
        }

        #[test]
        fn secrets() {
            let store = {
                let mut map = std::collections::HashMap::new();
                map.insert(own!("foo1"), own!("bar1"));
                map.insert(own!("foo2"), own!("bar2"));
                map.insert(own!("foo3"), own!("bar3"));
                Store(map)
            };
            let list = store.secrets().collect::<Vec<_>>();
            assert_eq!(list.len(), 3);
            assert!(list.contains(&&own!("foo1")));
            assert!(list.contains(&&own!("foo2")));
            assert!(list.contains(&&own!("foo3")));
        }
    }
}
