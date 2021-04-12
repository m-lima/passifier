use super::args;
use super::store;

pub fn load(
    source: &Option<args::Source>,
    crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
) -> anyhow::Result<store::Store> {
    source.map_or_else(
        || Ok(store::Store::new()),
        |s| match s {
            args::Source::File(path) => file::load(path, crypter_supplier),
            args::Source::Directory(path) => directory::load(path),
            args::Source::S3(_) => anyhow::bail!("S3 not yet implemented"),
        },
    )
}

pub fn save(
    node: &store::Node,
    output: args::Source,
    crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
) -> anyhow::Result<()> {
    match output {
        args::Source::File(path) => file::save(node, path, true, crypter_supplier),
        args::Source::Directory(path) => directory::save(&node, path, true),
        args::Source::S3(_) => {
            anyhow::bail!("S3 not yet implemented")
        }
    }
}

mod file {
    use super::store;
    use anyhow::Context;

    pub fn load<P: AsRef<std::path::Path>>(
        path: P,
        crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
    ) -> anyhow::Result<store::Store> {
        let data = read_file(path).with_context(|| "Reading from file")?;
        Ok(crypter_supplier()
            .ok_or_else(|| anyhow::anyhow!("No crypto"))?
            .decrypt(&data)
            .with_context(|| "Decrypting")?)
    }

    pub fn save<P: AsRef<std::path::Path>>(
        node: &store::Node,
        path: P,
        check_conflict: bool,
        crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
    ) -> anyhow::Result<()> {
        let path = path.as_ref();
        if check_conflict && path.exists() {
            anyhow::bail!("File exists");
        }
        let encrypted = crypter_supplier()
            .ok_or_else(|| anyhow::anyhow!("No crypto"))?
            .encrypt(node)
            .with_context(|| "Encrypting")?;
        Ok(write_file(&encrypted, path).with_context(|| "Writing to file")?)
    }

    fn read_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn write_file<P: AsRef<std::path::Path>>(data: &[u8], path: P) -> Result<(), std::io::Error> {
        use std::io::Write;
        std::fs::File::create(path)?.write_all(data).map(|_| ())
    }
}

mod directory {
    use super::store;
    use anyhow::Context;

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Store> {
        Ok(store::Store::from(
            read_directory(path).with_context(|| "Reading from directory")?,
        ))
    }

    pub fn save<P: AsRef<std::path::Path>>(
        node: &store::Node,
        path: P,
        check_conflict: bool,
    ) -> anyhow::Result<()> {
        let path = path.as_ref();
        if path.exists() {
            if check_conflict {
                anyhow::bail!("File exists");
            } else {
                if path.is_dir() {
                    std::fs::remove_dir_all(path)
                } else {
                    std::fs::remove_file(path)
                }
                .with_context(|| "Overriding file")?;
            }
        }
        write_node(node, path)
    }

    fn read_directory<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::NestedMap> {
        let mut map = store::NestedMap::new();
        for maybe_entry in path.as_ref().read_dir()? {
            let value = maybe_entry.map(|e| e.path())?;
            if let Some(key) = value.file_name().map(|name| {
                name.to_str()
                    .ok_or_else(|| anyhow::anyhow!("bad file name"))
            }) {
                map.insert(
                    String::from(key?),
                    if value.is_dir() {
                        store::Node::Branch(read_directory(value)?)
                    } else {
                        read_file(value)?
                    },
                );
            }
        }
        Ok(map)
    }

    fn read_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Node> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if core::str::from_utf8(&buffer).is_ok() {
            // SAFETY: Previously checked
            Ok(store::Node::Leaf(store::Entry::String(unsafe {
                String::from_utf8_unchecked(buffer)
            })))
        } else {
            Ok(store::Node::Leaf(store::Entry::Binary(buffer)))
        }
    }

    fn write_node<P: AsRef<std::path::Path>>(node: &store::Node, path: P) -> anyhow::Result<()> {
        match node {
            store::Node::Leaf(store::Entry::String(value)) => {
                std::fs::write(path, value)?;
            }
            store::Node::Leaf(store::Entry::Binary(value)) => {
                std::fs::write(path, value)?;
            }
            store::Node::Branch(branch) => {
                write_map(branch, path)?;
            }
        }
        Ok(())
    }

    fn write_map<P: AsRef<std::path::Path>>(map: &store::NestedMap, path: P) -> anyhow::Result<()> {
        std::fs::create_dir_all(path.as_ref())?;
        for (name, node) in map.iter() {
            write_node(node, path.as_ref().join(name))?;
        }
        Ok(())
    }
}
