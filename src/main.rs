use chrono::prelude::*;
use promkit::preset::listbox::Listbox;
use reqwest::blocking::Client;
use reqwest::{Url, header};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::thread::sleep;
use strum::IntoEnumIterator;
use term_table::row::Row;
use term_table::table_cell::TableCell;
use term_table::{Table, TableStyle, row};

mod forum;

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

/// 訊息前方加上時間的 println!
///
/// # Example
///
/// ```
/// println_with_time!("Hello {}!", "world"); // console output: "[2025/01/01 00:00:00] Hello world!"
/// ```
///
macro_rules! println_with_time {
    ($($arg:tt)*) => {
        println!("[{}] {}", Local::now().format("%Y/%m/%d %H:%M:%S"), format_args!($($arg)*));
    };
}

fn main() {
    // 詢問選單，讓使用者選擇看板和排序依據
    let section = forum::SectionList::get_by_zh_name(
        Listbox::new(forum::SectionList::iter())
            .title("請選擇看板")
            .prompt()
            .unwrap()
            .run()
            .unwrap(),
    );
    let Some(section_variant) = section else {
        eprint!("看板列舉映射發生問題，請回報給開發人員");
        return;
    };
    let sort = forum::Sort::get_by_zh_name(
        Listbox::new(forum::Sort::iter())
            .title("請選擇排序依據")
            .prompt()
            .unwrap()
            .run()
            .unwrap(),
    );
    let Some(sort_variant) = sort else {
        eprint!("排序列舉映射發生問題，請回報給開發人員");
        return;
    };

    let target_url = forum::get_url(section_variant, sort_variant);
    println!("目標網址：{}", target_url);
    let url = Url::parse(target_url.as_str()).unwrap();
    let mut check_point = url.clone(); // 本次檢查點，可能是新建立或是上次檢查的第一篇文章

    loop {
        let mut posts: Vec<HashMap<&str, String>> = Vec::new();
        let mut next_check_point = url.clone(); // 下次檢查點，也就是這次檢查的第一篇文章

        let client = Client::new();
        let res = client.get(url.as_str())
            .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .send()
            .unwrap();
        println_with_time!("請求完成");
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
                // 如果讀取到的值不足 5 個，代表文章有問題或不是文章
                if values.len() < 5 {
                    continue;
                }
                let mut row = HashMap::new();
                for (i, value) in values.iter().enumerate() {
                    row.insert(HEADER_ROW[i], value.to_string());
                }

                // 跳過置頂
                if row.get("original_title").unwrap().contains("【頂】") {
                    continue;
                }

                // 找出連結
                let mut post_link = url.clone(); // 借用目標頁的 Url 作為初始值
                for title_td in tr
                    .select(&Selector::parse("td.pricelist_02[align=\"left\"] > div > a").unwrap())
                    .into_iter()
                {
                    post_link = url.join(title_td.attr("href").unwrap()).unwrap();
                }

                // 如果是初次啟動，後面就不用再檢查了
                if check_point.eq(&url) {
                    check_point = post_link.clone();
                    break;
                }
                // 第一篇文章就是下次的起始檢查點
                if next_check_point.eq(&url) {
                    next_check_point = post_link.clone();
                }
                // 如果重新看到 check_point，代表已經完成所有新文章的檢查
                if post_link.eq(&check_point) {
                    check_point = next_check_point;
                    break;
                }

                row.insert("link", post_link.to_string());

                posts.push(row);
            }
        }

        // 印出新文章
        let count = posts.len();
        let mut table_rows: Vec<Row> = vec![
            // 表格標題：時間
            row![
                TableCell::builder(Local::now().format("%Y/%m/%d %H:%M:%S"))
                    .col_span(2)
                    .alignment(term_table::table_cell::Alignment::Center)
                    .build(),
            ],
            // 表格標頭
            row![
                TableCell::builder("原始標題")
                    .alignment(term_table::table_cell::Alignment::Center)
                    .build(),
                TableCell::builder("連結")
                    .alignment(term_table::table_cell::Alignment::Center)
                    .build(),
            ],
        ];
        for post in posts {
            table_rows.push(row![
                TableCell::builder(post["original_title"].as_str()).build(),
                TableCell::builder(post["link"].as_str()).build(),
            ])
        }
        table_rows.push(
            // 表格結尾統計
            row![
                TableCell::builder(
                    Local::now().format(format!("本次計有 {} 篇新文章", count).as_str())
                )
                .col_span(2)
                .alignment(term_table::table_cell::Alignment::Center)
                .build(),
            ],
        );
        let table = Table::builder()
            .max_column_width(80)
            .style(TableStyle::extended())
            .rows(table_rows)
            .build();
        println!("{}", table.render());

        sleep(std::time::Duration::from_secs(60 * 10));
    }
}
