#![deny(warnings, rust_2018_idioms, missing_docs, clippy::pedantic)]

//! Handles secrets in a secret store

pub use crypter::Error as IOError;

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
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Store(std::collections::HashMap<String, String>);

impl Store {
    /// Load a secret store from the bytes with the given passphrase
    ///
    /// The bytes are expected to be encrypted, compressed, and encoded in raw binary
    ///
    /// # Errors
    /// Any decryption, deserialization, and decrompression failures will result in an
    /// [`IOError`](enum.IOError.html)
    pub fn load<S: AsRef<str>>(data: &[u8], pass: S) -> Result<Self, IOError> {
        crypter::Crypter::new(pass).decrypt(data)
    }

    /// Save the secret store back into bytes with the given passphrase
    ///
    /// The bytes will be encrypted, compressed, and encoded in raw binary
    ///
    /// # Errors
    /// Any encryption, serialization, and crompression failures will result in an
    /// [`IOError`](enum.IOError.html)
    pub fn save<S: AsRef<str>>(&self, pass: S) -> Result<Vec<u8>, IOError> {
        crypter::Crypter::new(pass).encrypt(self)
    }

    /// Creates a new secret in the store
    ///
    /// # Errors
    /// If the secret name already exists, a
    /// [`SecretAlreadyExists`](enum.StoreError.html#variant.SecretAlreadyExists) error will be returned
    pub fn create(&mut self, name: String, value: String) -> Result<(), StoreError> {
        match self.0.entry(name) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(StoreError::SecretAlreadyExists),
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
    /// [`SecretNotFound`](enum.StoreError.html#variant.SecretNotFound) error will be returned
    pub fn update<S: AsRef<str>>(&mut self, name: S, value: String) -> Result<(), StoreError> {
        let entry = self
            .0
            .get_mut(name.as_ref())
            .ok_or(StoreError::SecretNotFound)?;
        *entry = value;
        Ok(())
    }

    /// Deletes a secret from the store
    ///
    /// # Errors
    /// If the secret name does not exist, a
    /// [`SecretNotFound`](enum.StoreError.html#variant.SecretNotFound) error will be returned
    pub fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<String, StoreError> {
        self.0
            .remove(name.as_ref())
            .ok_or(StoreError::SecretNotFound)
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

    use super::Store;
    use super::StoreError;

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

    fn new_store() -> Store {
        let mut map = std::collections::HashMap::new();
        map.insert(own!("foo1"), own!("bar1"));
        map.insert(own!("foo2"), own!("bar2"));
        map.insert(own!("foo3"), own!("bar3"));
        Store(map)
    }

    #[test]
    fn create() {
        let (mut store, mut reference) = setup();

        assert_eq!(
            store
                .create(own!("existing"), own!("existing_new_value"))
                .unwrap_err(),
            StoreError::SecretAlreadyExists
        );
        assert_eq!(store.0, reference);

        assert!(store.create(own!("new"), own!("new_value")).is_ok());
        reference.insert(own!("new"), own!("new_value"));
        assert_eq!(store.0, reference);

        assert_eq!(
            store
                .create(own!("new"), own!("new_new_value"))
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
        assert_eq!(store.read("existing").unwrap(), "existing_value");
        assert_eq!(store.0, reference);
    }

    #[test]
    fn update() {
        let (mut store, mut reference) = setup();

        assert_eq!(
            store.update("new", own!("new_value")).unwrap_err(),
            StoreError::SecretNotFound
        );
        assert_eq!(store.0, reference);

        assert!(store.update("existing", own!("new_value")).is_ok());
        reference.insert(own!("existing"), own!("new_value"));
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
        let mut list = store.iter().collect::<Vec<_>>();
        list.sort();
        assert_eq!(
            list,
            [
                (&own!("foo1"), &own!("bar1")),
                (&own!("foo2"), &own!("bar2")),
                (&own!("foo3"), &own!("bar3"))
            ]
        );
    }

    #[test]
    fn round_trip() {
        let store = new_store();

        let bytes = store.save("mega-pass").unwrap();
        let recovered = Store::load(&bytes, "mega-pass").unwrap();
        assert_eq!(store, recovered);
    }
}
