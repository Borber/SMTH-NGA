#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("NGA_Checkin_Stats.csv")?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut flag = HashSet::new();
    let mut valid = vec![];
    let mut multi = vec![];
    let mut over = vec![];

    let mut records = vec![];

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
            over.push(record)
        } else {
            let time = record[1].to_owned();
            let time = time.split_once(' ').unwrap().0.to_owned();
            let name = record[2].to_owned();
            match flag.get(&(time.clone(), name.clone())) {
                Some(_) => {
                    multi.push(record);
                }
                None => {
                    count += 1;
                    flag.insert((time, name));
                    valid.push(record);
                }
            }
        }
    }

    let file = File::open("NGA_Invalid_Reply.csv")?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut invalid = vec![];

    for result in rdr.records() {
        let record = result?;
        invalid.push(record);
    }

    let mut wtr = csv::WriterBuilder::new().from_path("valid.csv")?;

    for record in valid {
        wtr.write_record(&record)?;
    }
    wtr.flush()?;
    add_bom("valid.csv")?;

    let mut wtr = csv::WriterBuilder::new().from_path("invalid.csv")?;
    for record in invalid {
        wtr.write_record(&record)?;
    }

    println!("{}", multi.len());

    wtr.write_record(["以下为多次回复", "", ""])?;
    for record in multi {
        wtr.write_record(&record)?;
    }
    wtr.write_record(["以下为超过700条数据", "", ""])?;
    for record in over {
        wtr.write_record(&record)?;
    }
    wtr.flush()?;
    add_bom("invalid.csv")?;

    Ok(())
}

fn add_bom(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 打开要处理的文件，并读取文件内容
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // 创建一个新的文件，并将 BOM 和原始文件内容写入新文件
    let mut new_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    // 写入 BOM
    new_file.write_all(&[0xEF, 0xBB, 0xBF])?;

    // 写入原始文件内容
    new_file.write_all(&content)?;

    Ok(())
}
