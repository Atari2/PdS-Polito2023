use dirinfo::{File, FileSystem, FileType};

#[test]
pub fn test_mkdir() {
    let mut fs = FileSystem::new();
    assert!(fs.mk_dir("a").is_ok());
    assert!(fs.mk_dir("/a/b").is_ok());
    assert!(fs.mk_dir("/a/b/c").is_ok());
    assert!(fs.mk_dir("/a/b/d").is_ok());
    let expect_err = fs.mk_dir("b/d");
    assert!(expect_err.is_err());
    let errstr = expect_err.err().unwrap().to_string();
    assert!(errstr == "File or directory already exists");
}

#[test]
pub fn test_rmdir() {
    let mut fs = FileSystem::new();
    assert!(fs.mk_dir("a").is_ok());
    assert!(fs.mk_dir("/a/b").is_ok());
    assert!(fs.mk_dir("/a/b/c").is_ok());
    assert!(fs.rm_dir("/a/b/c").is_ok());
    assert!(fs.rm_dir("/a").is_err());
    assert!(fs.rm_dir("/a/b").is_ok());
    assert!(fs.rm_dir("/a").is_ok());
}

#[test]
pub fn test_new_file() {
    let mut fs = FileSystem::new();
    fs.mk_dir("a").unwrap();
    fs.mk_dir("/a/b").unwrap();
    fs.mk_dir("/a/c").unwrap();
    assert!(fs.new_file(File::from_name("/a/test.txt")).is_ok());
    assert!(fs.new_file(File::from_name("/a/b/test.txt")).is_ok());
    assert!(fs.new_file(File::from_name("/a/c/test.txt")).is_ok());
    assert!(fs.new_file(File::from_name("/a/test.txt")).is_err());
    assert!(fs.new_file(File::from_name("/a/b/test.txt")).is_err());
    assert!(fs.new_file(File::from_name("/a/c/test.txt")).is_err());
    assert!(fs.new_file(File::from_name("/a/d/test.txt")).is_err());
    assert!(fs.new_file(File::from_name("/a/b/test2.bin")).is_ok());
}

#[test]
pub fn test_rm_file() {
    let mut fs = FileSystem::new();
    fs.mk_dir("a").unwrap();
    fs.mk_dir("/a/b").unwrap();
    fs.mk_dir("/a/c").unwrap();
    fs.new_file(File::from_name("/a/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/c/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test2.bin")).unwrap();
    assert!(fs.rm_file("/a/test.txt").is_ok());
    assert!(fs.rm_file("/a/b/test.txt").is_ok());
    assert!(fs.rm_file("/a/c/test.txt").is_ok());
    assert!(fs.rm_file("/a/test.txt").is_err());
    assert!(fs.rm_file("/a/b/test.txt").is_err());
    assert!(fs.rm_file("/a/c/test.txt").is_err());
    assert!(fs.rm_file("/a/d/test.txt").is_err());
    assert!(fs.rm_file("/a/b/test2.bin").is_ok());
}

#[test]
#[cfg(target_os = "linux")]
pub fn test_get_file() {
    use std::path::PathBuf;

    let mut fs = FileSystem::new();
    fs.mk_dir("a").unwrap();
    fs.mk_dir("/a/b").unwrap();
    fs.mk_dir("/a/c").unwrap();
    fs.new_file(File::from_name("/a/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/c/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test2.bin")).unwrap();
    let testtxt = fs.get_file("/a/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("/a/test.txt"));
    let filename = testtxt.filename();
    assert!(filename.is_ok());
    let filename = filename.unwrap();
    assert!(filename == "test.txt");
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("/a/b/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("/a/b/test.txt"));
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("/a/c/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("/a/c/test.txt"));
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("/a/b/test2.bin");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("/a/b/test2.bin"));
    assert!(*testtxt.filetype() == FileType::Binary);
    let testtxt = fs.get_file("/a/d/test.txt");
    assert!(testtxt.is_none());
}

#[test]
#[cfg(target_os = "windows")]
pub fn test_get_file() {
    use std::path::PathBuf;

    let mut fs = FileSystem::new();
    fs.mk_dir("a").unwrap();
    fs.mk_dir("/a/b").unwrap();
    fs.mk_dir("/a/c").unwrap();
    fs.new_file(File::from_name("/a/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/c/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test2.bin")).unwrap();
    let testtxt = fs.get_file("/a/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("C:/a/test.txt"));
    let filename = testtxt.filename();
    assert!(filename.is_ok());
    let filename = filename.unwrap();
    assert!(filename == "test.txt");
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("C:/a/b/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("C:/a/b/test.txt"));
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("C:/a/c/test.txt");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("C:/a/c/test.txt"));
    assert!(*testtxt.filetype() == FileType::Text);
    let testtxt = fs.get_file("C:/a/b/test2.bin");
    assert!(testtxt.is_some());
    let testtxt = testtxt.unwrap();
    assert!(testtxt.name() == PathBuf::from("C:/a/b/test2.bin"));
    assert!(*testtxt.filetype() == FileType::Binary);
    let testtxt = fs.get_file("C:/a/d/test.txt");
    assert!(testtxt.is_none());
}

#[test]
pub fn test_query_fs() {
    let mut fs = FileSystem::new();
    fs.mk_dir("a").unwrap();
    fs.mk_dir("/a/b").unwrap();
    fs.mk_dir("/a/c").unwrap();
    fs.new_file(File::from_name("/a/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/c/test.txt")).unwrap();
    fs.new_file(File::from_name("/a/b/test2.bin")).unwrap();
    let queries = vec!["name:test"];
    let result = fs.search(&queries);
    assert!(result.queries.len() == 1);
    assert!(result.nodes.len() == 4);
}
