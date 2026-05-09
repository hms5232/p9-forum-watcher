//! 討論區列表

use super::Sort;
use crate::prune_link;
use chrono::{Local, NaiveDateTime};
use reqwest::Url;
use std::collections::HashMap;

/// 列表文章
#[derive(Clone, Debug)]
pub(crate) struct Post {
    /// 最新回應時間
    pub latest_reply_at: NaiveDateTime,
    /// 討論主題
    pub original_title: String,
    // 發表人
    pub op: String,
    /// 篇數，就是回應數
    pub reply_count: u32,
    /// 人氣
    pub views: u32,

    pub url: Url,
    pub pinned: bool,
    pub image_included: bool,

    fake: bool,
}

impl Post {
    /// # Parameter
    ///
    /// * table_content: crawled and split table content
    pub fn new(table_content: &HashMap<&str, String>, url: Url) -> Self {
        let original_title = table_content.get("original_title").unwrap();
        Self {
            latest_reply_at: NaiveDateTime::parse_from_str(
                table_content.get("time").unwrap(),
                "%Y/%m/%d %H:%M:%S",
            )
            .unwrap(),
            original_title: original_title.clone(),
            op: table_content.get("author").unwrap().clone(),
            reply_count: table_content
                .get("reply_count")
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            views: table_content.get("views").unwrap().parse::<u32>().unwrap(),
            url,
            pinned: original_title.contains("【頂】"),
            image_included: original_title.contains("[ 圖 ]"),
            fake: false,
        }
    }

    /// 檢查此文章是否在給定的另一篇文章之後（比較新）
    pub fn after(&self, other: &Self, order_by: Sort) -> bool {
        match order_by {
            Sort::LastReplyTime => self.latest_reply_at > other.latest_reply_at,
            Sort::PostTime => {
                panic!("目前沒有取得建立時間資料")
            }
        }
    }

    /// 取得修剪後的文章網址
    pub fn get_pruned_url(&self) -> String {
        prune_link(self.url.to_string()).unwrap()
    }

    /// 製做一個假的文章結構體，用來偷懶用
    pub(crate) fn fake_post() -> Self {
        Self {
            latest_reply_at: Local::now().naive_local(),
            original_title: "".to_string(),
            op: "".to_string(),
            reply_count: 0,
            views: 0,
            url: Url::parse(super::BASE_URL).unwrap(),
            pinned: false,
            image_included: false,
            fake: true,
        }
    }

    pub(crate) fn is_fake(&self) -> bool {
        self.fake
    }
}

impl PartialEq for Post {
    /// 檢查是否為同一篇文章
    fn eq(&self, other: &Self) -> bool {
        // 同一篇的話網址會相同
        self.url == other.url
    }
}
