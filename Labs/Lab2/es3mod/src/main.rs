use dirinfo::{FileOrDirError, FileSystem};

#[cfg(target_os = "linux")]
fn main() -> Result<(), FileOrDirError> {
    let mut fs = FileSystem::from_dir("/home/alessiorosiello/Dev/PdS-Polito2023/Labs/Lab2/es3")?;
    let queries = vec!["name:.toml", "content:main"];
    let res = fs.search(&queries);
    println!("{}", res);
    Ok(())
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), FileOrDirError> {
    let mut fs = FileSystem::from_dir(r#"C:\Users\aless\Programming\PdS-Polito2023\Labs\Lab2\es1\binary_io"#)?;
    println!("{}", fs);
    let queries = vec!["name:.toml", "content:main"];
    let res = fs.search(&queries);
    println!("{}", res);
    Ok(())
}