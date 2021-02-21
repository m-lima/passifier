type Result<T> = super::Result<T>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Store(std::collections::HashMap<String, String>);

impl super::Store for Store {
    fn create(&mut self, name: String, value: String) -> Result<()> {
        match self.0.entry(name) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                Err(super::Error::SecretAlreadyExists)
            }
        }
    }

    fn read<S: AsRef<str>>(&self, name: S) -> Option<&String> {
        self.0.get(name.as_ref())
    }

    fn update<S: AsRef<str>>(&mut self, name: S, value: String) -> Result<()> {
        let entry = self
            .0
            .get_mut(name.as_ref())
            .ok_or(super::Error::SecretNotFound)?;
        *entry = value;
        Ok(())
    }

    fn delete<S: AsRef<str>>(&mut self, name: S) -> Result<String> {
        self.0
            .remove(name.as_ref())
            .ok_or(super::Error::SecretNotFound)
    }

    fn secrets(&self) -> super::SecretNames<'_> {
        super::SecretNames(self.0.keys())
    }

    fn iter(&self) -> super::Secrets<'_> {
        super::Secrets(self.0.iter())
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
