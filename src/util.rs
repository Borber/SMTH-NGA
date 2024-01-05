use std::fs::OpenOptions;
use std::io::Write;

pub fn write(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个新的文件
    let mut new_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    // 写入 BOM
    new_file.write_all(&[0xEF, 0xBB, 0xBF])?;

    // 写入文件内容
    new_file.write_all(content.as_bytes())?;

    Ok(())
}
