use super::store;
use anyhow::Context;

pub(super) fn load_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Store> {
    let crypter = make_crypter().with_context(|| "Creating key")?;
    let data = read_from_file(path).with_context(|| "Reading from file")?;
    Ok(crypter.decrypt(&data).with_context(|| "Decrypting")?)
}

pub(super) fn save_file<P: AsRef<std::path::Path>>(
    store: &store::Store,
    path: P,
) -> anyhow::Result<()> {
    let crypter = make_crypter().with_context(|| "Creating key")?;
    let encrypted = crypter.encrypt(store).with_context(|| "Encrypting")?;
    Ok(save_to_file(&encrypted, path).with_context(|| "Writing to file")?)
}

pub(super) fn load_directory<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Store> {
    let store = store::Store::from(directory_to_map(path)?);
    Ok(store)
}

pub(super) fn save_directory<P: AsRef<std::path::Path>>(
    store: &store::Store,
    path: P,
) -> anyhow::Result<()> {
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

fn make_crypter() -> anyhow::Result<crypter::Crypter> {
    let password = rpassword::prompt_password_stderr("Password: ")?;
    Ok(crypter::Crypter::new(password))
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
