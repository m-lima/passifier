use super::NestedMap;
use super::Node;

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
            <std::collections::HashMap<K, Node<K, V, S>, S> as serde::Deserialize>::deserialize(
                deserializer,
            )?,
        ))
    }
}

impl<K, V, S> serde::Serialize for Node<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Serialize,
    V: serde::Serialize,
    S: std::hash::BuildHasher,
{
    fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
    where
        SE: serde::Serializer,
    {
        #[cfg(feature = "flatten")]
        match *self {
            Self::Leaf(ref leaf) => leaf.serialize(serializer),
            Self::Branch(ref branch) => branch.serialize(serializer),
        }

        #[cfg(not(feature = "flatten"))]
        match *self {
            Self::Leaf(ref leaf) => serializer.serialize_newtype_variant("Node", 0, "Leaf", leaf),
            Self::Branch(ref branch) => {
                serializer.serialize_newtype_variant("Node", 1, "Branch", branch)
            }
        }
    }
}

impl<'d, K, V, S> serde::Deserialize<'d> for Node<K, V, S>
where
    K: Eq + std::hash::Hash + serde::Deserialize<'d>,
    V: serde::Deserialize<'d>,
    S: Default + std::hash::BuildHasher,
{
    // TODO: Update once serde::de::Content is made public
    #[cfg(feature = "flatten")]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'d>,
    {
        // struct NestedVisitor<'d, K, V, S>(
        //     std::marker::PhantomData<K>,
        //     std::marker::PhantomData<V>,
        //     std::marker::PhantomData<S>,
        //     std::marker::PhantomData<&'d ()>,
        // )
        // where
        //     K: Eq + std::hash::Hash + serde::Deserialize<'d>,
        //     V: serde::Deserialize<'d>,
        //     S: Default + std::hash::BuildHasher;

        // impl<'d, K, V, S> serde::de::Visitor<'d> for NestedVisitor<'d, K, V, S>
        // where
        //     K: Eq + std::hash::Hash + serde::Deserialize<'d>,
        //     V: serde::Deserialize<'d>,
        //     S: Default + std::hash::BuildHasher,
        // {
        //     type Value = NestedMap<K, V, S>;

        //     fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         fmt.write_str("`Node::Branch`")
        //     }

        //     fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        //     where
        //         A: serde::de::MapAccess<'d>,
        //     {
        //         println!("Serde::visit_map");
        //         <NestedMap<K, V, S> as serde::Deserialize<'d>>::deserialize(
        //             serde::de::value::MapAccessDeserializer::new(map),
        //         )
        //     }
        // }

        // struct NodeVisitor<'d, V>(
        //     std::marker::PhantomData<V>,
        //     std::marker::PhantomData<&'d ()>,
        // )
        // where
        //     V: serde::Deserialize<'d>;

        // impl<'d, V> serde::de::Visitor<'d> for NodeVisitor<'d, V>
        // where
        //     V: serde::Deserialize<'d>,
        // {
        //     type Value = V;

        //     fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         fmt.write_str("`Node::Leaf`")
        //     }

        //     fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: serde::de::Deserializer<'d>,
        //     {
        //         println!("Serde::visit_newtype_struct");
        //         <V as serde::Deserialize<'d>>::deserialize(deserializer)
        //     }
        // }

        // deserializer
        //     .deserialize_map(NestedVisitor(
        //         std::marker::PhantomData::<K>,
        //         std::marker::PhantomData::<V>,
        //         std::marker::PhantomData::<S>,
        //         std::marker::PhantomData::<&'d ()>,
        //     ))
        //     .map(Node::Branch)
        //     .or_else(|_| {
        //         deserializer
        //             .deserialize_newtype_struct(
        //                 "Leaf",
        //                 NodeVisitor(
        //                     std::marker::PhantomData::<V>,
        //                     std::marker::PhantomData::<&'d ()>,
        //                 ),
        //             )
        //             .map(Node::Leaf)
        //     })

        let content =
            match <serde::__private::de::Content<'d> as serde::Deserialize<'d>>::deserialize(
                deserializer,
            ) {
                Ok(val) => val,
                Err(err) => return Err(err),
            };

        if let Ok(leaf) =
            V::deserialize(serde::__private::de::ContentRefDeserializer::<DE::Error>::new(&content))
                .map(Self::Leaf)
        {
            return Ok(leaf);
        }

        if let Ok(branch) = <NestedMap<K, V, S> as serde::Deserialize<'d>>::deserialize(
            serde::__private::de::ContentRefDeserializer::<DE::Error>::new(&content),
        )
        .map(Self::Branch)
        {
            return Ok(branch);
        }

        Err(serde::de::Error::custom(
            "data did not match any variant of untagged enum Node",
        ))
    }

    #[cfg(not(feature = "flatten"))]
    #[allow(clippy::too_many_lines)]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'d>,
    {
        enum NodeType {
            Leaf,
            Branch,
        }

        struct NodeTypeVisitor;
        const VARIANTS: [&str; 2] = ["Leaf", "Branch"];

        impl<'d> serde::de::Visitor<'d> for NodeTypeVisitor {
            type Value = NodeType;

            fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.write_str("`Leaf` or `Branch`")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(NodeType::Leaf),
                    1 => Ok(NodeType::Branch),
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 2",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "Leaf" => Ok(NodeType::Leaf),
                    "Branch" => Ok(NodeType::Branch),
                    _ => Err(serde::de::Error::unknown_variant(value, &VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"Leaf" => Ok(NodeType::Leaf),
                    b"Branch" => Ok(NodeType::Branch),
                    _ => {
                        let value_str = String::from_utf8_lossy(value);
                        Err(serde::de::Error::unknown_variant(&value_str, &VARIANTS))
                    }
                }
            }
        }

        impl<'d> serde::Deserialize<'d> for NodeType {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'d>,
            {
                deserializer.deserialize_identifier(NodeTypeVisitor)
            }
        }

        struct NodeVisitor<'d, K, V, S>(
            std::marker::PhantomData<K>,
            std::marker::PhantomData<V>,
            std::marker::PhantomData<S>,
            std::marker::PhantomData<&'d ()>,
        )
        where
            K: Eq + std::hash::Hash + serde::Deserialize<'d>,
            V: serde::Deserialize<'d>,
            S: Default + std::hash::BuildHasher;

        impl<'d, K, V, S> serde::de::Visitor<'d> for NodeVisitor<'d, K, V, S>
        where
            K: Eq + std::hash::Hash + serde::Deserialize<'d>,
            V: serde::Deserialize<'d>,
            S: Default + std::hash::BuildHasher,
        {
            type Value = Node<K, V, S>;
            fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.write_str("enum Node")
            }

            fn visit_enum<A>(self, value: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'d>,
            {
                match value.variant()? {
                    (NodeType::Leaf, leaf) => {
                        serde::de::VariantAccess::newtype_variant::<V>(leaf).map(Node::Leaf)
                    }
                    (NodeType::Branch, branch) => {
                        serde::de::VariantAccess::newtype_variant::<NestedMap<K, V, S>>(branch)
                            .map(Node::Branch)
                    }
                }
            }
        }

        deserializer.deserialize_enum(
            "Node",
            &VARIANTS,
            NodeVisitor(
                std::marker::PhantomData::<K>,
                std::marker::PhantomData::<V>,
                std::marker::PhantomData::<S>,
                std::marker::PhantomData::<&'d ()>,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::NestedMap;

    #[test]
    fn serde_typed_json() {
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
    fn serde_flat_json() {
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

    #[test]
    fn serde_typed_rmp() {
        use serde::Serialize;

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

        let mut binary = Vec::new();
        map.serialize(&mut rmp_serde::Serializer::new(&mut binary))
            .unwrap();
        let recovered: NestedMap<String, Value> = rmp_serde::from_read_ref(&binary).unwrap();
        assert_eq!(map, recovered);
    }

    #[cfg(feature = "flatten")]
    #[test]
    fn serde_flat_rmp() {
        use serde::Serialize;

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

        let mut binary = Vec::new();
        map.serialize(&mut rmp_serde::Serializer::new(&mut binary))
            .unwrap();
        let recovered: NestedMap<String, Value> = rmp_serde::from_read_ref(&binary).unwrap();
        assert_eq!(map, recovered);
    }
}