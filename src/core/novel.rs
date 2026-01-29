use super::pagination::ChapterMetadata;

pub const BOOKMARK_SYMBOL: &str = "🔖";

/// Represents a single bookmark entry.
#[derive(Debug, Clone)]
pub struct BookmarkEntry<'a> {
    /// Global line index in the file
    pub global_index: usize,
    /// The index of the chapter this line belongs to
    pub chapter_index: usize,
    /// The line index relative to the chapter start
    pub line_in_chapter: usize,
    /// The text content
    pub content: &'a str,
    // (removed) title of chapter is read from `Novel.chapters` via `chapter_index`
}

/// The Domain Model holding all data.
/// Optimized to avoid string duplication.
pub struct Novel {
    /// The Single Source of Truth for text content.
    pub lines: Vec<String>,
    /// Metadata determining view ranges.
    pub chapters: Vec<ChapterMetadata>,
}

impl Novel {
    pub fn new(lines: Vec<String>) -> Self {
        let chapters = ChapterMetadata::parse_chapters(&lines);
        Self { lines, chapters }
    }

    /// Returns a slice of lines corresponding to a specific chapter.
    /// Zero-copy: returns references to the main vector.
    pub fn get_chapter_lines(&self, chapter_idx: usize) -> Option<&[String]> {
        self.chapters
            .get(chapter_idx)
            .and_then(|meta| self.lines.get(meta.range.clone()))
    }

    /// Scans the entire file to rebuild the list of bookmarks.
    pub fn collect_bookmarks<'a>(&'a self) -> Vec<BookmarkEntry<'a>> {
        let mut bookmarks = Vec::new();
        let symbol_len = BOOKMARK_SYMBOL.len();

        for (c_idx, chapter) in self.chapters.iter().enumerate() {
            // Iterate only within chapter ranges
            let slice = &self.lines[chapter.range.clone()];
            for (local_idx, line) in slice.iter().enumerate() {
                let trimmed = line.trim_end();
                if trimmed.ends_with(BOOKMARK_SYMBOL) {
                    let content_end = trimmed.len().saturating_sub(symbol_len);
                    let clean_content = trimmed[..content_end].trim();

                    if !clean_content.is_empty() {
                        bookmarks.push(BookmarkEntry {
                            global_index: chapter.range.start + local_idx,
                            chapter_index: c_idx,
                            line_in_chapter: local_idx,
                            content: clean_content,
                        });
                    }
                }
            }
        }
        bookmarks
    }

    /// Toggles the bookmark symbol on a specific line.
    /// Returns true if changed.
    pub fn toggle_bookmark(&mut self, global_index: usize, remove_bookmark: bool) -> bool {
        if let Some(line) = self.lines.get_mut(global_index) {
            let trimmed_len = line.trim_end().len();
            // if line is empty return false
            if line.trim().is_empty() {
                return false;
            }

            if line.trim_end().ends_with(BOOKMARK_SYMBOL) && remove_bookmark {
                // Remove it
                let without_symbol = trimmed_len.saturating_sub(BOOKMARK_SYMBOL.len());
                line.truncate(without_symbol);
                // Clean trailing spaces
                let new_len = line.trim_end().len();
                line.truncate(new_len);
            } else {
                // Add it
                if line.trim_end().ends_with(BOOKMARK_SYMBOL) {
                    // Already has bookmark
                    return false;
                }
                line.truncate(trimmed_len);
                line.push(' ');
                line.push_str(BOOKMARK_SYMBOL);
            }
            return true;
        }
        false
    }

    pub fn remove_bookmark(&mut self, global_index: usize) {
        if let Some(line) = self.lines.get_mut(global_index) {
            let trimmed = line.trim_end();
            if trimmed.ends_with(BOOKMARK_SYMBOL) {
                let without_symbol = trimmed.len().saturating_sub(BOOKMARK_SYMBOL.len());
                line.truncate(without_symbol);
                let new_len = line.trim_end().len();
                line.truncate(new_len);
            }
        }
    }
}
