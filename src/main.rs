use chrono::prelude::*;
use reqwest::blocking::Client;
use reqwest::{Url, header};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::thread::sleep;

const HEADER_ROW: [&str; 8] = [
    "time",
    "original_title",
    "author",
    "reply_count",
    "views",
    "title1",
    "title2",
    "link",
];

fn main() {
    let mut check_point = Local::now(); // 確認新文章的時間點，程式剛啟動就是當下。注意：未考量執行機器非台北時區的情況
    loop {
        let mut posts: Vec<HashMap<&str, String>> = Vec::new();

        let url = Url::parse(
            "https://www.p9.com.tw/Forum/ForumSection.aspx?id=3&BoardId=12&Sort=Post_Time",
        )
        .unwrap();
        let client = Client::new();
        let res = client.get(url.as_str())
            .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .send()
            .unwrap();
        println!("[{}] 請求完成，檢查點 {}", Local::now().format("%Y/%m/%d %H:%M:%S"), check_point.format("%Y/%m/%d %H:%M:%S"));
        let document = Html::parse_document(&res.text().unwrap());
        let tbody_selector = Selector::parse(".contentMain > table:nth-child(1) > tbody:nth-child(1) > tr:nth-child(6) > td:nth-child(1) > table:nth-child(1) > tbody:nth-child(1)").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        // 選擇 <tbody> 並遍歷每個 <tr>
        if let Some(tbody) = document.select(&tbody_selector).next() {
            for tr in tbody.select(&tr_selector).skip(1) {
                // 抓出每個 <td> 作為原始數值
                let values: Vec<String> = tr
                    .select(&Selector::parse("td").unwrap())
                    .map(|td| {
                        td.text()
                            .collect::<String>()
                            .replace("\n", "")
                            .trim()
                            .to_string()
                    })
                    .collect();
                let mut row = HashMap::new();
                for (i, value) in values.iter().enumerate() {
                    row.insert(HEADER_ROW[i], value.to_string());
                }

                // 如果 check_point 比文章時間晚，代表是舊文章
                if NaiveDateTime::parse_from_str(&row["time"], "%Y/%m/%d %H:%M:%S").unwrap()
                    < check_point.naive_local()
                {
                    continue;
                }

                // 找出連結
                for title_td in tr
                    .select(&Selector::parse("td.pricelist_02[align=\"left\"] > div > a").unwrap())
                    .into_iter()
                {
                    // 實際上只會有一個，所以直接 insert
                    row.insert(
                        "link",
                        url.join(title_td.attr("href").unwrap())
                            .unwrap()
                            .to_string(),
                    );
                }

                posts.push(row);
            }
        }

        for post in posts {
            println!("新文章〈{}〉{}", post["original_title"], post["link"]);
        }

        check_point = Local::now();
        sleep(std::time::Duration::from_secs(60 * 10));
    }
}
