use crate::chapter::Chapter;

// 保持常量定义
pub const BOOKMARK_SYMBOL: &str = "🔖";

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub chapter_index: usize,
    pub line_in_chapter: usize,
    pub line_content: String,
}

pub fn parse_bookmarks(chapters: &[Chapter]) -> Vec<Bookmark> {
    let symbol_len = BOOKMARK_SYMBOL.len();
    let mut bookmarks = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        for (j, line) in chapter.content.iter().enumerate() {
            let trimmed_line = line.trim_end();

            if trimmed_line.ends_with(BOOKMARK_SYMBOL) {
                let content_end = trimmed_line.len() - symbol_len;
                let content_slice = trimmed_line[..content_end].trim(); // 去除内容末尾可能存在的空格

                // 优化 3: 在分配内存前检查是否为空 (Zero Allocation check)
                if !content_slice.is_empty() {
                    bookmarks.push(Bookmark {
                        chapter_index: i,
                        line_in_chapter: j,
                        line_content: content_slice.to_string(),
                    });
                }
            }
        }
    }
    bookmarks
}

pub fn toggle_bookmark_symbol_in_place(line: &mut String) {
    let symbol = BOOKMARK_SYMBOL;

    // 注意：trim_end() 返回的是 slice，不会分配内存
    if line.trim_end().ends_with(symbol) {
        remove_bookmark_symbol_from_line(line);
    } else {
        // 添加书签
        // 1. 去除当前行尾的空格（直接修改长度）
        let trimmed_len = line.trim_end().len();
        line.truncate(trimmed_len);

        // 2. 追加 " " 和 符号
        // push_str 通常会利用现有的 capacity，往往不需要重新分配内存
        line.push(' ');
        line.push_str(symbol);
    }
}

pub fn remove_bookmark_symbol_from_line(line: &mut String) {
    let symbol = BOOKMARK_SYMBOL;
    let trimmed = line.trim_end();

    if trimmed.ends_with(symbol) {
        let length_without_symbol = trimmed.len().saturating_sub(symbol.len());

        // 第一步截断：去掉符号和符号后的空格（如果有）
        line.truncate(length_without_symbol);

        let clean_len = line.trim_end().len();
        line.truncate(clean_len);
    }
}
