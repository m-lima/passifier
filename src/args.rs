use super::store;

#[derive(clap::Clap, Debug)]
#[clap(name = "Passify", version)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,

    /// Load secret store from INPUT
    #[clap(name = "INPUT", parse(try_from_str = parse_input))]
    pub store: Option<Source>,

    /// Save the store to OUTPUT
    #[clap(short, long, name = "OUTPUT", parse(try_from_str = parse_output))]
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

#[derive(Debug, Eq, PartialEq)]
pub enum Source {
    File(std::path::PathBuf),
    Directory(std::path::PathBuf),
    S3(String),
}

fn parse_input(string: &str) -> anyhow::Result<Source> {
    use std::str::FromStr;

    let trimmed = string.trim();
    if let Some(path) = trimmed.strip_prefix("s3://") {
        if path.is_empty() {
            Err(anyhow::anyhow!("Invalid S3 path"))
        } else {
            Ok(Source::S3(String::from(path)))
        }
    } else {
        let path = std::path::PathBuf::from_str(string)?;
        if !path.exists() {
            Err(anyhow::anyhow!("file does not exist"))
        } else if path.is_dir() {
            Ok(Source::Directory(std::path::PathBuf::from_str(string)?))
        } else {
            Ok(Source::File(std::path::PathBuf::from_str(string)?))
        }
    }
}

fn parse_output(string: &str) -> anyhow::Result<Source> {
    use std::str::FromStr;

    let trimmed = string.trim();
    if let Some(path) = trimmed.strip_prefix("s3://") {
        if path.is_empty() {
            Err(anyhow::anyhow!("Invalid S3 path"))
        } else {
            Ok(Source::S3(String::from(path)))
        }
    } else {
        let path = std::path::PathBuf::from_str(string)?;
        if path.is_dir() || string.ends_with('/') || string.ends_with('\\') {
            Ok(Source::Directory(std::path::PathBuf::from_str(string)?))
        } else {
            Ok(Source::File(std::path::PathBuf::from_str(string)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::store::{Entry, NestedMap, Node};

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn root() -> std::path::PathBuf {
            std::env::temp_dir().join("passifier-test")
        }

        fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
            let path = Self::root().join(path.as_ref());
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl std::ops::Deref for TempDir {
        type Target = std::path::PathBuf;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::Drop for TempDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(Self::root()).unwrap();
        }
    }

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

    #[test]
    fn parse_input() {
        assert_eq!(
            super::parse_input("s3://foo/bar").unwrap(),
            super::Source::S3(String::from("foo/bar"))
        );

        assert!(super::parse_input("s3://").is_err());

        let temp_dir = TempDir::new("parse_input");
        let temp_file = temp_dir.join("foo");
        std::fs::File::create(&temp_file).unwrap();

        assert_eq!(
            super::parse_input(temp_dir.to_str().unwrap()).unwrap(),
            super::Source::Directory(std::path::PathBuf::from(temp_dir.to_str().unwrap()))
        );

        assert_eq!(
            super::parse_input(temp_file.to_str().unwrap()).unwrap(),
            super::Source::File(temp_file)
        );

        assert_eq!(
            super::parse_input(".").unwrap(),
            super::Source::Directory(std::path::PathBuf::from("."))
        );
    }

    #[test]
    fn parse_output() {
        assert_eq!(
            super::parse_output("s3://foo/bar").unwrap(),
            super::Source::S3(String::from("foo/bar"))
        );

        assert!(super::parse_output("s3://").is_err());

        assert_eq!(
            super::parse_output("./foo/bar").unwrap(),
            super::Source::File(std::path::PathBuf::from("./foo/bar"))
        );

        assert_eq!(
            super::parse_output("./foo/bar/").unwrap(),
            super::Source::Directory(std::path::PathBuf::from("./foo/bar/"))
        );

        assert_eq!(
            super::parse_output("c:\\foo\\bar\\").unwrap(),
            super::Source::Directory(std::path::PathBuf::from("c:\\foo\\bar\\"))
        );
    }
}
