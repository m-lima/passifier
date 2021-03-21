use super::store;
use anyhow::Context;

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

pub(super) fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<store::Store> {
    let crypter = make_crypter().with_context(|| "Creating key")?;
    let data = read_from_file(path).with_context(|| "Reading from file")?;
    Ok(crypter.decrypt(&data).with_context(|| "Decrypting")?)
}

pub(super) fn save<P: AsRef<std::path::Path>>(store: &store::Store, path: P) -> anyhow::Result<()> {
    let crypter = make_crypter().with_context(|| "Creating key")?;
    let encrypted = crypter.encrypt(store).with_context(|| "Encrypting")?;
    Ok(save_to_file(&encrypted, path).with_context(|| "Writing to file")?)
}
