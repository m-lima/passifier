// #![deny(
//     warnings,
//     rust_2018_idioms,
//     // missing_docs,
//     clippy::pedantic,
// )]

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Failed to load secret store
    #[error("Could not load secret store")]
    Loading,

    /// Failed to deserialize secret store
    #[error("Failed to deserialize secret store: {0}")]
    Deserializing(String),

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
}

struct StoreImpl {
    map: std::collections::HashMap<String, String>,
}

impl Store for StoreImpl {
    fn create(&mut self, key: String, value: String) -> Result<()> {
        match self.map.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(Error::KeyAlreadyExists),
        }
    }

    fn read<K: AsRef<str>>(&self, key: K) -> Option<&String> {
        self.map.get(key.as_ref())
    }

    fn update<K: AsRef<str>>(&mut self, key: K, value: String) -> Result<()> {
        let entry = self.map.get_mut(key.as_ref()).ok_or(Error::NotFound)?;
        *entry = value;
        Ok(())
    }

    fn delete<K: AsRef<str>>(&mut self, key: K) -> Result<String> {
        self.map.remove(key.as_ref()).ok_or(Error::NotFound)
    }
}

#[cfg(test)]
trait TestableStore: Store {
    fn map(&mut self) -> &mut std::collections::HashMap<String, String>;
}

#[cfg(test)]
impl TestableStore for StoreImpl {
    fn map(&mut self) -> &mut std::collections::HashMap<String, String> {
        &mut self.map
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::StoreImpl;
    use super::TestableStore;

    macro_rules! own {
        ($string:literal) => {
            String::from($string)
        };
    }

    #[test]
    fn store_impl_create() {
        create(&mut StoreImpl {
            map: std::collections::HashMap::new(),
        });
    }

    #[test]
    fn store_impl_read() {
        read(&mut StoreImpl {
            map: std::collections::HashMap::new(),
        });
    }

    #[test]
    fn store_impl_update() {
        update(&mut StoreImpl {
            map: std::collections::HashMap::new(),
        });
    }

    #[test]
    fn store_impl_delete() {
        delete(&mut StoreImpl {
            map: std::collections::HashMap::new(),
        });
    }

    fn setup(store: &mut impl TestableStore) -> std::collections::HashMap<String, String> {
        store.map().clear();
        store.map().insert(own!("existing"), own!("existing_value"));

        let mut reference = std::collections::HashMap::new();
        reference.insert(own!("existing"), own!("existing_value"));
        reference
    }

    fn create(store: &mut impl TestableStore) {
        let mut reference = setup(store);

        assert_eq!(
            store
                .create(own!("existing"), own!("existing_new_value"))
                .unwrap_err(),
            Error::KeyAlreadyExists
        );
        assert_eq!(store.map(), &reference);

        assert!(store.create(own!("new"), own!("new_value")).is_ok());
        reference.insert(own!("new"), own!("new_value"));
        assert_eq!(store.map(), &reference);

        assert_eq!(
            store
                .create(own!("new"), own!("new_new_value"))
                .unwrap_err(),
            Error::KeyAlreadyExists
        );
        assert_eq!(store.map(), &reference);
    }

    fn read(store: &mut impl TestableStore) {
        let reference = setup(store);
        assert!(store.read("new").is_none());
        assert_eq!(store.map(), &reference);
        assert_eq!(store.read("existing").unwrap(), "existing_value");
        assert_eq!(store.map(), &reference);
    }

    fn update(store: &mut impl TestableStore) {
        let mut reference = setup(store);

        assert_eq!(
            store.update("new", own!("new_value")).unwrap_err(),
            Error::NotFound
        );
        assert_eq!(store.map(), &reference);

        assert!(store.update("existing", own!("new_value")).is_ok());
        reference.insert(own!("existing"), own!("new_value"));
        assert_eq!(store.map(), &reference);
    }

    fn delete(store: &mut impl TestableStore) {
        let mut reference = setup(store);

        assert_eq!(store.delete("new").unwrap_err(), Error::NotFound);
        assert_eq!(store.map(), &reference);

        assert!(store.delete("existing").is_ok());
        reference.clear();
        assert_eq!(store.map(), &reference);
    }
}
