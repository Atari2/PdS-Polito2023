use std::collections::HashSet;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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
    name: PathBuf,
    content: Vec<u8>,
    creation_time: SystemTime,
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

impl File {
    pub fn name(&self) -> &Path {
        &self.name
    }
    pub fn filetype(&self) -> &FileType {
        &self.type_
    }
    pub fn filename(&self) -> Result<&str, FileOrDirError> {
        self.name
            .file_name()
            .ok_or(FileOrDirError::InvalidUtf8)
            .and_then(|os_str| os_str.to_str().ok_or(FileOrDirError::InvalidUtf8))
    }
    pub fn new(name: PathBuf, metadata: std::fs::Metadata) -> Result<File, FileOrDirError> {
        let mut content = vec![];
        let file = OpenOptions::new().read(true).open(&name)?;
        let mut reader = BufReader::new(file.take(1000));
        let extension = match name.extension() {
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
            name: name.to_path_buf(),
            content,
            creation_time: metadata.created()?,
            type_,
        })
    }
    pub fn from_name(name: &str) -> File {
        File {
            name: PathBuf::from(name),
            content: vec![],
            creation_time: SystemTime::now(),
            type_: FileType::Text,
        }
    }
    pub fn empty_from_parts(
        path: &Path,
        creation_time: SystemTime,
    ) -> Result<File, FileOrDirError> {
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
            name: path.to_path_buf(),
            content: vec![],
            creation_time,
            type_,
        })
    }
}

impl PartialEq<Path> for File {
    fn eq(&self, other: &Path) -> bool {
        self.name == other
    }
}

impl PartialEq<PathBuf> for File {
    fn eq(&self, other: &PathBuf) -> bool {
        self == other
    }
}

#[derive(Debug)]
pub struct Dir {
    name: PathBuf,
    creation_time: SystemTime,
    children: Vec<Node>,
}

impl Default for Dir {
    fn default() -> Self {
        Dir {
            name: PathBuf::default(),
            creation_time: SystemTime::now(),
            children: Vec::default(),
        }
    }
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut depth = 0;
        self.directory_structure(f, &mut depth, None)?;
        Ok(())
    }
}

impl<'b> Dir {
    fn directory_structure(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        depth: &mut usize,
        parent: Option<&'b Dir>,
    ) -> std::fmt::Result {
        let old_depth = *depth;
        let name = match parent {
            Some(parent) => {
                *depth += parent.name.file_name().ok_or(std::fmt::Error)?.len();
                self.name.strip_prefix(&parent.name).map_err(|_| std::fmt::Error)?
            },
            None => {
                let name = self.name.file_name().ok_or(std::fmt::Error)?;
                Path::new(name)
            }
        };
        let indent_string = str::repeat(" ", *depth);
        let next_indent = str::repeat(" ", *depth + name.as_os_str().len());
        if parent.is_none() {
            f.write_fmt(format_args!("  {}\n", name.display()))?;
        } else {
            f.write_fmt(format_args!("{}└--{}\n", indent_string, name.display()))?;
        }
        let mut child_iter = self.children.iter().filter_map(|child| match child {
            Node::File(f) => Some(f),
            _ => None
        }).peekable();
        while let Some(child) = child_iter.next() {
            let name_without_parent = child.name.strip_prefix(&self.name).map_err(|_| std::fmt::Error)?;
            if child_iter.peek().is_some() {
                f.write_fmt(format_args!("{}|--{}\n", next_indent, name_without_parent.display()))?;
            } else {
                f.write_fmt(format_args!("{}└--{}\n", next_indent, name_without_parent.display()))?;
            }
        }
        for child in self.children.iter() {
            if let Node::Dir(dir) = child {
                dir.directory_structure(f, depth, Some(self))?;
            }
        }
        *depth = old_depth;
        Ok(())
    }

    pub fn empty_from_parts(
        name: PathBuf,
        creation_time: SystemTime,
    ) -> Result<Dir, FileOrDirError> {
        Ok(Dir {
            name,
            creation_time,
            children: vec![],
        })
    }
    pub fn new(path: PathBuf) -> Result<Dir, FileOrDirError> {
        let mut dir = Dir {
            name: path,
            creation_time: SystemTime::now(),
            children: vec![],
        };
        let info = std::fs::metadata(&dir.name)?;
        dir.creation_time = info.created()?;
        let dirit = std::fs::read_dir(&dir.name)?;
        for entry in dirit {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                dir.children.push(Node::Dir(Dir::new(entry.path())?));
            } else if metadata.is_file() {
                dir.children
                    .push(Node::File(File::new(entry.path(), entry.metadata()?)?));
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
                            path.to_path_buf(),
                            SystemTime::now(),
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
            let parent = path.parent().ok_or(FileOrDirError::ParentDoesNotExist)?;
            if self == path {
                return Err(FileOrDirError::AlreadyExists);
            } else if self == parent {
                if self.children.iter().any(|ch| ch == path) {
                    return Err(FileOrDirError::AlreadyExists);
                }
                self.children
                    .push(Node::File(File::empty_from_parts(path, SystemTime::now())?));
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
    fn search<'a>(&'b mut self, queries: &[QueryType<'a>]) -> MatchResult<'a>
    where
        'b: 'a,
    {
        let mut result = MatchResult {
            queries: vec![],
            nodes: vec![],
        };
        let partials = self
            .children
            .iter_mut()
            .filter_map(|ch| {
                match ch {
                    Node::File(file) => {
                        for q in queries.iter() {
                            match q {
                                QueryType::Name(_, name) => {
                                    if file.name.components().any(|c| {
                                        match c.as_os_str().to_str() {
                                            Some(s) => s.contains(name),
                                            None => false,
                                        }
                                    }) {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Content(_, content) => {
                                    if *file.filetype() == FileType::Text {
                                        let file_contents = match std::str::from_utf8(&file.content)
                                        {
                                            Ok(content) => content,
                                            Err(_) => continue,
                                        };
                                        if file_contents.contains(content) {
                                            return Some(MatchResult {
                                                queries: vec![q.to_str()],
                                                nodes: vec![ch],
                                            });
                                        }
                                    }
                                }
                                QueryType::Larger(_, size) => {
                                    if file.content.len() > *size {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Smaller(_, size) => {
                                    if file.content.len() < *size {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Newer(_, timestamp) => {
                                    if file.creation_time > *timestamp {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Older(_, timestamp) => {
                                    if file.creation_time < *timestamp {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Node::Dir(dir) => {
                        let partial = dir.search(queries);
                        return Some(partial);
                    }
                }
                None
            })
            .collect::<Vec<MatchResult>>();
        for partial in partials {
            result.queries.extend(partial.queries);
            result.nodes.extend(partial.nodes);
        }
        result.queries = result.queries.into_iter().collect::<HashSet<_>>().into_iter().collect();
        result
    }
}

impl PartialEq<Path> for Dir {
    fn eq(&self, other: &Path) -> bool {
        self.name == other
    }
}

impl PartialEq<PathBuf> for Dir {
    fn eq(&self, other: &PathBuf) -> bool {
        &self.name == other
    }
}

#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl PartialEq<Path> for Node {
    fn eq(&self, other: &Path) -> bool {
        match self {
            Self::File(f) => f.name == other,
            Self::Dir(d) => d.name == other,
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

enum QueryType<'a> {
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
    fn make_absolute(&self, pb: &str) -> Result<PathBuf, FileOrDirError> {
        let root = self.root.as_ref().ok_or(FileOrDirError::ParentDoesNotExist)?;
        let mut pb = PathBuf::from(pb);
        if !pb.is_absolute() {
            pb = PathBuf::from(&root.name).join(pb);
        }
        Ok(pb)
    }

    fn make_absolute_no_borrow(root: &Dir, pb: &str) -> Result<PathBuf, FileOrDirError> {
        let mut pb = PathBuf::from(pb);
        if !pb.is_absolute() {
            pb = PathBuf::from(&root.name).join(pb);
        }
        Ok(pb)
    }

    #[cfg(target_os = "windows")]
    fn make_root_abs(&mut self, path: &str) -> Result<(), FileOrDirError> {
        let mut path = PathBuf::from(path);
        if !path.is_absolute() {
            path = PathBuf::from("C:\\").join(path);
        }
        self.root = Some(Dir::empty_from_parts(path, SystemTime::now())?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn make_root_abs(&mut self, path: &str) -> Result<(), FileOrDirError> {
        let mut path = PathBuf::from(path);
        if !path.starts_with(std::path::MAIN_SEPARATOR_STR) {
            path = PathBuf::from(std::path::MAIN_SEPARATOR_STR).join(path);
        }
        self.root = Dir::empty_from_parts(path, std::time::SystemTime::now())?;
        Ok(())
    }

    pub fn from_dir(path: &str) -> Result<FileSystem, FileOrDirError> {
        let mut fs = FileSystem::new();
        fs.root = Some(Dir::new(PathBuf::from(path))?);
        Ok(fs)
    }
    pub fn mk_dir(&mut self, path: &str) -> Result<(), FileOrDirError> {
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
    pub fn rm_dir(&mut self, path: &str) -> Result<(), FileOrDirError> {
        let pb = self.make_absolute(path)?;
        let root = self.root.as_mut().ok_or(FileOrDirError::ParentDoesNotExist)?;
        if *root == pb {
            if !root.children.is_empty() {
                return Err(FileOrDirError::DirectoryNotEmpty);
            }
            self.root = None;
        } else {
            root.rm_dir(&pb)?;
        }
        Ok(())
    }
    /* accordign to homework sheet signature should be &mut self, path: &str, file: File but since path is already contained in File it doesn't make sense to duplicate the information */
    pub fn new_file(&mut self, file: File) -> Result<(), FileOrDirError> {
        let root = self.root.as_mut().ok_or(FileOrDirError::ParentDoesNotExist)?;
        let pb = PathBuf::from(&root.name).join(file.name);
        if *root == pb {
            return Err(FileOrDirError::AlreadyExists);
        }
        root.new_file(&pb)
    }
    pub fn rm_file(&mut self, path: &str) -> Result<(), FileOrDirError> {
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
