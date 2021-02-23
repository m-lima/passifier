#[derive(clap::Clap, Debug)]
#[clap(name = "Passify", version)]
pub struct Args {
    #[clap(subcommand)]
    action: Action,

    /// Load secret store from INPUT
    #[clap(name = "INPUT")]
    store: Option<Source>,

    /// Save the store to OUTPUT
    #[clap(short, long, name = "OUTPUT")]
    save: Option<Source>,
}

impl Args {
    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn store(&self) -> Option<&Source> {
        self.store.as_ref()
    }

    pub fn save(&self) -> Option<&Source> {
        self.save.as_ref()
    }
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
    pretty: bool,
}

impl Print {
    pub fn pretty(&self) -> bool {
        self.pretty
    }
}

#[derive(clap::Clap, Debug)]
pub struct Path {
    /// Path to the secret
    path: Entries,
}

impl Path {
    pub fn path(&self) -> &[String] {
        &self.path.0
    }
}

#[derive(clap::Clap, Debug)]
pub struct Entry {
    /// Path to the secret
    path: Entries,

    /// Value for the secret
    secret: super::json::Entry,
}

impl Entry {
    pub fn path(&self) -> &[String] {
        &self.path.0
    }

    pub fn secret(&self) -> &super::json::Entry {
        &self.secret
    }
}

#[derive(Debug)]
struct Entries(Vec<String>);

impl AsRef<[String]> for Entries {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}

impl std::str::FromStr for Entries {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let entries = string
            .split('.')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(String::from(s))
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
