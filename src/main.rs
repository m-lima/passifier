#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod io;
mod store;

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    run(args::Args::parse())
}

fn run(args: args::Args) -> anyhow::Result<()> {
    let mut store = if let Some(source) = args.store {
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

    match args.action {
        args::Action::Create(write) => store.create(write)?,
        args::Action::Read(read) => store.read(read).map(|_| ())?,
        args::Action::Update(write) => store.update(write)?,
        args::Action::Delete(delete) => store.delete(delete)?,
        args::Action::Print(print) => return store.print(&print),
    }

    if let Some(save) = args.save {
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
