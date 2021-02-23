#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Store(std::collections::HashMap<String, Entry>);

impl std::convert::From<store::Store> for Store {
    fn from(store: store::Store) -> Self {
        Self(store.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl std::convert::Into<store::Store> for Store {
    fn into(self) -> store::Store {
        let mut store = store::Store::new();
        for entry in self.0 {
            store.create(entry.0, entry.1.into()).unwrap();
        }
        store
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Entry {
    String(String),
    Binary(Vec<u8>),
    Nested(Store),
}

impl std::str::FromStr for Entry {
    type Err = serde_json::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(string).or_else(|_| Ok(Self::String(String::from(string))))
    }
}

impl std::convert::From<store::Entry> for Entry {
    fn from(entry: store::Entry) -> Self {
        match entry {
            store::Entry::String(string) => Entry::String(string),
            store::Entry::Binary(binary) => Entry::Binary(binary),
            store::Entry::Nested(store) => Entry::Nested(store.into()),
        }
    }
}

impl std::convert::Into<store::Entry> for Entry {
    fn into(self) -> store::Entry {
        match self {
            Self::String(string) => store::Entry::String(string),
            Self::Binary(binary) => store::Entry::Binary(binary),
            Self::Nested(store) => store::Entry::Nested(store.into()),
        }
    }
}
