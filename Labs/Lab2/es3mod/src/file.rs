use crate::common::{FileOrDirError, FileType, FsResult};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
    pub fn content(&self) -> &[u8] {
        &self.content
    }
    pub fn creation_time(&self) -> &SystemTime {
        &self.creation_time
    }
    pub fn filename(&self) -> FsResult<&str> {
        self.name
            .file_name()
            .ok_or(FileOrDirError::InvalidUtf8)
            .and_then(|os_str| os_str.to_str().ok_or(FileOrDirError::InvalidUtf8))
    }
    pub fn new(name: PathBuf, metadata: std::fs::Metadata) -> FsResult<File> {
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
    pub fn empty_from_parts(path: &Path, creation_time: SystemTime) -> FsResult<File> {
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

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "File {{ Name: {}, Type: {} }}", self.name.display(), self.type_)
    }
}