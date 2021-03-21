#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod io;
mod store;

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    let arguments = args::Args::parse();

    let mut store = if let Some(source) = arguments.store {
        match source {
            args::Source::File(path) => io::load_file(path)?,
            args::Source::Directory(path) => io::load_directory(path)?,
            args::Source::S3(_) => {
                anyhow::bail!("S3 not yet implemented")
            }
        }
    } else {
        store::Store::new()
    };

    match arguments.action {
        args::Action::Create(write) => store.create(write)?,
        args::Action::Read(read) => store.read(read).map(|_| ())?,
        args::Action::Update(write) => store.update(write)?,
        args::Action::Delete(delete) => store.delete(delete)?,
        args::Action::Print(print) => return store.print(&print),
    }

    if let Some(save) = arguments.save {
        match save {
            args::Source::File(path) => io::save_file(&store, path)?,
            args::Source::Directory(path) => io::save_directory(&store, path)?,
            args::Source::S3(_) => {
                anyhow::bail!("S3 not yet implemented")
            }
        }
    }

    Ok(())
}
