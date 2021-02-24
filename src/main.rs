// #![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod json;
mod ops;

// fn save_to_file<P: AsRef<std::path::Path>>(data: &[u8], path: P) -> Result<(), std::io::Error> {
//     use std::io::Write;

//     std::fs::File::create(path)?.write_all(data).map(|_| ())
// }

fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// fn create() -> anyhow::Result<store::Store> {
//     let mut store = store::Store::new();
//     store.create(
//         String::from("aws"),
//         store::Entry::String(String::from("foobar")),
//     )?;

//     store.create(String::from("nested"), store::Entry::Nested(store.clone()))?;
//     store.create(
//         String::from("binary"),
//         store::Entry::Binary(store.encrypt("foo")?),
//     )?;
//     Ok(store)
// }

// fn to_entry(path: &[String]) -> (String, store::Entry) {
//     path[1..].r
// }

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    let arguments = args::Args::parse();
    // println!("{:?}", arguments);

    let mut store = if let Some(source) = arguments.store() {
        match source {
            args::Source::File(path) => {
                let data = read_from_file(path)?;
                let password = rpassword::prompt_password_stderr("Password: ")?;
                store::Store::decrypt(&data, password)?
            }
            args::Source::S3(path) => {
                anyhow::bail!("S3 not yet implemented")
            }
        }
    } else {
        store::Store::new()
    };

    match arguments.action() {
        args::Action::Create(entry) => println!("Create => {:?}", entry),
        args::Action::Read(path) => {
            let json = json::Entry::from(ops::read(&store, path.path())?.clone());
            println!("{}", serde_json::to_string(&json)?);
        }
        args::Action::Update(entry) => println!("Update => {:?}", entry),
        args::Action::Delete(path) => ops::delete(&mut store, path.path())?,
        args::Action::Print(print) => {
            let store = json::Store::from(store);

            let json = if print.pretty() {
                serde_json::to_string_pretty(&store)?
            } else {
                serde_json::to_string(&store)?
            };

            println!("{}", json);
        }
    }

    Ok(())
}
