#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::Write,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 标记当日是否回复
    let mut flag = HashSet::new();

    // 有效回复
    let mut valid = "".to_owned();
    // 多次回复
    let mut multi = "".to_owned();
    // 超过700条
    let mut over = "".to_owned();

    // 记录
    let mut records = vec![];

    let file = File::open("NGA_Checkin_Stats.csv")?;

    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        records.push(record);
    }
    records.sort_by(|a, b| {
        let a = a[0].to_owned();
        let a = a.parse::<u64>().unwrap();
        let b = b[0].to_owned();
        let b = b.parse::<u64>().unwrap();
        a.cmp(&b)
    });

    let mut count = 0;
    for record in records {
        if count >= 700 {
            over = format!(
                "{}{}\n",
                over,
                record.into_iter().collect::<Vec<_>>().join(",")
            )
        } else {
            let time = record[1].to_owned();
            let time = time.split_once(' ').unwrap().0.to_owned();
            let name = record[2].to_owned();
            match flag.get(&(time.clone(), name.clone())) {
                Some(_) => {
                    multi = format!(
                        "{}{}\n",
                        multi,
                        record.into_iter().collect::<Vec<_>>().join(",")
                    )
                }
                None => {
                    count += 1;
                    flag.insert((time, name));
                    valid = format!(
                        "{}{}\n",
                        valid,
                        record.into_iter().collect::<Vec<_>>().join(",")
                    )
                }
            }
        }
    }

    let file = File::open("NGA_Invalid_Reply.csv")?;
    let mut rdr = csv::Reader::from_reader(file);

    // 无效回复
    let mut invalid = "".to_owned();
    for result in rdr.records() {
        let record = result?;
        invalid = format!(
            "{}{}\n",
            invalid,
            record.into_iter().collect::<Vec<_>>().join(",")
        );
    }

    invalid = format!(
        "{}以下为多次回复\n{}以下为超过700条数据\n{}",
        invalid, multi, over
    );

    write("valid.csv", &valid)?;
    write("invalid.csv", &invalid)?;

    Ok(())
}

fn write(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
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
