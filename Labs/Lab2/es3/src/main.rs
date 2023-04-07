use dirinfo::{FileOrDirError, FileSystem};

fn main() -> Result<(), FileOrDirError> {
    let mut fs = FileSystem::from_dir("/home/alessiorosiello/Dev/PdS-Polito2023/Labs/Lab2/es3")?;
    let queries = vec!["name:.toml", "content:main"];
    let res = fs.search(&queries);
    println!("{}", res);
    Ok(())
}
