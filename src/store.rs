type Result<T> = super::Result<T>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Store(std::collections::HashMap<String, String>);

impl super::Store for Store {
    fn create(&mut self, key: String, value: String) -> Result<()> {
        match self.0.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(super::Error::KeyAlreadyExists),
        }
    }

    fn read<K: AsRef<str>>(&self, key: K) -> Option<&String> {
        self.0.get(key.as_ref())
    }

    fn update<K: AsRef<str>>(&mut self, key: K, value: String) -> Result<()> {
        let entry = self.0.get_mut(key.as_ref()).ok_or(super::Error::NotFound)?;
        *entry = value;
        Ok(())
    }

    fn delete<K: AsRef<str>>(&mut self, key: K) -> Result<String> {
        self.0.remove(key.as_ref()).ok_or(super::Error::NotFound)
    }

    fn list(&self) -> super::Secrets<'_> {
        super::Secrets(self.0.keys())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Error;
    use super::super::Store as StoreTrait;
    use super::Store;

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
            Error::KeyAlreadyExists
        ));
        assert_eq!(store.0, reference);

        assert!(store.create(own!("new"), own!("new_value")).is_ok());
        reference.insert(own!("new"), own!("new_value"));
        assert_eq!(store.0, reference);

        assert!(matches!(
            store
                .create(own!("new"), own!("new_new_value"))
                .unwrap_err(),
            Error::KeyAlreadyExists
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
            Error::NotFound
        ));
        assert_eq!(store.0, reference);

        assert!(store.update("existing", own!("new_value")).is_ok());
        reference.insert(own!("existing"), own!("new_value"));
        assert_eq!(store.0, reference);
    }

    #[test]
    fn delete() {
        let (mut store, mut reference) = setup();

        assert!(matches!(store.delete("new").unwrap_err(), Error::NotFound));
        assert_eq!(store.0, reference);

        assert!(store.delete("existing").is_ok());
        reference.clear();
        assert_eq!(store.0, reference);
    }

    #[test]
    fn list() {
        let store = {
            let mut map = std::collections::HashMap::new();
            map.insert(own!("foo1"), own!("bar1"));
            map.insert(own!("foo2"), own!("bar2"));
            map.insert(own!("foo3"), own!("bar3"));
            Store(map)
        };
        let list = store.list().collect::<Vec<_>>();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&&own!("foo1")));
        assert!(list.contains(&&own!("foo2")));
        assert!(list.contains(&&own!("foo3")));
    }
}
