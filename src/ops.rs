use super::{args, Node, Store};

pub(super) fn create(store: &mut Store, write: args::Write) -> anyhow::Result<()> {
    let args::Write { path, secret } = write;
    if should_delete(&secret) {
        anyhow::bail!("Empty secret");
    } else if store.contains_path_iter(path.iter()) {
        anyhow::bail!("Conflict");
    } else {
        store.insert_into_iter(path.iter(), secret);
    }

    Ok(())
}

pub(super) fn read(store: &Store, read: args::Read) -> anyhow::Result<()> {
    let args::Read { path, pretty } = read;
    store
        .get_from_iter(path.iter())
        .ok_or_else(|| anyhow::anyhow!("Not found"))
        .and_then(|node| print_node(node, pretty))
}

pub(super) fn update(store: &mut Store, write: args::Write) -> anyhow::Result<()> {
    let args::Write { path, secret } = write;
    if should_delete(&secret) {
        delete_path(store, &path)?;
    } else {
        store.insert_into_iter(path.iter(), secret);
    }

    Ok(())
}

pub(super) fn delete(store: &mut Store, delete: args::Delete) -> anyhow::Result<()> {
    let args::Delete { path } = delete;
    delete_path(store, &path)
}

pub(super) fn print(store: Store, print: &args::Print) -> anyhow::Result<()> {
    print_node(&Node::Branch(store), print.pretty)
}

pub(super) fn delete_path(store: &mut Store, path: &args::Path) -> anyhow::Result<()> {
    store
        .remove_from_iter(path.iter())
        .ok_or_else(|| anyhow::anyhow!("Not found"))?;

    Ok(())
}

fn print_node(node: &Node, pretty: bool) -> anyhow::Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(&node)?
    } else {
        serde_json::to_string(&node)?
    };

    println!("{}", json);
    Ok(())
}

fn should_delete(secret: &Node) -> bool {
    if let Node::Branch(ref branch) = *secret {
        if branch.is_empty() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::Entry;
    use super::{args, Node, Store};

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
            Node::Leaf(Entry::String(String::from($string)))
        };
    }

    macro_rules! path {
        ($($string:literal),*) => {
            &args::Path(vec![$(own!($string)),*])
        };
    }

    macro_rules! read {
        ($($string:literal),*) => {
            args::Read{ path: args::Path::new(vec![$(own!($string)),*]), pretty: false }
        };
    }

    macro_rules! parse {
        ($string:expr) => {
            serde_json::from_str::<Store>($string).unwrap().into()
        };
        (e $string:literal) => {
            Node::Branch(parse!($string))
        };
    }

    macro_rules! update_empty {
        ($path:expr) => {{
            let mut updated = make_store();
            let mut deleted = make_store();
            super::update(&mut updated, $path, parse!(e "{}")).unwrap();
            super::delete(&mut deleted, $path).unwrap();
            assert_eq!(updated, deleted);
        }};
    }

    macro_rules! delete {
        ($path:expr, $expected:literal) => {{
            let mut store = make_store();
            super::delete(&mut store, $path).unwrap();
            assert_eq!(store, parse!($expected));
        }};
    }

    fn make_store() -> Store {
        parse!(MAP)
    }

    #[test]
    fn create() {
        use super::create;

        let mut store = store::Store::new();

        create(&mut store, path!["new"], own!(e "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value"}"#));

        create(&mut store, path!["foo"], own!(e "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value","foo":"new_value"}"#));

        create(&mut store, path!["nested", "inner", "foo"], own!(e "bar")).unwrap();
        assert_eq!(
            store,
            parse!(r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":"bar"}}}"#)
        );

        create(
            &mut store,
            path!["nested", "other", "foo", "deep", "deeper"],
            own!(e "here"),
        )
        .unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":"bar"},"other":{"foo":{"deep":{"deeper":"here"}}}}}"#
            )
        );
    }

    #[test]
    fn create_conflict() {
        use super::create;

        let mut store = make_store();

        assert!(create(&mut store, path!["binary"], own!(e "new_value")).is_err());
        assert!(create(&mut store, path!["nested"], own!(e "new_value")).is_err());
        assert!(create(&mut store, path!["nested", "sibling"], own!(e "new_value")).is_err());
        assert!(create(
            &mut store,
            path!["nested", "sibling", "deep"],
            own!(e "new_value")
        )
        .is_err());
    }

    #[test]
    fn create_empty() {
        use super::create;
        let mut store = make_store();
        assert!(create(&mut store, path!["nested"], parse!(e "{}")).is_err());
    }

    #[test]
    fn read() {
        use super::read;

        let store = make_store();

        assert_eq!(
            read(&store, read!["binary"]).unwrap(),
            &Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            read(&store, read!["nested"]).unwrap(),
            store.read("nested").unwrap()
        );

        assert_eq!(
            read(&store, read!["nested", "inner"]).unwrap(),
            &parse!(e r#"{"deep":{"foo":"bar"}}"#)
        );

        assert_eq!(
            read(&store, read!["nested", "inner", "deep"]).unwrap(),
            &parse!(e r#"{"foo":"bar"}"#)
        );

        assert_eq!(
            read(&store, read!["nested", "inner", "deep", "foo"]).unwrap(),
            &own!(e "bar")
        );

        assert_eq!(
            read(&store, read!["nested", "sibling"]).unwrap(),
            &own!(e "inner_sibling")
        );

        assert_eq!(
            read(&store, read!["binary"]).unwrap(),
            &store::Entry::Binary(vec![245, 107, 95, 100])
        );

        assert_eq!(
            read(&store, read!["sibling"]).unwrap(),
            &own!(e "outer_sibling")
        );
    }

    #[test]
    fn read_not_found() {
        use super::read;

        let store = make_store();

        assert!(read(&store, read!["bla"]).is_err());
        assert!(read(&store, read!["binary", "245"]).is_err());
        assert!(read(&store, read!["nested", "bla"]).is_err());
        assert!(read(&store, read!["nested", "bla", "foo"]).is_err());
        assert!(read(&store, read!["nested", "inner", "bla"]).is_err());
        assert!(read(&store, read!["nested", "inner", "bla", "deep"]).is_err());
        assert!(read(&store, read!["nested", "inner", "deep", "bla"]).is_err());
        assert!(read(&store, read!["nested", "inner", "deep", "foo", "bla"]).is_err());
        assert!(read(&store, read![""]).is_err());
    }

    #[test]
    fn update() {
        use super::update;

        let mut store = make_store();

        // update top level
        update(&mut store, path!["binary"], own!(e "new")).unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{
                     "binary": "new",
                     "nested": {
                       "inner": {
                         "deep": {
                           "foo": "bar"
                         }
                       },
                       "sibling": "inner_sibling"
                     },
                     "sibling": "outer_sibling"
                   }"#
            )
        );

        // update deep
        update(
            &mut store,
            path!["nested", "inner", "deep", "foo"],
            own!(e "new"),
        )
        .unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{
                     "binary": "new",
                     "nested": {
                       "inner": {
                         "deep": {
                           "foo": "new"
                         }
                       },
                       "sibling": "inner_sibling"
                     },
                     "sibling": "outer_sibling"
                   }"#
            )
        );

        // update root of deep tree
        update(&mut store, path!["nested"], own!(e "new")).unwrap();
        assert_eq!(
            store,
            parse!(
                r#"{
                     "binary": "new",
                     "nested": "new",
                     "sibling": "outer_sibling"
                   }"#
            )
        );
    }

    #[test]
    fn update_empty_just_deletes() {
        update_empty!(path!["binary"]);
        update_empty!(path!["sibling"]);
        update_empty!(path!["nested"]);
        update_empty!(path!["nested", "sibling"]);
        update_empty!(path!["nested", "inner"]);
        update_empty!(path!["nested", "inner", "deep"]);
        update_empty!(path!["nested", "inner", "deep", "foo"]);
    }

    #[test]
    fn update_not_found() {
        use super::update;

        let mut store = make_store();

        assert!(update(&mut store, path!["bla"], own!(e "")).is_err());
        assert!(update(&mut store, path!["binary", "245"], own!(e "")).is_err());
        assert!(update(&mut store, path!["nested", "bla"], own!(e "")).is_err());
        assert!(update(&mut store, path!["nested", "bla", "foo"], own!(e "")).is_err());
        assert!(update(&mut store, path!["nested", "inner", "bla"], own!(e "")).is_err());
        assert!(update(
            &mut store,
            path!["nested", "inner", "bla", "deep"],
            own!(e "")
        )
        .is_err());
        assert!(update(
            &mut store,
            path!["nested", "inner", "deep", "bla"],
            own!(e "")
        )
        .is_err());
        assert!(update(
            &mut store,
            path!["nested", "inner", "deep", "foo", "bla"],
            own!(e "")
        )
        .is_err());
        assert!(update(&mut store, path![""], own!(e "")).is_err());
    }

    #[test]
    fn delete() {
        delete!(
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
               }"#
        );

        delete!(
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
               }"#
        );

        delete!(
            path!["nested"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "sibling": "outer_sibling"
               }"#
        );

        delete!(
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
               }"#
        );

        delete!(
            path!["nested", "inner"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );

        delete!(
            path!["nested", "inner", "deep"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );

        delete!(
            path!["nested", "inner", "deep", "foo"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );
    }

    #[test]
    fn delete_not_found() {
        use super::delete;

        let mut store = make_store();

        assert!(delete(&mut store, path!["bla"]).is_err());
        assert!(delete(&mut store, path!["binary", "245"]).is_err());
        assert!(delete(&mut store, path!["nested", "bla"]).is_err());
        assert!(delete(&mut store, path!["nested", "bla", "foo"]).is_err());
        assert!(delete(&mut store, path!["nested", "inner", "bla"]).is_err());
        assert!(delete(&mut store, path!["nested", "inner", "bla", "deep"]).is_err());
        assert!(delete(&mut store, path!["nested", "inner", "deep", "bla"]).is_err());
        assert!(delete(&mut store, path!["nested", "inner", "deep", "foo", "bla"]).is_err());
        assert!(delete(&mut store, path![""]).is_err());
    }
}
