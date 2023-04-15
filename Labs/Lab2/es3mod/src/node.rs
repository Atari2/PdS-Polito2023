use crate::MatchResult;
use crate::{dir::Dir, QueryType};
use crate::file::File;
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
impl<'b> Node {
    pub fn search<'a>(&'b mut self, queries: &[QueryType<'a>], mut result: MatchResult<'a>) -> MatchResult<'a>
    where
        'b: 'a,
    {
        let self_ptr = self as *const Self as *mut Self;
        let node = unsafe { &mut *self_ptr };
        if let Some(q) = queries.iter().find(|q| q.matches(self)) {
            result.queries.push(q.to_str());
            result.nodes.push(node);
        }
        match self {
            Self::File(_) => (),
            Self::Dir(dir) => {
                result = dir.search(queries, result);
            }
        }
        result
    }
}