pub(super) type NestedMap = nested_map::NestedMap<String, Entry>;
pub(super) type Node = nested_map::Node<String, Entry>;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub(super) struct Store(NestedMap);

impl Store {
    pub fn new() -> Self {
        Self::from(NestedMap::new())
    }

    pub fn from(map: NestedMap) -> Self {
        Self(map)
    }

    pub fn create(&mut self, path: &[String], secret: Node) -> anyhow::Result<()> {
        if should_delete(&secret) {
            anyhow::bail!("Empty secret");
        } else if is_new_entry(self, &path) {
            self.0.insert_into_iter(path.iter(), secret);
        } else {
            anyhow::bail!("Conflict");
        }

        Ok(())
    }

    pub fn read(&self, path: &[String]) -> anyhow::Result<&Node> {
        self.get_from_iter(path.iter())
            .ok_or_else(|| anyhow::anyhow!("Not found"))
    }

    pub fn take(mut self, path: &[String]) -> anyhow::Result<Node> {
        self.0
            .remove_from_iter(path.iter())
            .ok_or_else(|| anyhow::anyhow!("Not found"))
    }

    pub fn update(&mut self, path: &[String], secret: Node) -> anyhow::Result<()> {
        if !self.contains_path_iter(path.iter()) {
            anyhow::bail!("Not found");
        } else if should_delete(&secret) {
            delete_path(self, &path)?;
        } else {
            self.0.insert_into_iter(path.iter(), secret);
        }

        Ok(())
    }

    pub fn delete(&mut self, path: &[String]) -> anyhow::Result<()> {
        delete_path(self, &path)
    }
}

impl std::ops::Deref for Store {
    type Target = NestedMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::Into<NestedMap> for Store {
    fn into(self) -> NestedMap {
        self.0
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub(super) enum Entry {
    String(String),
    Binary(Vec<u8>),
}

impl From<String> for Entry {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<&str> for Entry {
    fn from(string: &str) -> Self {
        Self::String(String::from(string))
    }
}

impl From<Vec<u8>> for Entry {
    fn from(vec: Vec<u8>) -> Self {
        Self::Binary(vec)
    }
}

impl From<&[u8]> for Entry {
    fn from(vec: &[u8]) -> Self {
        Self::Binary(vec.iter().map(Clone::clone).collect())
    }
}

fn delete_path(store: &mut Store, path: &[String]) -> anyhow::Result<()> {
    fn clean_up(map: &mut NestedMap, path: &[String]) -> bool {
        if map.is_empty() {
            return true;
        }

        if let Some(true) = map.get_mut(&path[0]).map(|node| {
            if let Node::Branch(ref mut branch) = *node {
                clean_up(branch, &path[1..])
            } else {
                unreachable!("impossible to not be a branch");
            }
        }) {
            map.remove(&path[0]);
        }
        map.is_empty()
    }

    store
        .0
        .remove_from_iter(path.iter())
        .ok_or_else(|| anyhow::anyhow!("Not found"))?;

    if clean_up(&mut store.0, path) {
        store.0.remove(&path[0]);
    }

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

fn is_new_entry(store: &Store, path: &[String]) -> bool {
    fn inspect(map: &NestedMap, path: &[String]) -> bool {
        if path.is_empty() {
            false
        } else {
            match map.get(&path[0]) {
                None => true,
                Some(Node::Leaf(_)) => false,
                Some(Node::Branch(branch)) => inspect(branch, &path[1..]),
            }
        }
    }

    inspect(store, path)
}

// #[cfg(test)]
// mod tests {
//     use super::Entry;
//     use super::{NestedMap, Node, Store};

//     static MAP: &str = r#"{
//                             "binary": [ 245, 107, 95, 100 ],
//                             "nested": {
//                               "inner": {
//                                 "deep": {
//                                   "foo": "bar"
//                                 }
//                               },
//                               "sibling": "inner_sibling"
//                             },
//                             "sibling": "outer_sibling"
//                           }"#;

//     macro_rules! path {
//         ($($string:literal),*) => {
//             args::Path(vec![$(String::from($string)),*])
//         };
//     }

//     macro_rules! parse {
//         ($string:expr) => {
//             serde_json::from_str::<NestedMap>($string).unwrap().into()
//         };
//     }

//     macro_rules! store {
//         ($string:expr) => {
//             Store(parse!($string))
//         };
//     }

//     macro_rules! leaf {
//         ($string:literal) => {
//             Node::Leaf(Entry::String(String::from($string)))
//         };

//         ($($binary:literal),*) => {
//             Node::Leaf(Entry::Binary(vec![$($binary),*]))
//         };
//     }

//     macro_rules! branch {
//         ($string:literal) => {
//             Node::Branch(parse!($string))
//         };
//     }

//     macro_rules! create {
//         ([$path:literal], $value:literal) => {
//             args::Write { path: path!($path), secret: leaf!($value) }
//         };

//         ([$($path:literal),*], $value:literal) => {
//             args::Write { path: path!($($path),*), secret: leaf!($value) }
//         };

//         ([$path:literal]) => {
//             args::Write { path: path!($path), secret: branch!("{}") }
//         };
//     }

//     macro_rules! read {
//         ($($path:literal),*) => {
//             args::Read{ path: path!($($path),*), pretty: false }
//         };
//     }

//     macro_rules! update {
//         ([$($path:literal),*]) => {
//             args::Write { path: path!($($path),*), secret: branch!("{}") }
//         };

//         ([$path:literal], $value:literal) => {
//             args::Write { path: path!($path), secret: leaf!($value) }
//         };

//         ([$($path:literal),*], $value:literal) => {
//             args::Write { path: path!($($path),*), secret: leaf!($value) }
//         };
//     }

//     macro_rules! delete {
//         ($($path:literal),*) => {
//             args::Delete{ path: path!($($path),*) }
//         };
//     }

//     #[test]
//     fn create() {
//         let mut store = Store::new();

//         store.create(create!(["new"], "new_value")).unwrap();
//         assert_eq!(store, store!(r#"{"new":"new_value"}"#));

//         store.create(create!(["foo"], "new_value")).unwrap();
//         assert_eq!(store, store!(r#"{"new":"new_value","foo":"new_value"}"#));

//         store
//             .create(create!(["nested", "inner", "foo"], "bar"))
//             .unwrap();
//         assert_eq!(
//             store,
//             store!(r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":"bar"}}}"#)
//         );

//         store
//             .create(create!(
//                 ["nested", "other", "foo", "deep", "deeper"],
//                 "here"
//             ))
//             .unwrap();
//         assert_eq!(
//             store,
//             store!(
//                 r#"{"new":"new_value","foo":"new_value","nested":{"inner":{"foo":"bar"},"other":{"foo":{"deep":{"deeper":"here"}}}}}"#
//             )
//         );
//     }

//     #[test]
//     fn create_conflict() {
//         let mut store = store!(MAP);

//         assert!(store.create(create!(["binary"], "new_value")).is_err());
//         assert!(store.create(create!(["nested"], "new_value")).is_err());
//         assert!(store
//             .create(create!(["nested", "sibling"], "new_value"))
//             .is_err());
//         assert!(store
//             .create(create!(["nested", "sibling", "deep"], "new_value"))
//             .is_err());
//     }

//     #[test]
//     fn create_empty() {
//         let mut store = store!(MAP);
//         assert!(store.create(create!(["nested"])).is_err());
//     }

//     #[test]
//     fn read() {
//         let store = store!(MAP);

//         assert_eq!(
//             store.read(read!["binary"]).unwrap(),
//             &leaf![245, 107, 95, 100]
//         );

//         assert_eq!(
//             store.read(read!["nested"]).unwrap(),
//             store.get("nested").unwrap()
//         );

//         assert_eq!(
//             store.read(read!["nested", "inner"]).unwrap(),
//             &branch!(r#"{"deep":{"foo":"bar"}}"#)
//         );

//         assert_eq!(
//             store.read(read!["nested", "inner", "deep"]).unwrap(),
//             &branch!(r#"{"foo":"bar"}"#)
//         );

//         assert_eq!(
//             store.read(read!["nested", "inner", "deep", "foo"]).unwrap(),
//             &leaf!("bar")
//         );

//         assert_eq!(
//             store.read(read!["nested", "sibling"]).unwrap(),
//             &leaf!("inner_sibling")
//         );

//         assert_eq!(
//             store.read(read!["binary"]).unwrap(),
//             &leaf![245, 107, 95, 100]
//         );

//         assert_eq!(
//             store.read(read!["sibling"]).unwrap(),
//             &leaf!("outer_sibling")
//         );
//     }

//     #[test]
//     fn read_not_found() {
//         let store = store!(MAP);

//         assert!(store.read(read!["bla"]).is_err());
//         assert!(store.read(read!["binary", "245"]).is_err());
//         assert!(store.read(read!["nested", "bla"]).is_err());
//         assert!(store.read(read!["nested", "bla", "foo"]).is_err());
//         assert!(store.read(read!["nested", "inner", "bla"]).is_err());
//         assert!(store.read(read!["nested", "inner", "bla", "deep"]).is_err());
//         assert!(store.read(read!["nested", "inner", "deep", "bla"]).is_err());
//         assert!(store
//             .read(read!["nested", "inner", "deep", "foo", "bla"])
//             .is_err());
//         assert!(store.read(read![""]).is_err());
//     }

//     #[test]
//     fn update() {
//         let mut store = store!(MAP);

//         // update top level
//         store.update(update!(["binary"], "new")).unwrap();
//         assert_eq!(
//             store,
//             store!(
//                 r#"{
//                      "binary": "new",
//                      "nested": {
//                        "inner": {
//                          "deep": {
//                            "foo": "bar"
//                          }
//                        },
//                        "sibling": "inner_sibling"
//                      },
//                      "sibling": "outer_sibling"
//                    }"#
//             )
//         );

//         // update deep
//         store
//             .update(update!(["nested", "inner", "deep", "foo"], "new"))
//             .unwrap();
//         assert_eq!(
//             store,
//             store!(
//                 r#"{
//                      "binary": "new",
//                      "nested": {
//                        "inner": {
//                          "deep": {
//                            "foo": "new"
//                          }
//                        },
//                        "sibling": "inner_sibling"
//                      },
//                      "sibling": "outer_sibling"
//                    }"#
//             )
//         );

//         // update root of deep tree
//         store.update(update!(["nested"], "new")).unwrap();
//         assert_eq!(
//             store,
//             store!(
//                 r#"{
//                      "binary": "new",
//                      "nested": "new",
//                      "sibling": "outer_sibling"
//                    }"#
//             )
//         );
//     }

//     #[test]
//     fn update_empty_just_deletes() {
//         macro_rules! update_empty {
//             ($($path:literal),*) => {{
//                 let mut updated = store!(MAP);
//                 let mut deleted = store!(MAP);
//                 updated.update(update!([$($path),*])).unwrap();
//                 deleted.delete(delete!($($path),*)).unwrap();
//                 assert_eq!(updated, deleted);
//             }};
//         }

//         update_empty!["binary"];
//         update_empty!["sibling"];
//         update_empty!["nested"];
//         update_empty!["nested", "sibling"];
//         update_empty!["nested", "inner"];
//         update_empty!["nested", "inner", "deep"];
//         update_empty!["nested", "inner", "deep", "foo"];
//     }

//     #[test]
//     fn update_not_found() {
//         let mut store = store!(MAP);

//         assert!(store.update(update!(["bla"], "")).is_err());
//         assert!(store.update(update!(["binary", "245"], "")).is_err());
//         assert!(store.update(update!(["nested", "bla"], "")).is_err());
//         assert!(store.update(update!(["nested", "bla", "foo"], "")).is_err());
//         assert!(store
//             .update(update!(["nested", "inner", "bla"], ""))
//             .is_err());
//         assert!(store
//             .update(update!(["nested", "inner", "bla", "deep"], ""))
//             .is_err());
//         assert!(store
//             .update(update!(["nested", "inner", "deep", "bla"], ""))
//             .is_err());
//         assert!(store
//             .update(update!(["nested", "inner", "deep", "foo", "bla"], ""))
//             .is_err());
//         assert!(store.update(update!([""], "")).is_err());
//     }

//     #[test]
//     fn delete() {
//         macro_rules! test_delete {
//             ([$($path:literal),*], $expected:literal) => {{
//                 let mut store = store!(MAP);
//                 store.delete(delete!($($path),*)).unwrap();
//                 assert_eq!(store, store!($expected));
//             }};

//         }

//         test_delete!(
//             ["binary"],
//             r#"{
//                  "nested": {
//                    "inner": {
//                      "deep": {
//                        "foo": "bar"
//                      }
//                    },
//                    "sibling": "inner_sibling"
//                  },
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         test_delete!(
//             ["sibling"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "nested": {
//                    "inner": {
//                      "deep": {
//                        "foo": "bar"
//                      }
//                    },
//                    "sibling": "inner_sibling"
//                  }
//                }"#
//         );

//         test_delete!(
//             ["nested"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         test_delete!(
//             ["nested", "sibling"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "nested": {
//                    "inner": {
//                      "deep": {
//                        "foo": "bar"
//                      }
//                    }
//                  },
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         test_delete!(
//             ["nested", "inner"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "nested": {
//                    "sibling": "inner_sibling"
//                  },
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         test_delete!(
//             ["nested", "inner", "deep"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "nested": {
//                    "sibling": "inner_sibling"
//                  },
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         test_delete!(
//             ["nested", "inner", "deep", "foo"],
//             r#"{
//                  "binary": [ 245, 107, 95, 100 ],
//                  "nested": {
//                    "sibling": "inner_sibling"
//                  },
//                  "sibling": "outer_sibling"
//                }"#
//         );

//         let mut store = store!(r#"{"one":{"two":{"three":"value"}}}"#);
//         assert!(store.delete(delete!["one", "two", "three"]).is_ok());
//         assert_eq!(store, store!("{}"));
//     }

//     #[test]
//     fn delete_not_found() {
//         let mut store = store!(MAP);

//         assert!(store.delete(delete!["bla"]).is_err());
//         assert!(store.delete(delete!["binary", "245"]).is_err());
//         assert!(store.delete(delete!["nested", "bla"]).is_err());
//         assert!(store.delete(delete!["nested", "bla", "foo"]).is_err());
//         assert!(store.delete(delete!["nested", "inner", "bla"]).is_err());
//         assert!(store
//             .delete(delete!["nested", "inner", "bla", "deep"])
//             .is_err());
//         assert!(store
//             .delete(delete!["nested", "inner", "deep", "bla"])
//             .is_err());
//         assert!(store
//             .delete(delete!["nested", "inner", "deep", "foo", "bla"])
//             .is_err());
//         assert!(store.delete(delete![""]).is_err());
//     }
// }
