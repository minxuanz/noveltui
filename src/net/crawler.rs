use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};

pub struct NovelPage {
    pub title: String,
    pub content: Vec<String>,
}

pub fn fetch_novel(url: &str) -> Result<NovelPage, Box<dyn std::error::Error>> {
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
    let content: Vec<String> = document
        .select(&p_selector)
        .map(|p| p.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if content.is_empty() {
        return Err("No content found in article container".into());
    }

    Ok(NovelPage { title, content })
}
