use cgr::classfile::ClassFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args()
        .nth(1)
        .expect("First argument is input class file");
    let file = std::fs::read(file)?;
    let file = match ClassFile::read(&file) {
        Ok((_, file)) => file,
        Err(err) => return Err(err.to_owned().into()),
    };

    println!("{file:#?}");

    Ok(())
}
