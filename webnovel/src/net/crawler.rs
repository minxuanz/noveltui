use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};

pub const SUPPORTED_DOMAIN: &str = "https://ixdzs8.com/read/";

pub struct NovelPage {
    pub title: String,
    pub content: Vec<String>,
}

/// URL 处理工具
pub struct UrlHandler;

impl UrlHandler {
    /// 检查 URL 是否受支持
    pub fn is_valid(url: &str) -> bool {
        !url.is_empty() && url.starts_with(SUPPORTED_DOMAIN)
    }

    /// 获取验证错误信息
    pub fn validation_error(url: &str) -> &'static str {
        if url.is_empty() {
            "URL cannot be empty."
        } else {
            "Invalid URL: Only https://ixdzs... supported."
        }
    }

    /// 更新章节号
    pub fn update_chapter(url: &str, delta: i32) -> String {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 5 {
            return url.to_string();
        }

        let last = parts.last().unwrap_or(&"");
        let chapter_num = last
            .strip_prefix('p')
            .and_then(|s| s.strip_suffix(".html"))
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        let next_chapter = (chapter_num + delta).max(1);
        format!(
            "https://{}/{}/{}/p{}.html",
            parts[2], parts[3], parts[4], next_chapter
        )
    }
}

pub fn fetch_novel(url: &str) -> Result<NovelPage> {
    let ops = LaunchOptions::default_builder().headless(true).build()?;

    let browser = Browser::new(ops)?;
    let tab = browser.new_tab()?;

    // Set a timeout for navigation
    tab.navigate_to(url)?;

    // Wait for the specific article container
    tab.wait_for_element("article.page-content")?;

    let html_content = tab.get_content()?;
    let document = Html::parse_document(&html_content);

    // <tilte> </title>
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|e| e.text().collect())
        .unwrap_or_else(|| "Untitled".to_string());
    // <p> </p>
    let p_selector = Selector::parse("article.page-content p").unwrap();

    let content = document
        .select(&p_selector)
        .map(|p| p.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    if content.is_empty() {
        return Err(anyhow::anyhow!("No content found at the provided URL."));
    }

    tab.close(true)?;

    Ok(NovelPage { title, content })
}
