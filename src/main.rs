// #![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod json;

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

fn navigate<'r, 'p>(
    root: &'r mut store::Store,
    path: &'p [String],
) -> (&'r mut store::Store, &'p [String]) {
    if path.is_empty() {
        return (root, path);
    }

    if let Some(store::Entry::Nested(_)) = root.read(&path[0]) {
        if let Some(store::Entry::Nested(inner)) = root.get(&path[0]) {
            return navigate(inner, &path[1..]);
        } else {
            unreachable!();
        }
    }

    (root, path)
}

fn read<'a>(root: &'a store::Store, path: &[String]) -> anyhow::Result<&'a store::Entry> {
    fn read_inner<'a>(root: &'a store::Store, path: &[String]) -> Option<&'a store::Entry> {
        root.read(&path[0]).and_then(|entry| {
            if path.len() == 1 {
                Some(entry)
            } else if let store::Entry::Nested(inner) = entry {
                read_inner(inner, &path[1..])
            } else {
                None
            }
        })
    }

    read_inner(root, path).ok_or_else(|| anyhow::anyhow!("Not found"))
}

// fn delete

fn main() -> anyhow::Result<()> {
    use clap::Clap;
    let arguments = args::Args::parse();
    // println!("{:?}", arguments);

    let mut store = if let Some(args::Source::File(path)) = arguments.store() {
        let data = read_from_file(path)?;
        let password = rpassword::prompt_password_stderr("Password: ")?;
        store::Store::decrypt(&data, password)?
    } else {
        store::Store::new()
    };

    match arguments.action() {
        args::Action::Create(entry) => println!("Create => {:?}", entry),
        args::Action::Read(path) => {
            let json = json::Entry::from(read(&store, path.path())?.clone());
            println!("{}", serde_json::to_string(&json)?);
        }
        args::Action::Update(entry) => println!("Update => {:?}", entry),
        args::Action::Delete(path) => {
            let path = path.path();
            let (root, rest) = navigate(&mut store, &path[..path.len() - 1]);
            if rest.is_empty() {
                root.delete(&path[path.len() - 1]).unwrap();
            } else {
                println!("not found");
            }
        }
        args::Action::Print(print) => {
            let json_store = json::Store::from(store);
            println!(
                "{}",
                if print.pretty() {
                    serde_json::to_string_pretty(&json_store)?
                } else {
                    serde_json::to_string(&json_store)?
                }
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    static MAP: &str = r#"
{
  "binary": [ 245, 107, 95, 100 ],
  "nested": {
    "inner": {
      "deep": {
        "foo": "bar"
      }
    },
    "sibling": "inner_sibling"
  },
  "sibling": "outer_sibling"
}"#;

    macro_rules! own {
        ($string:literal) => {
            String::from($string)
        };
    }

    macro_rules! path {
        ($($string:literal),*) => {
            &[$(own!($string)),*]
        };
    }

    macro_rules! parse {
        ($string:expr) => {
            serde_json::from_str::<super::json::Store>($string)
                .unwrap()
                .into()
        };
    }

    fn make_store() -> store::Store {
        parse!(MAP)
    }

    #[test]
    fn read() {
        let store = make_store();

        assert_eq!(
            super::read(&store, path!("binary")).unwrap(),
            &store::Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            super::read(&store, path!("nested")).unwrap(),
            store.read("nested").unwrap()
        );

        assert_eq!(
            super::read(&store, path!("nested", "inner")).unwrap(),
            &store::Entry::Nested(parse!(r#"{"deep":{"foo":"bar"}}"#))
        );

        assert_eq!(
            super::read(&store, path!("nested", "inner", "deep")).unwrap(),
            &store::Entry::Nested(parse!(r#"{"foo":"bar"}"#))
        );

        assert_eq!(
            super::read(&store, path!("nested", "inner", "deep", "foo")).unwrap(),
            &store::Entry::String(own!("bar"))
        );

        assert_eq!(
            super::read(&store, path!("nested", "sibling")).unwrap(),
            &store::Entry::String(own!("inner_sibling"))
        );

        assert_eq!(
            super::read(&store, path!("binary")).unwrap(),
            &store::Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            super::read(&store, path!("sibling")).unwrap(),
            &store::Entry::String(own!("outer_sibling"))
        );
    }

    #[test]
    fn read_not_found() {
        let store = make_store();

        assert!(super::read(&store, path!("bla")).is_err());
        assert!(super::read(&store, path!("binary", "245")).is_err());
        assert!(super::read(&store, path!("nested", "bla")).is_err());
        assert!(super::read(&store, path!("nested", "bla", "foo")).is_err());
        assert!(super::read(&store, path!("nested", "inner", "bla")).is_err());
        assert!(super::read(&store, path!("nested", "inner", "bla", "deep")).is_err());
        assert!(super::read(&store, path!("nested", "inner", "deep", "bla")).is_err());
        assert!(super::read(&store, path!("nested", "inner", "deep", "foo", "bla")).is_err());
        assert!(super::read(&store, path!("")).is_err());
    }

    // #[test]
    // fn delete() {
    //     {
    //         let mut store = make_store();
    //         super::delete(&store, path!("binary")).unwrap();

    //         assert_eq!(store, parse!(""));
    //     }

    //     assert_eq!(
    //         super::get(&store, path!("nested")).unwrap(),
    //         store.read("nested").unwrap()
    //     );

    //     assert_eq!(
    //         super::get(&store, path!("nested", "inner")).unwrap(),
    //         &store::Entry::Nested(parse!(r#"{"foo":"bar"}"#))
    //     );

    //     assert_eq!(
    //         super::get(&store, path!("nested", "inner", "foo")).unwrap(),
    //         &store::Entry::String(own!("bar"))
    //     );

    //     assert_eq!(
    //         super::get(&store, path!("nested", "sibling")).unwrap(),
    //         &store::Entry::String(own!("inner_sibling"))
    //     );

    //     assert_eq!(
    //         super::get(&store, path!("binary")).unwrap(),
    //         &store::Entry::Binary(vec![245, 107, 95, 100])
    //     );

    //     assert_eq!(
    //         super::get(&store, path!("sibling")).unwrap(),
    //         &store::Entry::String(own!("outer_sibling"))
    //     );
    // }
}
