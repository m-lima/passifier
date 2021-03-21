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

pub(super) fn read(store: &Store, read: args::Read) -> anyhow::Result<&Node> {
    let args::Read { path, pretty } = read;
    store
        .get_from_iter(path.iter())
        .ok_or_else(|| anyhow::anyhow!("Not found"))
        .and_then(|node| {
            print_node(node, pretty)?;
            Ok(node)
        })
}

pub(super) fn update(store: &mut Store, write: args::Write) -> anyhow::Result<()> {
    let args::Write { path, secret } = write;
    if !store.contains_path_iter(path.iter()) {
        anyhow::bail!("Not found");
    } else if should_delete(&secret) {
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
    fn clean_up(store: &mut Store, path: &[String]) -> bool {
        if store.is_empty() {
            return true;
        }

        if let Some(true) = store.get_mut(&path[0]).map(|node| {
            if let Node::Branch(ref mut branch) = *node {
                clean_up(branch, &path[1..])
            } else {
                unreachable!("impossible to not be a branch");
            }
        }) {
            store.remove(&path[0]);
        }
        store.is_empty()
    }

    store
        .remove_from_iter(path.iter())
        .ok_or_else(|| anyhow::anyhow!("Not found"))?;

    if clean_up(store, &path.0) {
        store.remove(&path.0[0]);
    }

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

    macro_rules! path {
        ($($string:literal),*) => {
            args::Path(vec![$(String::from($string)),*])
        };
    }

    macro_rules! parse {
        ($string:expr) => {
            serde_json::from_str::<Store>($string).unwrap().into()
        };
    }

    macro_rules! leaf {
        ($string:literal) => {
            Node::Leaf(Entry::String(String::from($string)))
        };

        ($($binary:literal),*) => {
            Node::Leaf(Entry::Binary(vec![$($binary),*]))
        };
    }

    macro_rules! branch {
        ($string:literal) => {
            Node::Branch(parse!($string))
        };
    }

    macro_rules! create {
        ([$path:literal], $value:literal) => {
            args::Write { path: path!($path), secret: leaf!($value) }
        };

        ([$($path:literal),*], $value:literal) => {
            args::Write { path: path!($($path),*), secret: leaf!($value) }
        };

        ([$path:literal]) => {
            args::Write { path: path!($path), secret: branch!("{}") }
        };
    }

    macro_rules! read {
        ($($path:literal),*) => {
            args::Read{ path: path!($($path),*), pretty: false }
        };
    }

    macro_rules! update {
        ([$($path:literal),*]) => {
            args::Write { path: path!($($path),*), secret: branch!("{}") }
        };

        ([$path:literal], $value:literal) => {
            args::Write { path: path!($path), secret: leaf!($value) }
        };

        ([$($path:literal),*], $value:literal) => {
            args::Write { path: path!($($path),*), secret: leaf!($value) }
        };
    }

    macro_rules! delete {
        ($($path:literal),*) => {
            args::Delete{ path: path!($($path),*) }
        };
    }

    fn make_store() -> Store {
        parse!(MAP)
    }

    #[test]
    fn create() {
        use super::create;

        let mut store = Store::new();

        create(&mut store, create!(["new"], "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value"}"#));

        create(&mut store, create!(["foo"], "new_value")).unwrap();
        assert_eq!(store, parse!(r#"{"new":"new_value","foo":"new_value"}"#));

        create(&mut store, create!(["nested", "inner", "foo"], "bar")).unwrap();
        assert_eq!(
            store,
            parse!(r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":"bar"}}}"#)
        );

        create(
            &mut store,
            create!(["nested", "other", "foo", "deep", "deeper"], "here"),
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

        assert!(create(&mut store, create!(["binary"], "new_value")).is_err());
        assert!(create(&mut store, create!(["nested"], "new_value")).is_err());
        assert!(create(&mut store, create!(["nested", "sibling"], "new_value")).is_err());
        assert!(create(
            &mut store,
            create!(["nested", "sibling", "deep"], "new_value")
        )
        .is_err());
    }

    #[test]
    fn create_empty() {
        use super::create;
        let mut store = make_store();
        assert!(create(&mut store, create!(["nested"])).is_err());
    }

    #[test]
    fn read() {
        use super::read;

        let store = make_store();

        assert_eq!(
            read(&store, read!["binary"]).unwrap(),
            &leaf![245, 107, 95, 100]
        );

        assert_eq!(
            read(&store, read!["nested"]).unwrap(),
            store.get("nested").unwrap()
        );

        assert_eq!(
            read(&store, read!["nested", "inner"]).unwrap(),
            &branch!(r#"{"deep":{"foo":"bar"}}"#)
        );

        assert_eq!(
            read(&store, read!["nested", "inner", "deep"]).unwrap(),
            &branch!(r#"{"foo":"bar"}"#)
        );

        assert_eq!(
            read(&store, read!["nested", "inner", "deep", "foo"]).unwrap(),
            &leaf!("bar")
        );

        assert_eq!(
            read(&store, read!["nested", "sibling"]).unwrap(),
            &leaf!("inner_sibling")
        );

        assert_eq!(
            read(&store, read!["binary"]).unwrap(),
            &leaf![245, 107, 95, 100]
        );

        assert_eq!(
            read(&store, read!["sibling"]).unwrap(),
            &leaf!("outer_sibling")
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
        update(&mut store, update!(["binary"], "new")).unwrap();
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
            update!(["nested", "inner", "deep", "foo"], "new"),
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
        update(&mut store, update!(["nested"], "new")).unwrap();
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
        macro_rules! update_empty {
            ($($path:literal),*) => {{
                let mut updated = make_store();
                let mut deleted = make_store();
                super::update(&mut updated, update!([$($path),*])).unwrap();
                super::delete(&mut deleted, delete!($($path),*)).unwrap();
                assert_eq!(updated, deleted);
            }};
        }

        update_empty!["binary"];
        update_empty!["sibling"];
        update_empty!["nested"];
        update_empty!["nested", "sibling"];
        update_empty!["nested", "inner"];
        update_empty!["nested", "inner", "deep"];
        update_empty!["nested", "inner", "deep", "foo"];
    }

    #[test]
    fn update_not_found() {
        use super::update;

        let mut store = make_store();

        assert!(update(&mut store, update!(["bla"], "")).is_err());
        assert!(update(&mut store, update!(["binary", "245"], "")).is_err());
        assert!(update(&mut store, update!(["nested", "bla"], "")).is_err());
        assert!(update(&mut store, update!(["nested", "bla", "foo"], "")).is_err());
        assert!(update(&mut store, update!(["nested", "inner", "bla"], "")).is_err());
        assert!(update(&mut store, update!(["nested", "inner", "bla", "deep"], "")).is_err());
        assert!(update(&mut store, update!(["nested", "inner", "deep", "bla"], "")).is_err());
        assert!(update(
            &mut store,
            update!(["nested", "inner", "deep", "foo", "bla"], "")
        )
        .is_err());
        assert!(update(&mut store, update!([""], "")).is_err());
    }

    #[test]
    fn delete() {
        macro_rules! test_delete {
            ([$($path:literal),*], $expected:literal) => {{
                let mut store = make_store();
                super::delete(&mut store, delete!($($path),*)).unwrap();
                assert_eq!(store, parse!($expected));
            }};

        }

        test_delete!(
            ["binary"],
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

        test_delete!(
            ["sibling"],
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

        test_delete!(
            ["nested"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "sibling": "outer_sibling"
               }"#
        );

        test_delete!(
            ["nested", "sibling"],
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

        test_delete!(
            ["nested", "inner"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );

        test_delete!(
            ["nested", "inner", "deep"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );

        test_delete!(
            ["nested", "inner", "deep", "foo"],
            r#"{
                 "binary": [ 245, 107, 95, 100 ],
                 "nested": {
                   "sibling": "inner_sibling"
                 },
                 "sibling": "outer_sibling"
               }"#
        );

        let mut store = parse!(r#"{"one":{"two":{"three":"value"}}}"#);
        assert!(super::delete(&mut store, delete!["one", "two", "three"]).is_ok());
        assert_eq!(store, parse!("{}"));
    }

    #[test]
    fn delete_not_found() {
        use super::delete;

        let mut store = make_store();

        assert!(delete(&mut store, delete!["bla"]).is_err());
        assert!(delete(&mut store, delete!["binary", "245"]).is_err());
        assert!(delete(&mut store, delete!["nested", "bla"]).is_err());
        assert!(delete(&mut store, delete!["nested", "bla", "foo"]).is_err());
        assert!(delete(&mut store, delete!["nested", "inner", "bla"]).is_err());
        assert!(delete(&mut store, delete!["nested", "inner", "bla", "deep"]).is_err());
        assert!(delete(&mut store, delete!["nested", "inner", "deep", "bla"]).is_err());
        assert!(delete(&mut store, delete!["nested", "inner", "deep", "foo", "bla"]).is_err());
        assert!(delete(&mut store, delete![""]).is_err());
    }
}
