use crate::chapter::Chapter;

pub const BOOKMARK_SYMBOL: &str = "🔖";

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub chapter_index: usize,
    pub line_in_chapter: usize,
    pub line_content: String,
}

pub fn parse_bookmarks(chapters: &[Chapter]) -> Vec<Bookmark> {
    let mut bookmarks = Vec::new();
    for (i, chapter) in chapters.iter().enumerate() {
        for (j, line) in chapter.content.iter().enumerate() {
            if line.trim().starts_with(BOOKMARK_SYMBOL) {
                let content = line
                    .trim()
                    .strip_prefix(BOOKMARK_SYMBOL)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                bookmarks.push(Bookmark {
                    chapter_index: i,
                    line_in_chapter: j,
                    line_content: content,
                });
            }
        }
    }
    bookmarks
}
