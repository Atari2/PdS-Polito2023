use std::fmt::Display;

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

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Text => write!(f, "Text"),
            FileType::Binary => write!(f, "Binary"),
        }
    }
}

pub type FsResult<T> = Result<T, FileOrDirError>;
