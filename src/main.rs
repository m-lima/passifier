#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod io;
mod ops;

type Store = nested_map::NestedMap<String, Entry>;
type Node = nested_map::Node<String, Entry>;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
enum Entry {
    String(String),
    Binary(Vec<u8>),
}

impl From<String> for Entry {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<&str> for Entry {
    fn from(string: &str) -> Self {
        Self::String(String::from(string))
    }
}

impl From<Vec<u8>> for Entry {
    fn from(vec: Vec<u8>) -> Self {
        Self::Binary(vec)
    }
}

impl From<&[u8]> for Entry {
    fn from(vec: &[u8]) -> Self {
        Self::Binary(vec.iter().map(Clone::clone).collect())
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    let arguments = args::Args::parse();

    let mut store = if let Some(source) = arguments.store {
        match source {
            args::Source::File(path) => io::load(path)?,
            args::Source::S3(_) => {
                anyhow::bail!("S3 not yet implemented")
            }
        }
    } else {
        Store::new()
    };

    match arguments.action {
        args::Action::Create(write) => ops::create(&mut store, write)?,
        args::Action::Read(read) => ops::read(&store, read)?,
        args::Action::Update(write) => ops::update(&mut store, write)?,
        args::Action::Delete(delete) => ops::delete(&mut store, delete)?,
        args::Action::Print(print) => return ops::print(store, &print),
    }

    if let Some(save) = arguments.save {
        match save {
            args::Source::File(path) => io::save(&store, path)?,
            args::Source::S3(_) => {
                anyhow::bail!("S3 not yet implemented")
            }
        }
    }

    Ok(())
}
