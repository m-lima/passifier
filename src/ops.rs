trait PathValidator {
    fn valid(&self) -> anyhow::Result<&Self>;
}

impl PathValidator for &[String] {
    fn valid(&self) -> anyhow::Result<&Self> {
        if self.is_empty() {
            Err(anyhow::anyhow!("Path is empty"))
        } else {
            Ok(self)
        }
    }
}

fn navigate<'r, 'p>(
    root: &'r mut store::Store,
    path: &'p [String],
) -> (&'r mut store::Store, &'p String, &'p [String]) {
    if path.len() == 1 {
        return (root, &path[0], &[]);
    }

    if let Some(store::Entry::Nested(_)) = root.read(&path[0]) {
        if let Some(store::Entry::Nested(inner)) = root.get(&path[0]) {
            return navigate(inner, &path[1..]);
        } else {
            unreachable!();
        }
    }

    (root, &path[0], &path[1..])
}

pub fn create(root: &mut store::Store, path: &[String], entry: store::Entry) -> anyhow::Result<()> {
    fn to_entry(path: &[String], entry: store::Entry) -> store::Entry {
        if path.is_empty() {
            return entry;
        }

        path.iter().rev().fold(entry, |acc, curr| {
            let mut store = store::Store::new();
            store.create(String::from(curr), acc).unwrap();
            store::Entry::Nested(store)
        })
    }

    fn create_inner<'r, 'p>(
        root: &'r mut store::Store,
        path: &'p [String],
    ) -> (&'r mut store::Store, &'p [String]) {
        if path.len() == 1 {
            return (root, path);
        }

        if let Some(store::Entry::Nested(_)) = root.read(&path[0]) {
            if let Some(store::Entry::Nested(inner)) = root.get(&path[0]) {
                return create_inner(inner, &path[1..]);
            } else {
                unreachable!();
            }
        }

        (root, path)
    }

    let (root, rest) = create_inner(root, path.valid()?);
    let entry = to_entry(&rest[1..], entry);
    root.create(rest[0].to_owned(), entry)?;
    Ok(())
}

pub fn read<'a>(root: &'a store::Store, path: &[String]) -> anyhow::Result<&'a store::Entry> {
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

    read_inner(root, path.valid()?).ok_or_else(|| anyhow::anyhow!("Not found"))
}

pub fn delete(root: &mut store::Store, path: &[String]) -> anyhow::Result<()> {
    fn delete_inner(root: &mut store::Store, path: &[String]) -> anyhow::Result<bool> {
        fn delete_nested(root: &mut store::Store, path: &[String]) -> anyhow::Result<bool> {
            match root.get(&path[0]) {
                Some(store::Entry::Nested(inner)) => delete_inner(inner, &path[1..]),
                Some(_) => anyhow::bail!("Invalid path"),
                None => anyhow::bail!("Not found"),
            }
        }

        if path.len() == 1 || delete_nested(root, path)? {
            root.delete(&path[0])?;
            Ok(root.secrets().next().is_none())
        } else {
            Ok(false)
        }
    }

    delete_inner(root, path.valid()?).map(|_| ())
}

#[cfg(test)]
mod tests {
    static MAP: &str = r#"{
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
        (e $string:literal) => {
            store::Entry::String(String::from($string))
        };
    }

    macro_rules! path {
        ($($string:literal),*) => {
            &[$(own!($string)),*]
        };
    }

    macro_rules! parse {
        ($string:expr) => {
            serde_json::from_str::<super::super::json::Store>($string)
                .unwrap()
                .into()
        };
        (e $string:literal) => {
            store::Entry::Nested(parse!($string))
        };
    }

    fn make_store() -> store::Store {
        parse!(MAP)
    }

    #[test]
    fn create() {
        let mut store = store::Store::new();

        super::create(&mut store, path!["new"], own!(e "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value"}"#));

        super::create(&mut store, path!["foo"], own!(e "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value","foo":"new_value"}"#));

        super::create(&mut store, path!["nested"], parse!(e "{}")).unwrap();
        assert_eq!(
            store,
            parse!(r#"{"new":"new_value","foo":"new_value","nested":{}}"#)
        );

        super::create(&mut store, path!["nested", "inner", "foo"], parse!(e "{}")).unwrap();
        assert_eq!(
            store,
            parse!(r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":{}}}}"#)
        );

        super::create(&mut store, path!["nested", "other", "foo"], parse!(e "{}")).unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":{}},"other":{"foo":{}}}}"#
            )
        );

        super::create(
            &mut store,
            path!["nested", "other", "foo", "deep", "deeper"],
            own!(e "here"),
        )
        .unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":{}},"other":{"foo":{"deep":{"deeper":"here"}}}}}"#
            )
        );
    }

    #[test]
    fn create_conflict() {
        let mut store = make_store();

        assert!(super::create(&mut store, path!["binary"], own!(e "new_value")).is_err());
        assert!(super::create(&mut store, path!["nested"], own!(e "new_value")).is_err());
        assert!(
            super::create(&mut store, path!["nested", "sibling"], own!(e "new_value")).is_err()
        );
        assert!(super::create(
            &mut store,
            path!["nested", "sibling", "deep"],
            own!(e "new_value")
        )
        .is_err());
    }

    #[test]
    fn read() {
        let store = make_store();

        assert_eq!(
            super::read(&store, path!["binary"]).unwrap(),
            &store::Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            super::read(&store, path!["nested"]).unwrap(),
            store.read("nested").unwrap()
        );

        assert_eq!(
            super::read(&store, path!["nested", "inner"]).unwrap(),
            &parse!(e r#"{"deep":{"foo":"bar"}}"#)
        );

        assert_eq!(
            super::read(&store, path!["nested", "inner", "deep"]).unwrap(),
            &parse!(e r#"{"foo":"bar"}"#)
        );

        assert_eq!(
            super::read(&store, path!["nested", "inner", "deep", "foo"]).unwrap(),
            &own!(e "bar")
        );

        assert_eq!(
            super::read(&store, path!["nested", "sibling"]).unwrap(),
            &own!(e "inner_sibling")
        );

        assert_eq!(
            super::read(&store, path!["binary"]).unwrap(),
            &store::Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            super::read(&store, path!["sibling"]).unwrap(),
            &own!(e "outer_sibling")
        );
    }

    #[test]
    fn read_not_found() {
        let store = make_store();

        assert!(super::read(&store, path!["bla"]).is_err());
        assert!(super::read(&store, path!["binary", "245"]).is_err());
        assert!(super::read(&store, path!["nested", "bla"]).is_err());
        assert!(super::read(&store, path!["nested", "bla", "foo"]).is_err());
        assert!(super::read(&store, path!["nested", "inner", "bla"]).is_err());
        assert!(super::read(&store, path!["nested", "inner", "bla", "deep"]).is_err());
        assert!(super::read(&store, path!["nested", "inner", "deep", "bla"]).is_err());
        assert!(super::read(&store, path!["nested", "inner", "deep", "foo", "bla"]).is_err());
        assert!(super::read(&store, path![""]).is_err());
    }

    fn delete_helper(path: &[String], expected: &'static str) {
        let mut store = make_store();
        super::delete(&mut store, path).unwrap_or_else(|err| panic!("{:?}: {}", path, err));
        assert_eq!(store, parse!(expected), "{:?}", path);
    }

    #[test]
    fn delete() {
        delete_helper(
            path!["binary"],
            r#"{
                 "nested": {
                   "inner": {
                     "deep": {
                       "foo": "bar"
                     }
                   },
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#,
        );

        delete_helper(
            path!["sibling"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "inner": {
                     "deep": {
                       "foo": "bar"
                     }
                   },
                   "sibling": "inner_sibling"
                 }
               }"#,
        );

        delete_helper(
            path!["nested"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "sibling": "outer_sibling"
               }"#,
        );

        delete_helper(
            path!["nested", "sibling"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "inner": {
                     "deep": {
                       "foo": "bar"
                     }
                   }
                 },
                 "sibling": "outer_sibling"
               }"#,
        );

        delete_helper(
            path!["nested", "inner"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#,
        );

        delete_helper(
            path!["nested", "inner", "deep"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#,
        );

        delete_helper(
            path!["nested", "inner", "deep", "foo"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#,
        );
    }

    #[test]
    fn delete_not_found() {
        let mut store = make_store();

        assert!(super::delete(&mut store, path!["bla"]).is_err());
        assert!(super::delete(&mut store, path!["binary", "245"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "bla"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "bla", "foo"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "inner", "bla"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "inner", "bla", "deep"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "inner", "deep", "bla"]).is_err());
        assert!(super::delete(&mut store, path!["nested", "inner", "deep", "foo", "bla"]).is_err());
        assert!(super::delete(&mut store, path![""]).is_err());
    }
}
