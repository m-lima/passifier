#![deny(warnings, rust_2018_idioms,
        // missing_docs,
        clippy::pedantic)]

//! Handles nested hash maps

#[cfg(feature = "serde")]
mod serde;

/// A nested hash map
#[derive(Clone)]
pub struct NestedMap<K, V, S = std::collections::hash_map::RandomState>(
    std::collections::HashMap<K, Node<K, V, S>, S>,
);

impl<K, V> NestedMap<K, V, std::collections::hash_map::RandomState> {
    /// Creates a new empty nested map
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(std::collections::HashMap::new())
    }
}

impl<K, V, S> NestedMap<K, V, S>
where
    K: Eq + std::hash::Hash,
    S: std::hash::BuildHasher,
{
    // [ ] entry
    // [ ] get_key_value
    // [ ] insert
    // [ ] retain
    // [ ] into_keys
    // [x] contains_key
    // [x] get
    // [x] get_mut
    // [x] remove
    // [x] remove_entry

    // [ ] get_last_path

    #[inline]
    pub fn contains_path<'a, P, Q: ?Sized>(&self, path: P) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        P: AsRef<[&'a Q]>,
    {
        self.contains_path_iter(path.as_ref().iter().map(Clone::clone))
    }

    pub fn contains_path_iter<'a, I, Q: ?Sized>(&self, iter: I) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        I: Iterator<Item = &'a Q>,
    {
        self.get_from_iter(iter).is_some()
    }

    #[inline]
    pub fn get_from<'a, P, Q: ?Sized>(&self, path: P) -> Option<&Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        P: AsRef<[&'a Q]>,
    {
        self.get_from_iter(path.as_ref().iter().map(Clone::clone))
    }

    pub fn get_from_iter<'a, I, Q: ?Sized>(&self, mut iter: I) -> Option<&Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        I: Iterator<Item = &'a Q>,
    {
        let mut root = self.get(iter.next()?)?;
        for key in iter {
            root = root.get(key)?;
        }
        Some(root)
    }

    #[inline]
    pub fn get_mut_from<'a, P, Q: ?Sized>(&mut self, path: P) -> Option<&mut Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        P: AsRef<[&'a Q]>,
    {
        self.get_mut_from_iter(path.as_ref().iter().map(Clone::clone))
    }

    pub fn get_mut_from_iter<'a, I, Q: ?Sized>(&mut self, mut iter: I) -> Option<&mut Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        I: Iterator<Item = &'a Q>,
    {
        let mut root = self.get_mut(iter.next()?)?;
        for key in iter {
            root = root.get_mut(key)?;
        }
        Some(root)
    }

    #[inline]
    pub fn remove_from<'a, P, Q: ?Sized>(&mut self, path: P) -> Option<Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        P: AsRef<[&'a Q]>,
    {
        self.remove_from_iter(path.as_ref().iter().map(Clone::clone))
    }

    pub fn remove_from_iter<'a, I, Q: ?Sized>(&mut self, iter: I) -> Option<Node<K, V, S>>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        I: Iterator<Item = &'a Q>,
    {
        let mut peekable = iter.peekable();
        let mut root = self;

        let last = loop {
            let key = peekable.next()?;
            if peekable.peek().is_none() {
                break key;
            }
            if let Node::Branch(branch) = root.get_mut(key)? {
                root = branch;
            } else {
                return None;
            }
        };
        root.remove(last)
    }

    #[inline]
    pub fn remove_entry_from<'a, P, Q: ?Sized>(&mut self, path: P) -> Option<(K, Node<K, V, S>)>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        P: AsRef<[&'a Q]>,
    {
        self.remove_entry_from_iter(path.as_ref().iter().map(Clone::clone))
    }

    pub fn remove_entry_from_iter<'a, I, Q: ?Sized>(
        &mut self,
        iter: I,
    ) -> Option<(K, Node<K, V, S>)>
    where
        K: std::borrow::Borrow<Q>,
        Q: 'a + Eq + std::hash::Hash,
        I: Iterator<Item = &'a Q>,
    {
        let mut peekable = iter.peekable();
        let mut root = self;

        let last = loop {
            let key = peekable.next()?;
            if peekable.peek().is_none() {
                break key;
            }
            if let Node::Branch(branch) = root.get_mut(key)? {
                root = branch;
            } else {
                return None;
            }
        };
        root.remove_entry(last)
    }

    // #[inline]
    // pub fn insert_into<'a, P, Q: ?Sized>(
    //     &mut self,
    //     path: P,
    //     node: Node<K, V, S>,
    // ) -> Option<Node<K, V, S>>
    // where
    //     K: std::borrow::Borrow<Q> + Clone,
    //     Q: 'a + Eq + std::hash::Hash,
    //     P: AsRef<[&'a Q]>,
    // {
    //     self.insert_into_iter(path.as_ref().iter().map(Clone::clone), node)
    // }

    // pub fn insert_into_iter<'a, I, Q: ?Sized>(
    //     &mut self,
    //     iter: I,
    //     node: Node<K, V, S>,
    // ) -> Option<Node<K, V, S>>
    // where
    //     K: std::borrow::Borrow<Q> + Clone,
    //     Q: 'a + Eq + std::hash::Hash,
    //     I: Iterator<Item = &'a Q>,
    // {
    //     let mut peekable = iter.peekable();
    //     let mut root = self;
    //     let mut replaced = None;
    //     let last = loop {
    //         let key = peekable.next()?;
    //         if peekable.peek().is_none() {
    //             break key;
    //         }
    //         if let Some(node) = root.get_mut(key) {
    //         } else {
    //             root.insert(key, Node::Branch(NestedMap::<K, V, S>::new()));
    //         }
    //         if let Node::Branch(branch) = root.get_mut(key)? {
    //             root = branch;
    //         } else {
    //             return None;
    //         }
    //     };
    //     root.insert(key, value)
    // }
}

impl<K, V, S> std::ops::Deref for NestedMap<K, V, S> {
    type Target = std::collections::HashMap<K, Node<K, V, S>, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> std::ops::DerefMut for NestedMap<K, V, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V, S> PartialEq for NestedMap<K, V, S>
where
    K: Eq + std::hash::Hash,
    V: PartialEq,
    S: std::hash::BuildHasher,
{
    #[inline]
    fn eq(&self, other: &NestedMap<K, V, S>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> std::fmt::Debug for NestedMap<K, V, S>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl<K, V, S> Default for NestedMap<K, V, S>
where
    S: Default,
{
    #[inline]
    fn default() -> NestedMap<K, V, S> {
        Self(std::collections::HashMap::default())
    }
}

impl<K, Q: ?Sized, V, S> std::ops::Index<&Q> for NestedMap<K, V, S>
where
    K: Eq + std::hash::Hash + std::borrow::Borrow<Q>,
    Q: Eq + std::hash::Hash,
    S: std::hash::BuildHasher,
{
    type Output = Node<K, V, S>;

    #[inline]
    fn index(&self, key: &Q) -> &Self::Output {
        &self.0[key]
    }
}

/// Node can be a leaf or a sub map
#[derive(Clone)]
pub enum Node<K, V, S = std::collections::hash_map::RandomState> {
    /// A node containing `V`
    Leaf(V),
    /// A sub map
    Branch(NestedMap<K, V, S>),
}

impl<K, V, S> Node<K, V, S>
where
    K: Eq + std::hash::Hash,
    S: std::hash::BuildHasher,
{
    /// Gets a reference to the value form `key` from this node if it is a branch
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Self>
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        if let Self::Branch(branch) = self {
            branch.get(key)
        } else {
            None
        }
    }

    /// Gets a mutable reference to the value form `key` from this node if it is a bracnch
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Self>
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        if let Self::Branch(branch) = self {
            branch.get_mut(key)
        } else {
            None
        }
    }
}

impl<K, V, S> From<V> for Node<K, V, S> {
    fn from(value: V) -> Self {
        Self::Leaf(value)
    }
}

impl<K, V, S> From<NestedMap<K, V, S>> for Node<K, V, S> {
    fn from(map: NestedMap<K, V, S>) -> Self {
        Self::Branch(map)
    }
}

impl<K, V, S> PartialEq for Node<K, V, S>
where
    K: Eq + std::hash::Hash,
    V: PartialEq,
    S: std::hash::BuildHasher,
{
    fn eq(&self, other: &Node<K, V, S>) -> bool {
        match self {
            Self::Leaf(leaf) => {
                if let Self::Leaf(other_leaf) = other {
                    leaf.eq(other_leaf)
                } else {
                    false
                }
            }
            Self::Branch(branch) => {
                if let Self::Branch(other_branch) = other {
                    branch.eq(other_branch)
                } else {
                    false
                }
            }
        }
    }
}

impl<K, V, S> Eq for Node<K, V, S>
where
    K: Eq + std::hash::Hash,
    V: Eq,
    S: std::hash::BuildHasher,
{
}

impl<K, V, S> std::fmt::Debug for Node<K, V, S>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(leaf) => leaf.fmt(fmt),
            Self::Branch(branch) => branch.fmt(fmt),
        }
    }
}

impl<K, Q: ?Sized, V, S> std::ops::Index<&Q> for Node<K, V, S>
where
    K: Eq + std::hash::Hash + std::borrow::Borrow<Q>,
    Q: Eq + std::hash::Hash,
    S: std::hash::BuildHasher,
{
    type Output = Node<K, V, S>;

    #[inline]
    fn index(&self, key: &Q) -> &Self::Output {
        if let Self::Branch(branch) = self {
            &branch[key]
        } else {
            panic!("no entry found for key")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NestedMap,
        Node::{Branch, Leaf},
    };

    macro_rules! own {
        ($string:literal) => {
            String::from($string)
        };
    }

    #[test]
    fn node_from_leaf() {
        let mut map = NestedMap::new();
        assert_eq!(map.insert("key", "value".into()), None);
        assert_eq!(map.get("key"), Some(&Leaf("value")));
    }

    #[test]
    fn branch_from_map() {
        let mut map = NestedMap::new();
        let mut inner = NestedMap::new();
        inner.insert("inner_key", "inner_value".into());
        map.insert("key", inner.clone().into());
        assert_eq!(map.get("key"), Some(&Branch(inner)));
    }

    #[test]
    fn get_from() {
        let mut map = NestedMap::new();
        map.insert(own!("key"), "value".into());

        let mut inner = NestedMap::new();
        inner.insert(own!("inner_key"), "inner_value".into());

        map.insert(own!("nested"), inner.clone().into());

        assert_eq!(map.get_from::<&[&String], String>(&[]), None);
        assert_eq!(map.get_from(["key"]), Some(&Leaf("value")));
        assert_eq!(map.get_from(["fake"]), None);
        assert_eq!(map.get_from(["nested"]), Some(&Branch(inner)));
        assert_eq!(map.get_from(["nested", "fake"]), None);
        assert_eq!(
            map.get_from(["nested", "inner_key"]),
            Some(&Leaf("inner_value"))
        );
        assert_eq!(map.get_from(["nested", "inner_key", "not_nested"]), None);
    }

    #[test]
    fn get_mut_from() {
        let mut map = NestedMap::new();
        map.insert(own!("key"), "value".into());

        let mut inner = NestedMap::new();
        inner.insert(own!("inner_key"), "inner_value".into());

        map.insert(own!("nested"), inner.clone().into());

        assert_eq!(map.get_mut_from::<&[&String], String>(&[]), None);
        assert_eq!(map.get_mut_from(["key"]), Some(&mut Leaf("value")));
        assert_eq!(map.get_mut_from(["fake"]), None);
        assert_eq!(map.get_mut_from(["nested"]), Some(&mut Branch(inner)));
        assert_eq!(map.get_mut_from(["nested", "fake"]), None);
        assert_eq!(
            map.get_mut_from(["nested", "inner_key"]),
            Some(&mut Leaf("inner_value"))
        );
        assert_eq!(
            map.get_mut_from(["nested", "inner_key", "not_nested"]),
            None
        );
    }

    #[test]
    fn remove_from() {
        let mut map = NestedMap::new();
        map.insert(own!("key"), "value".into());

        let mut inner = NestedMap::new();
        inner.insert(own!("inner_key"), "inner_value".into());

        map.insert(own!("nested"), inner.clone().into());
        map.insert(own!("nested2"), inner.clone().into());

        assert_eq!(map.remove_from::<&[&String], String>(&[]), None);
        assert_eq!(map.remove_from(["nested", "fake"]), None);
        assert_eq!(map.remove_from(["nested", "inner_key", "too_deep"]), None);
        assert_eq!(
            map.remove_from(["nested", "inner_key"]),
            Some(Leaf("inner_value"))
        );
        assert_eq!(map.remove_from(["nested", "inner_key"]), None);
        assert_eq!(map.remove_from(["nested2"]), Some(Branch(inner.clone())));
        assert_eq!(map.remove_from(["key"]), Some(Leaf("value")));
    }

    #[test]
    fn remove_entry_from() {
        let mut map = NestedMap::new();
        map.insert(own!("key"), "value".into());

        let mut inner = NestedMap::new();
        inner.insert(own!("inner_key"), "inner_value".into());

        map.insert(own!("nested"), inner.clone().into());
        map.insert(own!("nested2"), inner.clone().into());

        assert_eq!(map.remove_entry_from::<&[&String], String>(&[]), None);
        assert_eq!(map.remove_entry_from(["nested", "fake"]), None);
        assert_eq!(
            map.remove_entry_from(["nested", "inner_key", "too_deep"]),
            None
        );
        assert_eq!(
            map.remove_entry_from(["nested", "inner_key"]),
            Some((own!("inner_key"), Leaf("inner_value")))
        );
        assert_eq!(map.remove_entry_from(["nested", "inner_key"]), None);
        assert_eq!(
            map.remove_entry_from(["nested2"]),
            Some((own!("nested2"), Branch(inner.clone())))
        );
        assert_eq!(
            map.remove_entry_from(["key"]),
            Some((own!("key"), Leaf("value")))
        );
    }

    // TODO: Move to docs
    // TODO: Do the same for all signatures
    #[test]
    fn get_from_calling_signature() {
        let mut map = NestedMap::new();
        map.insert(own!("key"), "value".into());

        // Empty
        assert_eq!(map.get_from::<&[&String], String>(&[]), None);
        assert_eq!(map.get_from::<&[&str], str>(&[]), None);

        // Slice
        assert_eq!(map.get_from([&own!("key")]), Some(&Leaf("value")));
        assert_eq!(map.get_from(["key"]), Some(&Leaf("value")));

        // Slice reference
        assert_eq!(map.get_from(&[&own!("key")]), Some(&Leaf("value")));
        assert_eq!(map.get_from(&["key"]), Some(&Leaf("value")));

        // Vec
        assert_eq!(map.get_from(vec![&own!("key")]), Some(&Leaf("value")));
        assert_eq!(map.get_from(vec!["key"]), Some(&Leaf("value")));

        // Vec reference
        assert_eq!(map.get_from(&vec![&own!("key")]), Some(&Leaf("value")));
        assert_eq!(map.get_from(&vec!["key"]), Some(&Leaf("value")));

        // Iterator
        assert_eq!(
            map.get_from_iter(
                ["  key  "]
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
            ),
            Some(&Leaf("value"))
        );
    }
}
