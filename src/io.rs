use super::store;
use anyhow::Context;

pub(super) fn load_file<P: AsRef<std::path::Path>>(
    path: P,
    crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
) -> anyhow::Result<store::Store> {
    let data = read_from_file(path).with_context(|| "Reading from file")?;
    Ok(crypter_supplier()
        .ok_or_else(|| anyhow::anyhow!("No crypto"))?
        .decrypt(&data)
        .with_context(|| "Decrypting")?)
}

pub(super) fn save_file<P: AsRef<std::path::Path>>(
    store: &store::Store,
    path: P,
    crypter_supplier: impl Fn() -> Option<crypter::Crypter>,
    check_conflict: bool,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    if check_conflict && path.exists() {
        anyhow::bail!("File exists");
    }
    let encrypted = crypter_supplier()
        .ok_or_else(|| anyhow::anyhow!("No crypto"))?
        .encrypt(store)
        .with_context(|| "Encrypting")?;
    Ok(save_to_file(&encrypted, path).with_context(|| "Writing to file")?)
}

pub(super) fn load_directory<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Store> {
    let store = store::Store::from(directory_to_map(path)?);
    Ok(store)
}

pub(super) fn save_directory<P: AsRef<std::path::Path>>(
    store: &store::Store,
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
    write_file(store, path)
}

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

fn directory_to_map<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::NestedMap> {
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
                    store::Node::Branch(directory_to_map(value)?)
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

fn write_file<P: AsRef<std::path::Path>>(map: &store::NestedMap, path: P) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.as_ref())?;
    for (name, node) in map.iter() {
        match node {
            store::Node::Leaf(store::Entry::String(value)) => {
                std::fs::write(path.as_ref().join(name), value)?;
            }
            store::Node::Leaf(store::Entry::Binary(value)) => {
                std::fs::write(path.as_ref().join(name), value)?;
            }
            store::Node::Branch(branch) => {
                write_file(branch, path.as_ref().join(name))?;
            }
        }
    }
    Ok(())
}
