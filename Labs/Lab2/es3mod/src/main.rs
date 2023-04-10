use dirinfo::{FileOrDirError, FileSystem};

#[cfg(target_os = "linux")]
fn main() -> Result<(), FileOrDirError> {
    let cwd = std::env::current_dir()?;
    let mut fs = FileSystem::from_dir(cwd.to_str().ok_or(FileOrDirError::InvalidUtf8)?)?;
    let queries = vec!["name:.toml", "content:main"];
    let res = fs.search(&queries);
    println!("{}", res);
    Ok(())
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), FileOrDirError> {
    let mut cwd = std::env::current_dir()?;
    cwd = cwd.parent().ok_or(FileOrDirError::ParentDoesNotExist)?.to_path_buf();
    cwd.push("es1");
    cwd.push("binary_io");
    let mut fs = FileSystem::from_dir(cwd.to_str().ok_or(FileOrDirError::InvalidUtf8)?)?;
    println!("{}", fs);
    let queries = vec!["name:.toml", "content:main"];
    let res = fs.search(&queries);
    println!("{}", res);
    Ok(())
}