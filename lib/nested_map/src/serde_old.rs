use super::Entry;
use super::NestedMap;

#[cfg(feature = "serde")]
impl<K, V, S> serde::Serialize for NestedMap<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Serialize,
    V: serde::Serialize,
    S: std::hash::BuildHasher,
{
    fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
    where
        SE: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'d, K, V, S> serde::Deserialize<'d> for NestedMap<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Deserialize<'d>,
    V: serde::Deserialize<'d>,
    S: Default + std::hash::BuildHasher,
{
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'d>,
    {
        Ok(Self(
            <std::collections::HashMap<K, Entry<K, V, S>, S> as serde::Deserialize>::deserialize(
                deserializer,
            )?,
        ))
    }
}

// #[cfg(feature = "serde")]
// mod serde_entry {
//     use super::Entry;

//     #[derive(serde::Serialize)]
//     #[cfg_attr(feature = "flatten", serde(untagged))]
//     pub enum Serialize<'a, K, V, S>
//     where
//         // K: Eq + std::hash::Hash,
//         // S: Default + std::hash::BuildHasher,
//         K: Eq + std::hash::Hash + serde::Serialize,
//         V: serde::Serialize,
//         S: Default + std::hash::BuildHasher,
//     {
//         Node(&'a V),
//         /// A sub map
//         Nested(&'a super::NestedMap<K, V, S>),
//     }

//     impl<'a, K, V, S> Serialize<'a, K, V, S>
//     where
//         K: Eq + std::hash::Hash + serde::Serialize,
//         V: serde::Serialize,
//         S: Default + std::hash::BuildHasher,
//     {
//         pub fn new(entry: &'a Entry<K, V, S>) -> Self {
//             match entry {
//                 Entry::Node(node) => Self::Node(node),
//                 Entry::Nested(nested) => Self::Nested(nested),
//             }
//         }
//     }

//     #[derive(serde::Deserialize)]
//     #[cfg_attr(feature = "flatten", serde(untagged))]
//     pub enum Deserialize<K, V, S>
//     where
//         K: Eq + std::hash::Hash,
//         S: Default + std::hash::BuildHasher,
//     {
//         Node(V),
//         /// A sub map
//         Nested(super::NestedMap<K, V, S>),
//     }

//     impl<'d, K, V, S> Into<Entry<K, V, S>> for Deserialize<K, V, S>
//     where
//         K: Eq + std::hash::Hash + serde::Deserialize<'d>,
//         V: serde::Deserialize<'d>,
//         S: Default + std::hash::BuildHasher,
//     {
//         fn into(self) -> Entry<K, V, S> {
//             match self {
//                 Self::Node(node) => Entry::Node(node),
//                 Self::Nested(nested) => Entry::Nested(nested),
//             }
//         }
//     }
// }

#[cfg(feature = "serde")]
impl<K, V, S> serde::Serialize for Entry<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Serialize,
    V: serde::Serialize,
    S: std::hash::BuildHasher,
{
    fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
    where
        SE: serde::Serializer,
    {
        // serde_entry::Serialize::new(self).serialize(serializer)
        // // let serialize = serde_entry::Serialize::from(self);
        // // serde::Serialize::serialize(&serialize, serializer)

        #[cfg(feature = "flatten")]
        match self {
            Self::Node(node) => node.serialize(serializer),
            Self::Nested(nested) => nested.serialize(serializer),
        }

        #[cfg(not(feature = "flatten"))]
        match self {
            Self::Node(node) => serializer.serialize_newtype_variant("Entry", 0, "Node", node),
            Self::Nested(nested) => {
                serializer.serialize_newtype_variant("Entry", 1, "Nested", nested)
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'d, K, V, S> serde::Deserialize<'d> for Entry<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Deserialize<'d>,
    V: serde::Deserialize<'d>,
    S: Default + std::hash::BuildHasher,
{
    // fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    // where
    //     DE: serde::Deserializer<'d>,
    // {
    //     <serde_entry::Deserialize<K, V, S> as serde::Deserialize>::deserialize(deserializer)
    //         .map(|d| d.into())
    // }

    // #[cfg(not(feature = "flatten"))]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'d>,
    {
        let content =
            match <serde::__private::de::Content<'d> as serde::Deserialize<'d>>::deserialize(
                deserializer,
            ) {
                Ok(val) => val,
                Err(err) => return Err(err),
            };
        if let Ok(node) =
            V::deserialize(serde::__private::de::ContentRefDeserializer::<DE::Error>::new(&content))
                .map(Self::Node)
        {
            return Ok(node);
        }
        if let Ok(nested) = <NestedMap<K, V, S> as serde::Deserialize<'d>>::deserialize(
            serde::__private::de::ContentRefDeserializer::<DE::Error>::new(&content),
        )
        .map(Self::Nested)
        {
            return Ok(nested);
        }
        Err(serde::de::Error::custom(
            "data did not match any variant of untagged enum Entry",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::NestedMap;

    #[test]
    fn serde_typed() {
        #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
        enum Value {
            String(String),
            Vec(Vec<u8>),
        }

        let mut map = NestedMap::<String, Value>::new();
        let inner = {
            let mut map = NestedMap::<String, Value>::new();
            map.insert(
                String::from("inner_string"),
                Value::String(String::from("inner_value")).into(),
            );
            map.insert(String::from("inner_vec"), Value::Vec(vec![1, 2, 3]).into());
            map
        };
        map.insert(String::from("nested"), inner.into());
        map.insert(
            String::from("outer_string"),
            Value::String(String::from("outer_value")).into(),
        );
        map.insert(String::from("outer_vec"), Value::Vec(vec![4, 5, 6]).into());

        let json = serde_json::to_string(&map).unwrap();
        let recovered: NestedMap<String, Value> = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(map, recovered);
    }

    #[cfg(feature = "flatten")]
    #[test]
    fn serde_flat() {
        #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
        #[serde(untagged)]
        enum Value {
            String(String),
            Vec(Vec<u8>),
        }

        let mut map = NestedMap::<String, Value>::new();
        let inner = {
            let mut map = NestedMap::<String, Value>::new();
            map.insert(
                String::from("inner_string"),
                Value::String(String::from("inner_value")).into(),
            );
            map.insert(String::from("inner_vec"), Value::Vec(vec![1, 2, 3]).into());
            map
        };
        map.insert(String::from("nested"), inner.into());
        map.insert(
            String::from("outer_string"),
            Value::String(String::from("outer_value")).into(),
        );
        map.insert(String::from("outer_vec"), Value::Vec(vec![4, 5, 6]).into());

        let json = serde_json::to_string(&map).unwrap();
        let recovered: NestedMap<String, Value> = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(map, recovered);
    }
}
