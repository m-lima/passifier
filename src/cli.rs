use super::args;
use super::io;
use super::store;

pub fn run(
    input: Option<args::Source>,
    action: args::Action,
    mut output: Option<Option<args::Source>>,
) -> anyhow::Result<()> {
    let mut store = io::load(&input, make_crypter)?;

    let node = match action {
        args::Action::Create(args::Create { secret, path, .. }) => {
            store.create(path.as_ref(), secret);
            store::Node::from(store.into())
        }
        args::Action::Read(args::Read { path, pretty }) => {
            let node = store.take(path.as_ref().map_or(&[], AsRef::as_ref))?;
            if output.is_none() {
                output = Some(None);
            }
            store::Node::from(node)
        }
        args::Action::Update(args::Update { secret, path, .. }) => {
            store.update(path.as_ref(), secret);
            store::Node::from(store.into())
        }
        args::Action::Delete(args::Delete { path, .. }) => {
            store.delete(path.as_ref());
            store::Node::from(store.into())
        }
    };

    if let Some(output) = output {
        if let Some(output) = output {
        } else {
            io::save(
        }
    } else {
        let json = serde_json::to_string(node)?;
        println!("{}", json);
    }
}

pub fn make_crypter() -> Option<crypter::Crypter> {
    // Ok to swallow the error because it only errors on stdio
    let password = rpassword::prompt_password_stderr("Password: ").ok()?;
    Some(crypter::Crypter::new(password))
}
