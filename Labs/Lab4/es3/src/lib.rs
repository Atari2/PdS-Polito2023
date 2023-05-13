use std::cell::{RefCell};
use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::ops::{Deref};
use std::path::{Path, PathBuf};
use std::rc::Rc;
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    pub fn creation_time(&self) -> &u64 {
        &self.creation_time
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

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}

#[derive(Debug, Default)]
pub struct Dir {
    name: String,
    creation_time: u64,
    children: Vec<Rc<RefCell<Node>>>,
}

impl<'b> Dir {
    pub fn children(&self) -> &Vec<Rc<RefCell<Node>>> { &self.children }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn creation_time(&self) -> &u64 {
        &self.creation_time
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
                let node = Node::Dir(Rc::new(RefCell::new(Dir::new(name)?)));
                dir.children
                    .push(Rc::new(RefCell::new(node)));
            } else if metadata.is_file() {
                let filename = entry.path();
                let name = match filename.to_str() {
                    Some(name) => name,
                    None => continue,
                };
                let node = Node::File(Rc::new(RefCell::new(File::new(
                    name.to_string(),
                    entry.metadata()?,
                )?)));
                dir.children
                    .push(Rc::new(RefCell::new(node)));
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

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}

#[derive(Debug)]
pub enum Node {
    File(Rc<RefCell<File>>),
    Dir(Rc<RefCell<Dir>>),
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(file) => {
                f.write_fmt(format_args!("{}", file.borrow()))
            }
            Self::Dir(dir) => {
                f.write_fmt(format_args!("{}", dir.borrow()))
            }
        }
    }
}

trait Search {
    fn search<'a>(
        self,
        queries: &[QueryType<'a>],
        result: MatchResult<'a>,
    ) -> MatchResult<'a>;
}

impl Search for Rc<RefCell<Dir>> {
    fn search<'a>(
        self,
        queries: &[QueryType<'a>],
        mut result: MatchResult<'a>,
    ) -> MatchResult<'a>{
        let s = self.borrow_mut();
        for child in s.children().iter() {
            let c = Rc::clone(child);
            result = c.search(queries, result)
        }
        result.queries.sort_unstable();
        result.queries.dedup();
        result
    }
}

impl Search for Rc<RefCell<Node>> {
    fn search<'a>(
        self,
        queries: &[QueryType<'a>],
        mut result: MatchResult<'a>,
    ) -> MatchResult<'a>
    {
        {
            let node = self.borrow_mut();
            if let Some(q) = queries.iter().find(|q| { q.matches(&node) }) {
                result.queries.push(q.to_str());
                result.nodes.push(Rc::clone(&self));
            }
        }
        {
            let node = self.borrow_mut();
            match node.deref() {
                Node::Dir(s) => {
                    let c = Rc::clone(s);
                    result = c.search(queries, result);
                }
                _ => {}
            }
        }
        result
    }
}

impl PartialEq<Path> for Node {
    fn eq(&self, other: &Path) -> bool {
        match self {
            Self::File(f) => {
                let binding = f.borrow();
                Path::new(binding.name.as_str()) == other
            }
            Self::Dir(d) => {
                let binding = d.borrow();
                Path::new(binding.name.as_str()) == other
            }
        }
    }
}

impl PartialEq<PathBuf> for Node {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other.as_path()
    }
}

#[derive(Debug, Default)]
pub struct FileSystem {
    root: Rc<RefCell<Dir>>
}

#[derive(Debug, Default)]
pub struct MatchResult<'a> {
    pub queries: Vec<&'a str>,
    pub nodes: Vec<Rc<RefCell<Node>>>,
}

impl Display for MatchResult<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
            result.push_str(&format!("\n\t{}", node.borrow().deref()));
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
            Self::Name(_, name) => file.name().contains(name),
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
            Self::Newer(_, time) => file.creation_time() > time,
            Self::Older(_, time) => file.creation_time() < time,
        }
    }
    fn matches_dir(&self, dir: &Dir) -> bool {
        match self {
            QueryType::Name(_, name) => dir.name().contains(name),
            QueryType::Content(_, _) => false,
            QueryType::Larger(_, _) => false,
            QueryType::Smaller(_, _) => false,
            QueryType::Newer(_, time) => dir.creation_time() > time,
            QueryType::Older(_, time) => dir.creation_time() < time,
        }
    }
    pub fn matches(&self, node: &Node) -> bool {
        match node {
            Node::File(file) => self.matches_file(&file.borrow()),
            Node::Dir(dir) => self.matches_dir(&dir.borrow()),
        }
    }
}

impl<'b> FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root: Rc::new(RefCell::new(Dir::default())),
        }
    }

    pub fn from_dir(path: &str) -> Result<FileSystem, FileOrDirError> {
        let mut fs = FileSystem::new();
        fs.root = Rc::new(RefCell::new(Dir::new(path)?));
        Ok(fs)
    }
    pub fn search<'a>(&'b mut self, queries: &[&'a str]) -> MatchResult<'a>
    where
        'b: 'a
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
        Rc::clone(&self.root).search(&queries, MatchResult::default())
    }
}
