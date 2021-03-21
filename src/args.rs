use super::store;

#[derive(clap::Clap, Debug)]
#[clap(name = "Passify", version)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,

    /// Load secret store from INPUT
    #[clap(name = "INPUT")]
    pub store: Option<Source>,

    /// Save the store to OUTPUT
    #[clap(short, long, name = "OUTPUT")]
    pub save: Option<Source>,
}

#[derive(clap::Clap, Debug)]
pub enum Action {
    /// Print the store in JSON format
    Print(Print),

    /// Create new secret
    Create(Write),

    /// Read an existing secret
    Read(Read),

    /// Update an existing secret
    Update(Write),

    /// Delete an existing secret
    Delete(Delete),
}

#[derive(clap::Clap, Debug)]
pub struct Read {
    /// Path to the secret
    pub path: Path,

    /// Pretty print
    #[clap(short, long)]
    pub pretty: bool,
}

#[derive(clap::Clap, Debug)]
pub struct Print {
    /// Pretty print
    #[clap(short, long)]
    pub pretty: bool,
}

#[derive(clap::Clap, Debug)]
pub struct Delete {
    /// Path to the secret
    pub path: Path,
}

#[derive(clap::Clap, Debug)]
pub struct Write {
    /// Path to the secret
    pub path: Path,

    /// Value for the secret
    #[clap( parse(try_from_str = parse_entry))]
    pub(super) secret: store::Node,
}

#[derive(Debug)]
pub struct Path(pub Vec<String>);

impl Path {
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &String> {
        self.0.iter()
    }
}

fn parse_entry(string: &str) -> anyhow::Result<store::Node> {
    fn remove_empties(node: &mut store::Node) -> bool {
        if let nested_map::Node::Branch(branch) = node {
            let secrets = branch.keys().map(String::from).collect::<Vec<_>>();
            for secret in secrets {
                if remove_empties(branch.get_mut(&secret).unwrap()) {
                    branch.remove(&secret).unwrap();
                }
            }
            branch.keys().next().is_none()
        } else {
            false
        }
    }

    if string
        .split_whitespace()
        .next()
        .map(|s| s.starts_with('{') || s.starts_with('[') || s.starts_with('"'))
        .ok_or_else(|| anyhow::anyhow!("Empty secret"))?
    {
        let mut entry = serde_json::from_str(string)?;
        remove_empties(&mut entry);
        Ok(entry)
    } else {
        Ok(store::Node::Leaf(store::Entry::String(String::from(
            string,
        ))))
    }
}

impl std::str::FromStr for Path {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let entries = string
            .split('.')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(String::from(trimmed))
                }
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            Err(anyhow::anyhow!("Empty path"))
        } else {
            Ok(Self(entries))
        }
    }
}

#[derive(Debug)]
pub enum Source {
    File(std::path::PathBuf),
    S3(String),
}

impl std::str::FromStr for Source {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let trimmed = string.trim();
        if let Some(path) = trimmed.strip_prefix("s3://") {
            if path.is_empty() {
                Err(anyhow::anyhow!("Invalid S3 path"))
            } else {
                Ok(Self::S3(String::from(path)))
            }
        } else {
            Ok(Self::File(std::path::PathBuf::from_str(string)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::store::{Entry, NestedMap, Node};

    #[test]
    fn parse_entry() {
        assert_eq!(
            super::parse_entry("foobar").unwrap(),
            Node::Leaf(Entry::String(String::from("foobar")))
        );

        assert_eq!(
            super::parse_entry("\"foobar\"").unwrap(),
            Node::Leaf(Entry::String(String::from("foobar")))
        );

        assert_eq!(
            super::parse_entry(" \n\t\r \"foobar\"").unwrap(),
            Node::Leaf(Entry::String(String::from("foobar")))
        );

        assert_eq!(
            super::parse_entry("[1, 2]").unwrap(),
            Node::Leaf(Entry::Binary(vec![1, 2]))
        );

        assert_eq!(
            super::parse_entry("\n{\n}").unwrap(),
            Node::Branch(NestedMap::new())
        );

        assert_eq!(
            super::parse_entry(r#"{"nested":{"inner":{}}}"#).unwrap(),
            Node::Branch(NestedMap::new())
        );

        let mut store = NestedMap::new();
        store.insert_into(["nested", "inner", "key"], Entry::from("value"));
        assert_eq!(
            super::parse_entry(r#"{"nested":{"inner":{"key":"value", "empty":{}}}}"#).unwrap(),
            Node::Branch(store)
        );
    }
}
