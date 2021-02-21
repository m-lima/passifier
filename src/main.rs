fn save_to_file<P: AsRef<std::path::Path>>(data: &[u8], path: P) -> Result<(), std::io::Error> {
    use std::io::Write;

    std::fs::File::create(path)?.write_all(data).map(|_| ())
}

fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

mod json {
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
}

fn create() -> anyhow::Result<store::Store> {
    let mut store = store::Store::new();
    store.create(
        String::from("aws"),
        store::Entry::String(String::from("foobar")),
    )?;

    store.create(String::from("nested"), store::Entry::Nested(store.clone()))?;
    store.create(
        String::from("binary"),
        store::Entry::Binary(store.encrypt("foo")?),
    )?;
    Ok(store)
}

fn main() -> anyhow::Result<()> {
    let store = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("no command"))
        .and_then(|f| match f.as_str() {
            "save" => {
                let store = create()?;
                save_to_file(&store.encrypt("fulpac")?, "./here.store")?;
                Ok(store)
            }
            "save_json" => {
                let store = json::Store::from(create()?);
                save_to_file(&serde_json::to_vec(&store)?, "./here.json")?;
                Ok(store.into())
            }
            "load_json" => {
                let store: json::Store = serde_json::from_slice(&read_from_file("./here.json")?)?;
                Ok(store.into())
            }
            "load" => Ok(store::Store::decrypt(
                &read_from_file("./here.store")?,
                "fulpac",
            )?),
            _ => anyhow::bail!("unrecognized command"),
        })?;

    let json_store = json::Store::from(store);
    println!("{}", serde_json::to_string(&json_store)?);

    Ok(())
}
