use dirinfo::FileSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fs = FileSystem::from_dir("/home/alessiorosiello/Dev/PdS-Polito2023/Labs/Lab2/es3/src")?;
    fs.mk_dir("/home/alessiorosiello/Dev/PdS-Polito2023/Labs/Lab2/es3/src/a")?;
    fs.mk_dir("a/b")?;
    println!("{}", fs);
    Ok(())
}
