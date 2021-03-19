#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod io;
// mod ops;

type Store = nested_map::NestedMap<String, Entry>;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
enum Entry {
    String(String),
    Binary(Vec<u8>),
}

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    let arguments = args::Args::parse();

    let store = if let Some(source) = arguments.store {
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
        // args::Action::Create(entry) => ops::create(&mut store, entry.path.as_ref(), entry.secret)?,
        args::Action::Read(path) => {
            let entry = store.get_from_iter(path.iter());
            println!("{}", serde_json::to_string(&entry)?);
        }
        // args::Action::Update(entry) => ops::update(&mut store, entry.path.as_ref(), entry.secret)?,
        // args::Action::Delete(path) => ops::delete(&mut store, path.path.as_ref())?,
        args::Action::Print(print) => {
            let json = if print.pretty {
                serde_json::to_string_pretty(&store)?
            } else {
                serde_json::to_string(&store)?
            };

            println!("{}", json);
        }
        _ => eprintln!("Unimplemented"),
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
