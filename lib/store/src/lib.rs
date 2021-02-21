#![deny(warnings, rust_2018_idioms, missing_docs, clippy::pedantic)]

//! Handles secrets in a secret store

pub use crypter::Error as CryptoError;

/// Errors that may happen
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum StoreError {
    /// Secret already exists
    #[error("Secret already exists")]
    SecretAlreadyExists,

    /// Secret not found
    #[error("Secret not found")]
    SecretNotFound,
}

/// A secret store that can be loaded from a byte array and stored back into a byte array
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Store(std::collections::HashMap<String, Entry>);

/// Possible values that can be stored in the secret store
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Entry {
    /// Plain string
    String(String),
    /// Binary data
    Binary(Vec<u8>),
    /// A nested secret store
    Nested(Store),
}

impl std::fmt::Display for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => string.fmt(fmt),
            Self::Binary(_) => write!(fmt, "[Binary data]"),
            Self::Nested(_) => write!(fmt, "[Nested store]"),
        }
    }
}

impl Store {
    /// Creates a new empty store
    #[must_use]
    pub fn new() -> Self {
        Self(std::collections::HashMap::new())
    }

    /// Decrypt a secret store from bytes with the given passphrase
    ///
    /// The bytes are expected to be encrypted, compressed, and encoded in raw binary
    ///
    /// # Errors
    /// Any decryption, deserialization, and decrompression failures will result in an
    /// [`CryptoError`](enum.CryptoError.html)
    pub fn decrypt<S: AsRef<str>>(data: &[u8], pass: S) -> Result<Self, CryptoError> {
        crypter::Crypter::new(pass).decrypt(data)
    }

    /// Encrypt the secret store into bytes with the given passphrase
    ///
    /// The store will be encrypted, compressed, and encoded in raw binary
    ///
    /// # Errors
    /// Any encryption, serialization, and crompression failures will result in an
    /// [`CryptoError`](enum.CryptoError.html)
    pub fn encrypt<S: AsRef<str>>(&self, pass: S) -> Result<Vec<u8>, CryptoError> {
        crypter::Crypter::new(pass).encrypt(self)
    }

    /// Creates a new secret in the store
    ///
    /// # Errors
    /// If the secret name already exists, a
    /// [`SecretAlreadyExists`](enum.StoreError.html#variant.SecretAlreadyExists) error will be returned
    pub fn create(&mut self, name: String, entry: Entry) -> Result<(), StoreError> {
        match self.0.entry(name) {
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(entry);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(StoreError::SecretAlreadyExists),
        }
    }

    /// Reads a secret from the store, if it exists
    pub fn read<S: AsRef<str>>(&self, name: S) -> Option<&Entry> {
        self.0.get(name.as_ref())
    }

    /// Loads a secret from the store, if it exists
    pub fn get<S: AsRef<str>>(&mut self, name: S) -> Option<&mut Entry> {
        self.0.get_mut(name.as_ref())
    }

    /// Updates a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.StoreError.html#variant.SecretNotFound) error will be returned
    pub fn update<S: AsRef<str>>(&mut self, name: S, new_entry: Entry) -> Result<(), StoreError> {
        let entry = self
            .0
            .get_mut(name.as_ref())
            .ok_or(StoreError::SecretNotFound)?;
        *entry = new_entry;
        Ok(())
    }

    /// Deletes a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.StoreError.html#variant.SecretNotFound) error will be returned
    pub fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<Entry, StoreError> {
        self.0
            .remove(name.as_ref())
            .ok_or(StoreError::SecretNotFound)
    }

    /// An iterator over all the secret names stored
    pub fn secrets(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// An iterator over all the secret name/value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entry)> {
        self.0.iter()
    }
}

impl IntoIterator for Store {
    type Item = (String, Entry);
    type IntoIter = std::collections::hash_map::IntoIter<String, Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Entry;
    use super::Store;
    use super::StoreError;

    macro_rules! own {
        ($string:literal) => {
            String::from($string)
        };
    }

    macro_rules! entry {
        ($string:literal) => {
            Entry::String(String::from($string))
        };
    }

    fn setup() -> (Store, std::collections::HashMap<String, Entry>) {
        let mut reference = std::collections::HashMap::new();
        reference.insert(own!("existing"), entry!("existing_value"));

        let store = Store(reference.clone());

        (store, reference)
    }

    fn new_store() -> Store {
        let mut map = std::collections::HashMap::new();
        map.insert(own!("foo1"), entry!("bar1"));
        map.insert(own!("foo2"), entry!("bar2"));
        map.insert(own!("foo3"), entry!("bar3"));
        Store(map)
    }

    #[test]
    fn create() {
        let (mut store, mut reference) = setup();

        assert_eq!(
            store
                .create(own!("existing"), entry!("existing_new_value"))
                .unwrap_err(),
            StoreError::SecretAlreadyExists
        );
        assert_eq!(store.0, reference);

        assert!(store.create(own!("new"), entry!("new_value")).is_ok());
        reference.insert(own!("new"), entry!("new_value"));
        assert_eq!(store.0, reference);

        assert_eq!(
            store
                .create(own!("new"), entry!("new_new_value"))
                .unwrap_err(),
            StoreError::SecretAlreadyExists
        );
        assert_eq!(store.0, reference);
    }

    #[test]
    fn read() {
        let (store, reference) = setup();
        assert!(store.read("new").is_none());
        assert_eq!(store.0, reference);
        assert_eq!(
            store.read("existing").unwrap().to_string(),
            "existing_value"
        );
        assert_eq!(store.0, reference);
    }

    #[test]
    fn get() {
        let (mut store, mut reference) = setup();
        assert!(store.get("new").is_none());
        assert_eq!(store.0, reference);
        assert_eq!(store.get("existing").unwrap().to_string(), "existing_value");
        assert_eq!(store.0, reference);

        let entry = store.get("existing").unwrap();
        *entry = entry!("new_value");
        reference.insert(own!("existing"), entry!("new_value"));
        assert_eq!(store.0, reference);
    }

    #[test]
    fn update() {
        let (mut store, mut reference) = setup();

        assert_eq!(
            store.update("new", entry!("new_value")).unwrap_err(),
            StoreError::SecretNotFound
        );
        assert_eq!(store.0, reference);

        assert!(store.update("existing", entry!("new_value")).is_ok());
        reference.insert(own!("existing"), entry!("new_value"));
        assert_eq!(store.0, reference);
    }

    #[test]
    fn delete() {
        let (mut store, mut reference) = setup();

        assert_eq!(store.delete("new").unwrap_err(), StoreError::SecretNotFound);
        assert_eq!(store.0, reference);

        assert!(store.delete("existing").is_ok());
        reference.clear();
        assert_eq!(store.0, reference);
    }

    #[test]
    fn secrets() {
        let store = new_store();
        let mut list = store.secrets().collect::<Vec<_>>();
        list.sort();
        assert_eq!(list, ["foo1", "foo2", "foo3"]);
    }

    #[test]
    fn iter() {
        let store = new_store();
        let mut list = store
            .iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<Vec<_>>();
        list.sort();
        assert_eq!(
            list,
            [
                (&own!("foo1"), own!("bar1")),
                (&own!("foo2"), own!("bar2")),
                (&own!("foo3"), own!("bar3"))
            ]
        );
    }

    #[test]
    fn round_trip() {
        let mut store = new_store();
        let inner_store = new_store();
        store
            .create(own!("inner"), Entry::Nested(inner_store))
            .unwrap();

        let bytes = store.encrypt("mega-pass").unwrap();
        let recovered = Store::decrypt(&bytes, "mega-pass").unwrap();
        assert_eq!(store, recovered);
    }
}
