use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Text,
    Binary,
}

#[derive(Debug)]
pub enum FileOrDirError {
    IoError(std::io::Error),
    SystemTimeError(std::time::SystemTimeError),
    InvalidUtf8,
    AlreadyExists,
    ParentDoesNotExist,
    DirectoryNotEmpty,
    IsDirectory,
}

impl Display for FileOrDirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOrDirError::IoError(e) => write!(f, "IO error: {}", e),
            FileOrDirError::SystemTimeError(e) => write!(f, "System time error: {}", e),
            FileOrDirError::InvalidUtf8 => write!(f, "Invalid UTF-8 in path"),
            FileOrDirError::AlreadyExists => write!(f, "File or directory already exists"),
            FileOrDirError::ParentDoesNotExist => write!(f, "Parent directory does not exist"),
            FileOrDirError::DirectoryNotEmpty => write!(f, "Directory is not empty"),
            FileOrDirError::IsDirectory => write!(f, "Is a directory"),
        }
    }
}

impl std::error::Error for FileOrDirError {}

impl From<std::io::Error> for FileOrDirError {
    fn from(e: std::io::Error) -> Self {
        FileOrDirError::IoError(e)
    }
}
impl From<std::time::SystemTimeError> for FileOrDirError {
    fn from(e: std::time::SystemTimeError) -> Self {
        FileOrDirError::SystemTimeError(e)
    }
}

pub struct File {
    name: String,
    content: Vec<u8>,
    creation_time: u64,
    type_: FileType,
}

impl std::fmt::Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("name", &self.name)
            .field("type_", &self.type_)
            .finish()
    }
}

pub fn timestamp_to_u64(time: std::time::SystemTime) -> Result<u64, std::time::SystemTimeError> {
    Ok(time.duration_since(UNIX_EPOCH)?.as_secs())
}

impl File {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn filetype(&self) -> &FileType {
        &self.type_
    }
    pub fn filename(&self) -> Result<&str, FileOrDirError> {
        Path::new(&self.name)
            .file_name()
            .ok_or(FileOrDirError::InvalidUtf8)
            .and_then(|os_str| os_str.to_str().ok_or(FileOrDirError::InvalidUtf8))
    }
    pub fn content(&self) -> &[u8] {
        &self.content
    }
    pub fn creation_time(&self) -> u64 {
        self.creation_time
    }
    pub fn new(name: String, metadata: std::fs::Metadata) -> Result<File, FileOrDirError> {
        let mut content = vec![];
        let file = OpenOptions::new().read(true).open(&name)?;
        let mut reader = BufReader::new(file.take(1000));
        let path = Path::new(&name);
        let extension = match path.extension() {
            Some(ext) => ext.to_str().ok_or(FileOrDirError::InvalidUtf8)?,
            None => "",
        };
        let type_ = match extension {
            "txt" | "md" | "rs" | "py" | "js" | "html" | "css" | "json" | "toml" | "yaml"
            | "yml" => FileType::Text,
            _ => FileType::Binary,
        };
        reader.read_to_end(&mut content)?;
        Ok(File {
            name,
            content,
            creation_time: timestamp_to_u64(metadata.created()?)?,
            type_,
        })
    }
    pub fn from_name(name: &str) -> File {
        File {
            name: name.to_string(),
            content: vec![],
            creation_time: 0,
            type_: FileType::Text,
        }
    }
    pub fn empty_from_parts(path: &Path, creation_time: u64) -> Result<File, FileOrDirError> {
        let name = path
            .to_str()
            .ok_or(FileOrDirError::InvalidUtf8)?
            .to_string();
        let extension = match path.extension() {
            Some(ext) => ext.to_str().ok_or(FileOrDirError::InvalidUtf8)?,
            None => "",
        };
        let type_ = match extension {
            "txt" | "md" | "rs" | "py" | "js" | "html" | "css" | "json" | "toml" | "yaml"
            | "yml" => FileType::Text,
            _ => FileType::Binary,
        };
        Ok(File {
            name,
            content: vec![],
            creation_time,
            type_,
        })
    }
}

impl PartialEq<Path> for File {
    fn eq(&self, other: &Path) -> bool {
        let self_as_path = Path::new(self.name.as_str());
        self_as_path == other
    }
}

impl PartialEq<PathBuf> for File {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other.as_path()
    }
}

#[derive(Debug, Default)]
pub struct Dir {
    name: String,
    creation_time: u64,
    children: Vec<Node>,
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut depth = 0;
        self.directory_structure(f, &mut depth)?;
        Ok(())
    }
}

impl<'b> Dir {
    fn directory_structure(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        depth: &mut usize,
    ) -> std::fmt::Result {
        let indent_string = str::repeat("  ", *depth);
        *depth += 1;
        let next_indent = str::repeat("  ", *depth);
        f.write_fmt(format_args!("{}{}\n", indent_string, self.name))?;
        for child in self.children.iter() {
            match child {
                Node::Dir(dir) => dir.directory_structure(f, depth)?,
                Node::File(file) => f.write_fmt(format_args!("{}{}\n", next_indent, file.name))?,
            }
        }
        *depth -= 1;
        Ok(())
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn creation_time(&self) -> u64 {
        self.creation_time
    }
    pub fn empty_from_parts(path: &Path, creation_time: u64) -> Result<Dir, FileOrDirError> {
        let path = path.to_str().ok_or(FileOrDirError::InvalidUtf8)?;
        Ok(Dir {
            name: path.to_string(),
            creation_time,
            children: vec![],
        })
    }
    pub fn new(path: &str) -> Result<Dir, FileOrDirError> {
        let mut dir = Dir {
            name: path.to_string(),
            creation_time: 0,
            children: vec![],
        };
        let info = std::fs::metadata(path)?;
        dir.creation_time = info.created()?.duration_since(UNIX_EPOCH)?.as_secs();
        let dirit = std::fs::read_dir(path)?;
        for entry in dirit {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                let filename = entry.path();
                let name = match filename.to_str() {
                    Some(name) => name,
                    None => continue,
                };
                dir.children.push(Node::Dir(Dir::new(name)?));
            } else if metadata.is_file() {
                let filename = entry.path();
                let name = match filename.to_str() {
                    Some(name) => name,
                    None => continue,
                };
                dir.children
                    .push(Node::File(File::new(name.to_string(), entry.metadata()?)?));
            } else if metadata.is_symlink() {
                // we ignore symlinks for now
            } else {
                println!(
                    "Found something else: {:?}, type is {:?}",
                    entry.path(),
                    metadata.file_type()
                );
            }
        }
        Ok(dir)
    }
    pub fn mk_dir(&mut self, path: &Path) -> Result<(), FileOrDirError> {
        if path.starts_with(&self.name) {
            match path.parent() {
                Some(parent) => {
                    if self == parent {
                        if self.children.iter().any(|ch| ch == path) {
                            return Err(FileOrDirError::AlreadyExists);
                        }
                        self.children.push(Node::Dir(Dir::empty_from_parts(
                            path,
                            timestamp_to_u64(std::time::SystemTime::now())?,
                        )?));
                        return Ok(());
                    }
                }
                None => {
                    return Err(FileOrDirError::InvalidUtf8);
                }
            }
            // one of our children may have the path
            for ch in self.children.iter_mut() {
                match ch {
                    Node::File(_) => {}
                    Node::Dir(node) => match node.mk_dir(path) {
                        Ok(_) => return Ok(()),
                        Err(FileOrDirError::ParentDoesNotExist) => {}
                        Err(e) => return Err(e),
                    },
                }
            }
        }
        Err(FileOrDirError::ParentDoesNotExist)
    }
    pub fn rm_dir(&mut self, path: &Path) -> Result<(), FileOrDirError> {
        if path.starts_with(&self.name) {
            // one of our children may have the path
            let mut found = false;
            let mut index: usize = 0;
            for (i, ch) in self.children.iter_mut().enumerate() {
                match ch {
                    Node::File(_) => {}
                    Node::Dir(node) => {
                        if node == path {
                            if !node.children.is_empty() {
                                return Err(FileOrDirError::DirectoryNotEmpty);
                            } else {
                                found = true;
                                index = i;
                                break;
                            }
                        } else if node.rm_dir(path).is_ok() {
                            return Ok(());
                        }
                    }
                }
            }
            if found {
                self.children.remove(index);
                return Ok(());
            }
        }
        Err(FileOrDirError::ParentDoesNotExist)
    }
    pub fn new_file(&mut self, path: &Path) -> Result<(), FileOrDirError> {
        if path.starts_with(&self.name) {
            let parent = match path.parent() {
                Some(parent) => parent,
                None => return Err(FileOrDirError::ParentDoesNotExist),
            };
            if self == path {
                return Err(FileOrDirError::AlreadyExists);
            } else if self == parent {
                if self.children.iter().any(|ch| ch == path) {
                    return Err(FileOrDirError::AlreadyExists);
                }
                self.children.push(Node::File(File::empty_from_parts(
                    path,
                    timestamp_to_u64(std::time::SystemTime::now())?,
                )?));
                return Ok(());
            } else {
                for ch in self.children.iter_mut() {
                    match ch {
                        Node::File(_) => {}
                        Node::Dir(node) => match node.new_file(path) {
                            Ok(_) => return Ok(()),
                            Err(FileOrDirError::ParentDoesNotExist) => {}
                            Err(e) => return Err(e),
                        },
                    }
                }
            }
        }
        Err(FileOrDirError::ParentDoesNotExist)
    }
    pub fn rm_file(&mut self, path: &Path) -> Result<(), FileOrDirError> {
        if self == path {
            return Err(FileOrDirError::IsDirectory);
        } else if path.starts_with(&self.name) {
            let mut found = false;
            let mut index: usize = 0;
            for (i, ch) in self.children.iter_mut().enumerate() {
                match ch {
                    Node::File(node) => {
                        if node == path {
                            found = true;
                            index = i;
                            break;
                        }
                    }
                    Node::Dir(node) => {
                        if node.rm_file(path).is_ok() {
                            return Ok(());
                        }
                    }
                }
            }
            if found {
                self.children.remove(index);
                return Ok(());
            }
        }
        Err(FileOrDirError::ParentDoesNotExist)
    }
    pub fn get_file(&mut self, path: &Path) -> Option<&mut File> {
        if self == path {
            return None;
        } else if path.starts_with(&self.name) {
            for ch in self.children.iter_mut() {
                match ch {
                    Node::File(node) => {
                        if node == path {
                            return Some(node);
                        }
                    }
                    Node::Dir(node) => {
                        if let Some(file) = node.get_file(path) {
                            return Some(file);
                        }
                    }
                }
            }
        }
        None
    }
    fn search<'a>(&'b mut self, queries: &[QueryType<'a>], mut result: MatchResult<'a>) -> MatchResult<'a>
    where
        'b: 'a,
    {
        for child in self.children.iter_mut() {
            result = child.search(queries, result)
        }
        result.queries.sort_unstable();
        result.queries.dedup();
        result
    }
}

impl PartialEq<Path> for Dir {
    fn eq(&self, other: &Path) -> bool {
        let self_as_path = Path::new(self.name.as_str());
        self_as_path == other
    }
}

impl PartialEq<PathBuf> for Dir {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other.as_path()
    }
}

#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl<'b> Node {
    fn search<'a>(&'b mut self, queries: &[QueryType<'a>], mut result: MatchResult<'a>) -> MatchResult<'a>
    where
        'b: 'a,
    {
        match self {
            Self::File(_) => {
                if let Some(q) = queries.iter().find(|q| q.matches(self)) {
                    result.queries.push(q.to_str());
                    result.nodes.push(self);
                }
            },
            Self::Dir(dir) => {
                result = dir.search(queries, result);
                // TODO: find a way to match+add the directory itself
                // there is a trick I found but I'm 99.99999% sure that it is UB
                // it's in the es3mod directory in node.rs line 43/44
            }
        }
        result
    }
}

impl PartialEq<Path> for Node {
    fn eq(&self, other: &Path) -> bool {
        let self_as_path = match self {
            Self::File(f) => Path::new(f.name.as_str()),
            Self::Dir(d) => Path::new(d.name.as_str()),
        };
        self_as_path == other
    }
}

impl PartialEq<PathBuf> for Node {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other.as_path()
    }
}

#[derive(Debug, Default)]
pub struct FileSystem {
    root: Dir,
}

impl std::fmt::Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.root))
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

enum QueryType<'a> {
    Name(&'a str, &'a str),
    Content(&'a str, &'a str),
    Larger(&'a str, usize),
    Smaller(&'a str, usize),
    Newer(&'a str, u64),
    Older(&'a str, u64),
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
    fn matches_file(&self, file: &File) -> bool {
        match self {
            Self::Name(_, name) => file
                .name()
                .contains(name),
            Self::Content(_, content) => {
                if *file.filetype() == FileType::Text {
                    let file_contents = match std::str::from_utf8(file.content()) {
                        Ok(content) => content,
                        Err(_) => return false,
                    };
                    file_contents.contains(content)
                } else {
                    false
                }
            }
            Self::Larger(_, size) => file.content().len() > *size,
            Self::Smaller(_, size) => file.content().len() < *size,
            Self::Newer(_, time) => file.creation_time() > *time,
            Self::Older(_, time) => file.creation_time() < *time,
        }
    }
    fn matches_dir(&self, dir: &Dir) -> bool {
        match self {
            QueryType::Name(_, name) => {
                dir.name()
                    .contains(name)
            }
            QueryType::Content(_, _) => false,
            QueryType::Larger(_, _) => false,
            QueryType::Smaller(_, _) => false,
            QueryType::Newer(_, time) => dir.creation_time() > *time,
            QueryType::Older(_, time) => dir.creation_time() < *time,
        }
    }
    pub fn matches(&self, node: &Node) -> bool {
        match node {
            Node::File(file) => self.matches_file(file),
            Node::Dir(dir) => self.matches_dir(dir),
        }
    }
}

impl<'b> FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root: Dir::default(),
        }
    }
    fn make_absolute(&self, path: &str) -> PathBuf {
        let mut pb = PathBuf::from(path);
        if !pb.is_absolute() {
            pb = PathBuf::from(&self.root.name).join(pb);
        }
        pb
    }

    #[cfg(target_os = "windows")]
    fn make_root_abs(&mut self, path: &str) -> Result<(), FileOrDirError> {
        let mut path = PathBuf::from(path);
        if !path.is_absolute() {
            path = PathBuf::from("C:\\").join(path);
        }
        self.root =
            Dir::empty_from_parts(&path, timestamp_to_u64(std::time::SystemTime::now())?)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn make_root_abs(&mut self, path: &str) -> Result<(), FileOrDirError>  {
        let mut path = PathBuf::from(path);
        if !path.starts_with(std::path::MAIN_SEPARATOR_STR) {
            path = PathBuf::from(std::path::MAIN_SEPARATOR_STR).join(path);
        }
        self.root =
            Dir::empty_from_parts(&path, timestamp_to_u64(std::time::SystemTime::now())?)?;
        Ok(())
    }

    pub fn from_dir(path: &str) -> Result<FileSystem, FileOrDirError> {
        let mut fs = FileSystem::new();
        fs.root = Dir::new(path)?;
        Ok(fs)
    }
    pub fn mk_dir(&mut self, path: &str) -> Result<(), FileOrDirError> {
        // special case empty fs
        let pb = self.make_absolute(path);
        if self.root.name.is_empty() {
            self.make_root_abs(path)?;
        } else {
            self.root.mk_dir(&pb)?;
        }
        Ok(())
    }
    pub fn rm_dir(&mut self, path: &str) -> Result<(), FileOrDirError> {
        let pb = self.make_absolute(path);
        if self.root == pb {
            if !self.root.children.is_empty() {
                return Err(FileOrDirError::DirectoryNotEmpty);
            }
            self.root = Dir::default();
        } else {
            self.root.rm_dir(&pb)?;
        }
        Ok(())
    }
    /* accordign to homework sheet signature should be &mut self, path: &str, file: File but since path is already contained in File it doesn't make sense to duplicate the information */
    pub fn new_file(&mut self, file: File) -> Result<(), FileOrDirError> {
        if self.root.name.is_empty() {
            return Err(FileOrDirError::ParentDoesNotExist);
        }
        let pb = self.make_absolute(&file.name);
        if self.root == pb {
            return Err(FileOrDirError::AlreadyExists);
        }
        self.root.new_file(&pb)
    }
    pub fn rm_file(&mut self, path: &str) -> Result<(), FileOrDirError> {
        if self.root.name.is_empty() {
            return Err(FileOrDirError::ParentDoesNotExist);
        }
        let pb = self.make_absolute(path);
        self.root.rm_file(&pb)
    }
    pub fn get_file(&mut self, path: &str) -> Option<&mut File> {
        if self.root.name.is_empty() {
            return None;
        }
        let pb = self.make_absolute(path);
        self.root.get_file(&pb)
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
                        QueryType::Newer(s, time)
                    }
                    "older" => {
                        let time = query_value
                            .parse::<u64>()
                            .map_err(|_| InvalidQuery::InvalidQuery)?;
                        QueryType::Older(s, time)
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
        self.root.search(&queries, MatchResult::default())
    }
}
