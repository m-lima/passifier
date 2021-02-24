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
    Create(Entry),

    /// Read an existing secret
    Read(Path),

    /// Update an existing secret
    Update(Entry),

    /// Delete an existing secret
    Delete(Path),
}

#[derive(clap::Clap, Debug)]
pub struct Print {
    /// Pretty print
    #[clap(short, long)]
    pub pretty: bool,
}

#[derive(clap::Clap, Debug)]
pub struct Path {
    /// Path to the secret
    pub path: Entries,
}

#[derive(clap::Clap, Debug)]
pub struct Entry {
    /// Path to the secret
    pub path: Entries,

    /// Value for the secret
    #[clap( parse(try_from_str = parse_entry))]
    pub secret: store::Entry,
}

#[derive(Debug)]
pub struct Entries(Vec<String>);

impl AsRef<[String]> for Entries {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}

fn parse_entry(string: &str) -> anyhow::Result<store::Entry> {
    fn remove_empties(entry: &mut store::Entry) -> bool {
        if let store::Entry::Nested(nested) = entry {
            let secrets = nested.secrets().map(String::from).collect::<Vec<_>>();
            for secret in secrets {
                if remove_empties(nested.get(&secret).unwrap()) {
                    nested.delete(&secret).unwrap();
                }
            }
            nested.secrets().next().is_none()
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
        Ok(store::Entry::String(String::from(string)))
    }
}

impl std::str::FromStr for Entries {
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
    #[test]
    fn parse_entry() {
        assert_eq!(
            super::parse_entry("foobar").unwrap(),
            store::Entry::String(String::from("foobar"))
        );

        assert_eq!(
            super::parse_entry("\"foobar\"").unwrap(),
            store::Entry::String(String::from("foobar"))
        );

        assert_eq!(
            super::parse_entry(" \n\t\r \"foobar\"").unwrap(),
            store::Entry::String(String::from("foobar"))
        );

        assert_eq!(
            super::parse_entry("[1, 2]").unwrap(),
            store::Entry::Binary(vec![1, 2])
        );

        assert_eq!(
            super::parse_entry("\n{\n}").unwrap(),
            store::Entry::Nested(store::Store::new())
        );

        assert_eq!(
            super::parse_entry(r#"{"nested":{"inner":{}}}"#).unwrap(),
            store::Entry::Nested(store::Store::new())
        );
    }
}
