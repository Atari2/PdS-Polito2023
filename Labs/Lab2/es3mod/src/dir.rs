use crate::common::{FileOrDirError, FsResult};
use crate::file::File;
use crate::node::Node;
use crate::{MatchResult, QueryType};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

impl Display for Dir {
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
                self.name
                    .strip_prefix(&parent.name)
                    .map_err(|_| std::fmt::Error)?
            }
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
        let mut child_iter = self
            .children
            .iter()
            .filter_map(|child| match child {
                Node::File(f) => Some(f),
                _ => None,
            })
            .peekable();
        while let Some(child) = child_iter.next() {
            let name_without_parent = child
                .name()
                .strip_prefix(&self.name)
                .map_err(|_| std::fmt::Error)?;
            if child_iter.peek().is_some() {
                f.write_fmt(format_args!(
                    "{}|--{}\n",
                    next_indent,
                    name_without_parent.display()
                ))?;
            } else {
                f.write_fmt(format_args!(
                    "{}└--{}\n",
                    next_indent,
                    name_without_parent.display()
                ))?;
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

    pub fn empty_from_parts(name: PathBuf, creation_time: SystemTime) -> FsResult<Dir> {
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
        struct SingleMatch<'a> {
            query: &'a str,
            node: &'a mut Node,
        }
        enum PartialResult<'a> {
            File(SingleMatch<'a>),
            Dir(MatchResult<'a>),
        }
        let mut result = MatchResult {
            queries: vec![],
            nodes: vec![],
        };
        let partials = self
            .children
            .iter_mut()
            .filter_map(|ch| {
                match ch {
                    Node::File(_) => {
                        for q in queries.iter() {
                            if q.matches(ch) {
                                return Some(PartialResult::File(SingleMatch {
                                    query: q.to_str(),
                                    node: ch,
                                }));
                            }
                        }
                    }
                    Node::Dir(dir) => {
                        let dir_partials = dir.search(queries);
                        // so here there should be the part where I check if dir matches any of the queries as well
                        // however I can't seem to find a way to do a search on the dir and at the same time
                        // because I can't call q.matches(ch) because I can't borrow it again
                        // and I can't assign ch to node in SingleMatch for the same reason
                        // everything I tried results in the same issue every single time and at this point I'm not sure what to do.
                        /* 
                        for q in queries.iter() {
                            if q.matches(ch) {
                                dir_partials.queries.push(q.to_str());
                                dir_partials.nodes.push(ch);
                                break;
                            }
                        }
                        */
                        return Some(PartialResult::Dir(dir_partials));
                    }
                }
                None
            })
            .collect::<Vec<PartialResult>>();
        for partial in partials {
            match partial {
                PartialResult::File(SingleMatch { query, node }) => {
                    result.queries.push(query);
                    result.nodes.push(node);
                }
                PartialResult::Dir(MatchResult { queries, nodes }) => {
                    result.queries.extend(queries);
                    result.nodes.extend(nodes);
                }
            }
        }
        result.queries.sort_unstable();
        result.queries.dedup();
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
