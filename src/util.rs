use std::fs::OpenOptions;
use std::io::Write;

use chrono::{Datelike, Duration, Utc};

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

// 获取上月月份
pub fn get_last_month_date() -> String {
    let now = Utc::now();

    let last_month = (now - Duration::days(20)).with_day(1).unwrap();
    format!("{}-{:02}", last_month.year(), last_month.month())
}
