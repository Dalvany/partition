mod song;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub(crate) use song::Song;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Library {
    library: PathBuf,
    temporary: PathBuf,
}

impl Library {
    pub fn library_path(&self) -> &PathBuf {
        &self.library
    }

    pub fn temporary_path(&self) -> &PathBuf {
        &self.temporary
    }

    pub fn create_folder(&self) -> Result<()> {
        std::fs::create_dir_all(&self.library)
            .with_context(|| format!("Can't create {}", self.library.display()))?;
        std::fs::create_dir_all(&self.temporary)
            .with_context(|| format!("Can't create {}", self.temporary.display()))
    }
}

impl From<crate::config::Library> for Library {
    fn from(value: crate::config::Library) -> Self {
        Self {
            library: value.path(),
            temporary: value.tmp(),
        }
    }
}
