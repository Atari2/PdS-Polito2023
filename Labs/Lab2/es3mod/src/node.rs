use crate::file::File;
use crate::dir::Dir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl PartialEq<Path> for Node {
    fn eq(&self, other: &Path) -> bool {
        match self {
            Self::File(f) => f.name() == other,
            Self::Dir(d) => d.name() == other,
        }
    }
}

impl PartialEq<PathBuf> for Node {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other.as_path()
    }
}