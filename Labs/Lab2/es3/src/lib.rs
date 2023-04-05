use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub enum FileType {
    Text,
    Binary,
}

#[derive(Debug)]
pub struct File {
    name: String,
    content: Vec<u8>,
    creation_time: u64,
    type_: FileType,
}

pub fn timestamp_to_u64(time: std::time::SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

impl File {
    pub fn new(
        name: String,
        metadata: std::fs::Metadata,
    ) -> Result<File, Box<dyn std::error::Error>> {
        let mut content = vec![];
        let file = OpenOptions::new().read(true).open(name.clone())?;
        let mut reader = BufReader::new(file.take(1000));
        let path = Path::new(&name);
        let extension = match path.extension() {
            Some(ext) => ext.to_str().unwrap_or(""),
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
            creation_time: timestamp_to_u64(metadata.created()?),
            type_,
        })
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

impl Dir {

    fn directory_structure(&self, f: &mut std::fmt::Formatter<'_>, depth: &mut usize) -> std::fmt::Result {
        let indent_string = str::repeat("\t", *depth);
        *depth += 1;
        let next_indent = str::repeat("\t", *depth);
        f.write_fmt(format_args!("{}{}\n", indent_string, self.name))?;
        for child in self.children.iter() {
            match child {
                Node::Dir(dir) => dir.directory_structure(f, depth)?,
                Node::File(file) => f.write_fmt(format_args!("{}{}\n", next_indent, file.name))?
            }
        }
        Ok(())
    }

    pub fn empty_from_parts(path: &str, creation_time: u64) -> Dir {
        Dir {
            name: path.to_string(),
            creation_time,
            children: vec![],
        }
    }
    pub fn new(path: &str) -> Result<Dir, Box<dyn std::error::Error>> {
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
    pub fn mk_dir(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if path.starts_with(&self.name) {
            match path.parent() {
                Some(parent) => {
                    if let Some(parent_as_str) = parent.to_str() {
                        if parent_as_str == self.name {
                            // we're the parent of the mkdir
                            // maybe remove the unwrap but I don't care
                            self.children.push(Node::Dir(Dir::empty_from_parts(
                                path.to_str().unwrap(),
                                timestamp_to_u64(std::time::SystemTime::now()),
                            )));
                            return Ok(());
                        }
                    }
                }
                None => {
                    return Err(Box::from("Unexpected error"));
                }
            }
            // one of our children may have the path
            for ch in self.children.iter_mut() {
                match ch {
                    Node::File(_) => {}
                    Node::Dir(node) => {
                        if node.mk_dir(path).is_ok() {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(Box::from("Path wasn't contained in this filesystem"))
    }
}

#[derive(Debug)]
enum Node {
    File(File),
    Dir(Dir),
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

pub struct MatchResult<'a> {
    queries: Vec<&'a str>,
    nodes: Vec<&'a mut Node>,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root: Dir::default(),
        }
    }
    pub fn from_dir(path: &str) -> Result<FileSystem, Box<dyn std::error::Error>> {
        let mut fs = FileSystem::new();
        fs.root = Dir::new(path)?;
        Ok(fs)
    }
    pub fn mk_dir(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut pb = PathBuf::from(path);
        if !pb.is_absolute() {
            pb = PathBuf::from(&self.root.name).join(pb);
        }
        self.root.mk_dir(&pb)?;
        Ok(())
    }
    pub fn rm_dir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(std::fs::remove_dir(path)?) // remove_dir does all the checks for us
    }
    // new_file(path: &str, file: File) -> Result<(), Box<dyn std::error::Error>>
    // rm_file(path: &str) -> Result<(), Box<dyn std::error::Error>>
    // get_file(path: &str) -> Option<&mut File>
    // search(queries: &[&str]) -> MatchResult
}
