pub mod common;
pub mod file;
pub mod dir;
pub mod node;

use std::{fmt::Display, time::Duration};
use std::time::SystemTime;
use std::path::PathBuf;
pub use common::{FileOrDirError, FileType, FsResult};
pub use dir::Dir;
pub use file::File;
pub use node::Node;

#[derive(Debug, Default)]
pub struct FileSystem {
    root: Option<Dir>,
}

impl std::fmt::Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.root {
            Some(root) => f.write_fmt(format_args!("{}", root)),
            None => f.write_str(""),
        }
    }
}


#[derive(Debug, Default)]
pub struct MatchResult<'a> {
    pub queries: Vec<&'a str>,
    pub nodes: Vec<&'a mut Node>,
}

impl Display for MatchResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str("Matched queries: ");
        for query in self.queries.iter() {
            result.push_str(query);
            result.push_str(", ");
        }
        result.pop();
        result.pop();
        result.push_str("\nFound nodes: ");
        for node in self.nodes.iter() {
            result.push_str(&format!("\n\t{:?}", node));
        }
        f.write_str(&result)
    }
}

pub enum QueryType<'a> {
    Name(&'a str, &'a str),
    Content(&'a str, &'a str),
    Larger(&'a str, usize),
    Smaller(&'a str, usize),
    Newer(&'a str, SystemTime),
    Older(&'a str, SystemTime),
}

impl<'a> QueryType<'a> {
    fn to_str(&self) -> &'a str {
        match self {
            Self::Name(og, _) => og,
            Self::Content(og, _) => og,
            Self::Larger(og, _) => og,
            Self::Smaller(og, _) => og,
            Self::Newer(og, _) => og,
            Self::Older(og, _) => og,
        }
    }
}

impl<'b> FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root: None,
        }
    }
    fn make_absolute(&self, pb: &str) -> FsResult<PathBuf> {
        let root = self.root.as_ref().ok_or(FileOrDirError::ParentDoesNotExist)?;
        let mut pb = PathBuf::from(pb);
        if !pb.is_absolute() {
            pb = PathBuf::from(&root.name()).join(pb);
        }
        Ok(pb)
    }

    fn make_absolute_no_borrow(root: &Dir, pb: &str) -> FsResult<PathBuf> {
        let mut pb = PathBuf::from(pb);
        if !pb.is_absolute() {
            pb = PathBuf::from(&root.name()).join(pb);
        }
        Ok(pb)
    }

    #[cfg(target_os = "windows")]
    fn make_root_abs(&mut self, path: &str) -> FsResult<()> {
        let mut path = PathBuf::from(path);
        if !path.is_absolute() {
            path = PathBuf::from("C:\\").join(path);
        }
        self.root = Some(Dir::empty_from_parts(path, SystemTime::now())?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn make_root_abs(&mut self, path: &str) -> FsResult<()> {
        let mut path = PathBuf::from(path);
        if !path.starts_with(std::path::MAIN_SEPARATOR_STR) {
            path = PathBuf::from(std::path::MAIN_SEPARATOR_STR).join(path);
        }
        self.root = Some(Dir::empty_from_parts(path, std::time::SystemTime::now())?);
        Ok(())
    }

    pub fn from_dir(path: &str) -> FsResult<FileSystem> {
        let mut fs = FileSystem::new();
        fs.root = Some(Dir::new(PathBuf::from(path))?);
        Ok(fs)
    }
    pub fn mk_dir(&mut self, path: &str) -> FsResult<()> {
        // special case empty fs
        match &mut self.root {
            Some(root) => {
                let pb = Self::make_absolute_no_borrow(root, path)?;
                root.mk_dir(&pb)? 
            }
            None => {
                self.make_root_abs(path)?;
            }
        }
        Ok(())
    }
    pub fn rm_dir(&mut self, path: &str) -> FsResult<()> {
        let pb = self.make_absolute(path)?;
        let root = self.root.as_mut().ok_or(FileOrDirError::ParentDoesNotExist)?;
        if *root == pb {
            if !root.children().is_empty() {
                return Err(FileOrDirError::DirectoryNotEmpty);
            }
            self.root = None;
        } else {
            root.rm_dir(&pb)?;
        }
        Ok(())
    }
    /* accordign to homework sheet signature should be &mut self, path: &str, file: File but since path is already contained in File it doesn't make sense to duplicate the information */
    pub fn new_file(&mut self, file: File) -> FsResult<()> {
        let root = self.root.as_mut().ok_or(FileOrDirError::ParentDoesNotExist)?;
        let pb = PathBuf::from(&root.name()).join(file.name());
        if *root == pb {
            return Err(FileOrDirError::AlreadyExists);
        }
        root.new_file(&pb)
    }
    pub fn rm_file(&mut self, path: &str) -> FsResult<()> {
        let pb = self.make_absolute(path)?;
        let root = self.root.as_mut().ok_or(FileOrDirError::ParentDoesNotExist)?;
        root.rm_file(&pb)
    }
    pub fn get_file(&mut self, path: &str) -> Option<&mut File> {
        let pb = match self.make_absolute(path) {
            Ok(p) => p,
            Err(_) => return None,
        };
        let root = match &mut self.root {
            Some(r) => r,
            None => return None,
        };
        root.get_file(&pb)
    }

    fn u64_to_systemtime(u: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(u)
    }

    pub fn search<'a>(&'b mut self, queries: &[&'a str]) -> MatchResult<'a>
    where
        'b: 'a,
    {
        enum InvalidQuery {
            NoQuery,
            InvalidQuery,
        }
        let queries: Vec<QueryType> = queries
            .iter()
            .map(|s| {
                let mut query = s.split(':');
                let query_type = query.next().ok_or(InvalidQuery::NoQuery)?;
                let query_value = query.next().ok_or(InvalidQuery::NoQuery)?;
                let mappedquery = match query_type {
                    "name" => QueryType::Name(s, query_value),
                    "content" => QueryType::Content(s, query_value),
                    "larger" => {
                        let size = query_value
                            .parse::<usize>()
                            .map_err(|_| InvalidQuery::InvalidQuery)?;
                        QueryType::Larger(s, size)
                    }
                    "smaller" => {
                        let size = query_value
                            .parse::<usize>()
                            .map_err(|_| InvalidQuery::InvalidQuery)?;
                        QueryType::Smaller(s, size)
                    }
                    "newer" => {
                        let time = query_value
                            .parse::<u64>()
                            .map_err(|_| InvalidQuery::InvalidQuery)?;
                        QueryType::Newer(s, Self::u64_to_systemtime(time))
                    }
                    "older" => {
                        let time = query_value
                            .parse::<u64>()
                            .map_err(|_| InvalidQuery::InvalidQuery)?;
                        QueryType::Older(s, Self::u64_to_systemtime(time))
                    }
                    &_ => {
                        return Err(InvalidQuery::InvalidQuery);
                    }
                };
                Ok(mappedquery)
            })
            .filter_map(|x| match x {
                Ok(q) => Some(q),
                Err(_) => None,
            })
            .collect();
        match &mut self.root {
            Some(root) => root.search(&queries),
            None => MatchResult::default()
        }
    }
}
