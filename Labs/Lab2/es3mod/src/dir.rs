use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::node::Node;
use crate::common::{FsResult, FileOrDirError};
use crate::file::File;
use crate::{QueryType, MatchResult, FileType};

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
    pub fn name(&self) -> &PathBuf {
        &self.name
    }
    pub fn creation_time(&self) -> &SystemTime {
        &self.creation_time
    }
    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }
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
            let name_without_parent = child.name().strip_prefix(&self.name).map_err(|_| std::fmt::Error)?;
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
    ) -> FsResult<Dir> {
        Ok(Dir {
            name,
            creation_time,
            children: vec![],
        })
    }
    pub fn new(path: PathBuf) -> FsResult<Dir> {
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
    pub fn mk_dir(&mut self, path: &Path) -> FsResult<()> {
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
    pub fn rm_dir(&mut self, path: &Path) -> FsResult<()> {
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
    pub fn new_file(&mut self, path: &Path) -> FsResult<()> {
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
    pub fn rm_file(&mut self, path: &Path) -> FsResult<()> {
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
    pub fn search<'a>(&'b mut self, queries: &[QueryType<'a>]) -> MatchResult<'a>
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
                                    if file.name().components().any(|c| {
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
                                        let file_contents = match std::str::from_utf8(file.content())
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
                                    if file.content().len() > *size {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Smaller(_, size) => {
                                    if file.content().len() < *size {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Newer(_, timestamp) => {
                                    if file.creation_time() > timestamp {
                                        return Some(MatchResult {
                                            queries: vec![q.to_str()],
                                            nodes: vec![ch],
                                        });
                                    }
                                }
                                QueryType::Older(_, timestamp) => {
                                    if file.creation_time() < timestamp {
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