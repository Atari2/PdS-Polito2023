use crate::file::File;
use crate::dir::Dir;
use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

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

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(file) => std::fmt::Display::fmt(&file, f),
            Self::Dir(dir) => std::fmt::Display::fmt(&dir, f),
        }
    }
}
