#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod cli;
mod gui;
mod io;
mod repl;
mod store;

fn main() -> anyhow::Result<()> {
    use clap::Clap;

    if std::env::args().skip(1).next().is_none() {
        gui::run()
    } else {
        let args = args::Args::parse();
        let input = args;

        if let Some(action) = args.action {
            cli::run(args.input, action, args.output)
        } else {
            repl::run(args.input)
        }
    }
}

// fn perform_action(action: Option<args::Action>, store: &mut store::Store) -> anyhow::Result<store::Store> {
//     match action {
//         None => unimplemented!("Repl not ready yet"),
//         Some(
//         args::Action::Read(args::Read { path, pretty }) => {
//             if let Some(path) = path {
//             } else {
//             }
//         }
//         store
//             .read(
//                 path.as_ref().map_or(&[], AsRef::as_ref),
//                 &mut std::io::stdout(),
//                 pretty,
//             )
//     }
//     match action {
//         args::Action::Create(args::Create { secret, path, .. }) => {
//             store.create(path.as_ref(), secret).map(|_| true)
//         }
//         args::Action::Read(args::Read { path, pretty }) => store
//             .read(
//                 path.as_ref().map_or(&[], AsRef::as_ref),
//                 &mut std::io::stdout(),
//                 pretty,
//             )
//             .map(|_| false),
//         args::Action::Update(args::Update { secret, path, .. }) => {
//             store.update(path.as_ref(), secret).map(|_| true)
//         }
//         args::Action::Delete(args::Delete { path, .. }) => {
//             store.delete(path.as_ref()).map(|_| true)
//         }
//     }
// }

// fn run(args: args::Args) -> anyhow::Result<()> {
//     let mut store = load_store(&args.input)?;
//     let should_save = perform_action(args.action, &mut store)?;
//     save_store(store, args.save, should_save, args.store)
// }
