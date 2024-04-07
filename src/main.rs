// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashSet;

use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::Html;
use scraper::Selector;
use serde_json::Value;

use crate::util::get_last_month_date;
use crate::util::write;

mod util;

struct Record {
    count: usize,
    name: String,
    time: String,
    valid: bool,
}

static CLIENT: Lazy<Client> = Lazy::new(reqwest::Client::new);

const COOKIE: &str = "Hm_lvt_2728f3eacf75695538f5b1d1b5594170=1688656511; ngacn0comUserInfo=BORBER%09BORBER%0939%0939%09%0910%09300%094%090%090%0961_3; ngaPassportUid=60414626; ngaPassportUrlencodedUname=BORBER; ngaPassportCid=X9ie0ceeamja9qpii14l46e40h48dljqk3uu96ba; Hm_lvt_6933ef97905336bef84f9609785bcc3d=1701429444; Hm_lpvt_6933ef97905336bef84f9609785bcc3d=1701429531; ngacn0comUserInfoCheck=41020b6e203aacac6900db289217008c; ngacn0comInfoCheckTime=1701437416; lastvisit=1701437435; lastpath=/read.php?tid=20217469&_ff=428&page=478; bbsmisccookies=%7B%22uisetting%22%3A%7B0%3A1%2C1%3A1702034304%7D%2C%22pv_count_for_insad%22%3A%7B0%3A-20%2C1%3A1701450072%7D%2C%22insad_views%22%3A%7B0%3A1%2C1%3A1701450072%7D%7D";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 Edg/119.0.0.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO 获取上一月份
    let last_mouth = get_last_month_date();
    let tid = "20217469";
    let content_records = get_records(tid, &last_mouth).await?;
    println!("总计: {} 条", content_records.len());
    // 内容有效
    let mut content_valid = vec![];

    // 内容无效
    let mut content_invalid = vec![];

    // 此时为时间正序的 回复内容检测数组
    for record in content_records.into_iter().rev() {
        if record.valid {
            content_valid.push(record);
        } else {
            content_invalid.push(record);
        }
    }

    // 有效回复
    let mut valid = vec![];

    // 多次回复
    let mut multi = vec![];

    // 超过700条
    let mut over = vec![];

    // 标记当日是否回复
    let mut flag = HashSet::new();

    let mut count = 0;
    for record in content_valid {
        if count >= 700 {
            over.push(record);
        } else {
            let time = record.time.clone();
            let time = time.split_once(' ').unwrap().0.to_owned();
            match flag.get(&(time.clone(), record.name.clone())) {
                Some(_) => {
                    multi.push(record);
                }
                None => {
                    count += 1;
                    flag.insert((time.clone(), record.name.clone()));
                    valid.push(record);
                }
            }
        }
    }

    println!("有效回复: {} 条", valid.len());
    println!("无效回复: {} 条", content_invalid.len());
    println!("多次回复: {} 条", multi.len());
    println!("超过700: {} 条", over.len());

    // 有效回复
    let valid = valid
        .iter()
        .map(|r| format!("{},{},{}", r.count, r.time, r.name))
        .collect::<Vec<_>>()
        .join("\n");

    // 无效回复
    let invalid = content_invalid
        .iter()
        .map(|r| format!("{},{},{}", r.count, r.time, r.name))
        .collect::<Vec<_>>()
        .join("\n");

    // 多次回复
    let multi = multi
        .iter()
        .map(|r| format!("{},{},{}", r.count, r.time, r.name))
        .collect::<Vec<_>>()
        .join("\n");

    // 超过500条
    let over = over
        .iter()
        .map(|r| format!("{},{},{}", r.count, r.time, r.name))
        .collect::<Vec<_>>()
        .join("\n");

    let invalid = format!(
        "{}\n以下为多次回复\n{}\n以下为超过700条数据\n{}",
        invalid, multi, over
    );

    write("valid.csv", &valid)?;
    write("invalid.csv", &invalid)?;

    Ok(())
}

async fn get_records(tid: &str, last_mouth: &str) -> Result<Vec<Record>> {
    let mut pages = get_pages(tid).await?;

    let mut records = vec![];

    let mut finish = false;
    let mut flag = false;

    while !finish {
        // 回复楼层
        let mut counts = vec![];
        // 回复人ID列表
        let mut names = vec![];
        // 回复时间列表
        let mut times = vec![];
        // 回复内容列表
        let mut valids = vec![];

        let resp = CLIENT
            .get(format!(
                "https://ngabbs.com/read.php?tid={tid}&page={pages}"
            ))
            .header("Cookie", COOKIE)
            .header("User-Agent", USER_AGENT)
            .send()
            .await?
            .text()
            .await?;
        pages -= 1;

        let re = regex::Regex::new(r#"commonui.userInfo.setAll\( (.*?) \)"#).unwrap();
        let user_info_str = re.captures(&resp).unwrap().get(1).unwrap().as_str();
        let user_info: Value = serde_json::from_str(user_info_str).unwrap();

        let document = Html::parse_document(&resp);
        let selector_author = Selector::parse("a.author").unwrap();
        let selector_time = Selector::parse("div.postInfo span").unwrap();
        let selector_context = Selector::parse("span.postcontent").unwrap();

        // 回复人ID 注意此时为倒序
        for element in document.select(&selector_author).rev() {
            let uid = element.attr("href").unwrap().rsplit_once('=').unwrap().1;
            names.push(user_info[uid]["username"].as_str().unwrap());
            let count = element.attr("id").unwrap().replace("postauthor", "");
            let count = count.parse::<usize>().unwrap();
            counts.push(count);
        }

        // 回复时间 注意此时为倒序
        for element in document.select(&selector_time).rev() {
            times.push(element.text().next().unwrap());
        }

        // 回复是否有效 注意此时为倒序
        for element in document.select(&selector_context).rev() {
            let context = element.text().collect::<String>();
            valids.push(check_valid(context));
        }

        for (index, time) in times.iter().enumerate() {
            if !flag && !time.starts_with(last_mouth) {
                continue;
            }

            if !flag && time.starts_with(last_mouth) {
                flag = true;
            }

            if flag && time.starts_with(last_mouth) {
                println!(
                    "{} {} \t{} \t{}",
                    counts[index], names[index], times[index], valids[index]
                );
                records.push(Record {
                    count: counts[index].to_owned(),
                    name: names[index].to_owned(),
                    time: times[index].to_owned(),
                    valid: valids[index],
                });
            }

            if flag && !time.starts_with(last_mouth) {
                finish = true;
                break;
            }
        }
    }
    Ok(records)
}

async fn get_pages(tid: &str) -> Result<usize> {
    let resp = CLIENT.get(format!("https://ngabbs.com/read.php?tid={tid}"))
        .header("Cookie", "Hm_lvt_2728f3eacf75695538f5b1d1b5594170=1688656511; ngacn0comUserInfo=BORBER%09BORBER%0939%0939%09%0910%09300%094%090%090%0961_3; ngaPassportUid=60414626; ngaPassportUrlencodedUname=BORBER; ngaPassportCid=X9ie0ceeamja9qpii14l46e40h48dljqk3uu96ba; Hm_lvt_6933ef97905336bef84f9609785bcc3d=1701429444; Hm_lpvt_6933ef97905336bef84f9609785bcc3d=1701429531; ngacn0comUserInfoCheck=41020b6e203aacac6900db289217008c; ngacn0comInfoCheckTime=1701437416; lastvisit=1701437435; lastpath=/read.php?tid=20217469&_ff=428&page=478; bbsmisccookies=%7B%22uisetting%22%3A%7B0%3A1%2C1%3A1702034304%7D%2C%22pv_count_for_insad%22%3A%7B0%3A-20%2C1%3A1701450072%7D%2C%22insad_views%22%3A%7B0%3A1%2C1%3A1701450072%7D%7D")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 Edg/119.0.0.0")
        .send()
        .await?
        .text()
        .await?;
    let re = regex::Regex::new(r#"__PAGE = (.*?);"#).unwrap();
    let resp = re.captures(&resp).unwrap().get(1).unwrap().as_str();
    let resp = resp.replace(['{', '}'], "");
    let target = resp.split(',').collect::<Vec<_>>()[1];
    let pages_str = target.split_once(':').unwrap().1;
    let pages = pages_str.parse::<usize>().unwrap();

    Ok(pages)
}

fn check_valid(context: String) -> bool {
    let flag = check_img(&context);

    // 正则删除 图片链接
    let re = regex::Regex::new(r"\[img\].*?\[/img\]").unwrap();
    let context = re.replace_all(&context, "").to_string();

    let count = words_count::count(context);
    flag && count.words >= 10 || count.words >= 50
}

fn check_img(s: &str) -> bool {
    // 正则匹配是否有以图片链接
    let re = regex::Regex::new(r"\[img\].*\[/img\]").unwrap();
    re.is_match(s)
}
