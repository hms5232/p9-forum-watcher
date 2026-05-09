use crate::forum::Sort;
use crate::forum::list::Post;
use chrono::prelude::*;
use notify_rust::{Notification, Timeout};
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

const POST_BASE_URL: &str = "https://www.p9.com.tw/Forum/";

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
    let sort = Sort::get_by_zh_name(
        Listbox::new(Sort::iter())
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

    let target_url = forum::get_url(&section_variant, &sort_variant);
    println!("目標網址：{}", target_url);
    let url = Url::parse(target_url.as_str()).unwrap();
    let mut check_point = Post::fake_post(); // 本次檢查點，可能是新建立或是上次檢查的第一篇文章

    loop {
        let mut posts: Vec<Post> = Vec::new();

        let client = Client::new();
        let res = client
            .get(url.as_str())
            .header(
                header::USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:150.0) Gecko/20100101 Firefox/150.0",
            )
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

                let post = Post::new(&row, post_link);

                // 如果是初次啟動，後面就不用再檢查了
                if check_point.is_fake() {
                    check_point = post;
                    break;
                }

                match &sort_variant {
                    Sort::LastReplyTime => {
                        // 如果看到目前文章最後回覆時間早於檢查點的最後回覆時間，表示已經檢查完畢
                        if post.latest_reply_at.le(&check_point.latest_reply_at) {
                            break;
                        }
                    }
                    Sort::PostTime => {
                        // 如果重新看到上一次的最後一篇文章（檢查點），代表已經完成所有新文章的檢查
                        if post.eq(&check_point) {
                            break;
                        }
                    }
                }

                posts.push(post);
            }
        }
        // 初次啟動或沒有新的文章，posts 就會沒東西
        // 此情況在前面就已經賦值給檢查點變數了，所以不需要再覆蓋
        if !posts.is_empty() {
            // 向量中第一個 Post 就是第一篇文章
            check_point = posts.first().unwrap().clone();
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
            // insert row
            table_rows.push(row![
                TableCell::builder(post.original_title.as_str()).build(),
                TableCell::builder(post.get_pruned_url()).build(),
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
        #[cfg(debug_assertions)]
        {
            // debug 即便 0 也印出表格
            // 反之，不是 0 為了避免重複印出則不印
            if count == 0 {
                println!("{}", table.render());
            }
        }
        // 有新的動態
        if count > 0 {
            println!("{}", table.render());
            // 系統通知
            Notification::new()
                .summary("P9 論壇監視器")
                .body(&format!("發現 {} 篇新文章或有更新", count))
                .timeout(Timeout::Milliseconds(10000))
                .show()
                .unwrap();
        }

        // 檢查間隔
        let interval = 60 * if cfg!(debug_assertions) { 1 } else { 10 };
        sleep(std::time::Duration::from_secs(interval));
    }
}

/// 修剪貼文網址
///
/// 網址中的中文其實是沒必要的，修剪後就能搭配表格形式輸出了
fn prune_link(url: String) -> Option<String> {
    let re = regex::Regex::new(r"(Topics\d+)").unwrap();
    if let Some(capture) = re.captures(&url) {
        if let Some(post_id) = capture.get(1) {
            return Some(format!("{}{}.aspx", POST_BASE_URL, post_id.as_str()));
        }
    }
    None
}
