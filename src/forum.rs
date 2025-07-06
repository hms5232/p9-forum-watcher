//! 討論區

/// 網站討論區 URL
const BASE_URL: &str = "https://www.p9.com.tw/Forum/ForumSection.aspx";

/// 取得指定看板以及指定排序方式的討論區 URL
pub fn get_url(section: SectionList, sort: Sort) -> String {
    format!(
        "{}?Id={}&Sort={}",
        BASE_URL,
        section.get_id(),
        sort.get_query_string()
    )
}

/// 討論區
pub enum SectionList {
    Whisky, // 威士忌
    Brandy, // 白蘭地
}

impl SectionList {
    /// 取得中文名稱
    pub fn zh_name(&self) -> &'static str {
        match self {
            SectionList::Whisky => "威士忌",
            SectionList::Brandy => "白蘭地",
        }
    }

    /// 取得討論區 id
    fn get_id(&self) -> u8 {
        match self {
            SectionList::Whisky => 1,
            SectionList::Brandy => 3,
        }
    }

    /// 取得討價還價分類 id
    fn get_auction_id(&self) -> u8 {
        match self {
            SectionList::Whisky => 5,
            SectionList::Brandy => 12,
        }
    }
}

/// 文章列表排序方式
pub enum Sort {
    LastReplyTime, // 最後回應時間
    PostTime,      // 發文時間
}

impl Sort {
    /// 取查指定排序方式對應的查詢字串值
    pub fn get_query_string(&self) -> &'static str {
        match self {
            Sort::LastReplyTime => "Last_Reply_Time",
            Sort::PostTime => "Post_Time",
        }
    }
}
