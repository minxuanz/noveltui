use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub number: usize,
    pub title: String,
    pub start_line: usize,
    pub content: Vec<String>,
}

// 1. 使用 OnceLock 全局缓存正则
static RE_CN: OnceLock<Regex> = OnceLock::new();
static RE_EN: OnceLock<Regex> = OnceLock::new();

pub fn parse_lines(lines: &[String]) -> Vec<Chapter> {
    let re_cn = RE_CN.get_or_init(|| {
        // [0-9] 匹配阿拉伯数字
        // [零一二三四五六七八九十百千两〇] 匹配常见中文数字
        Regex::new(r"^\s*第\s*[0-9零一二三四五六七八九十百千两〇]+\s*章").unwrap()
    });
    let re_en = RE_EN.get_or_init(|| Regex::new(r"(?i)^\s*chapter\s*\d+").unwrap());

    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current: Option<Chapter> = None;
    let mut intro_lines: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {

        let is_potential_header = if line.len() > 100 {
            false
        } else {
            let trimmed = line.trim_start();
            trimmed.starts_with('第')
                || trimmed.starts_with("Chapter")
                || trimmed.starts_with("CHAPTER")
                || trimmed.starts_with("chapter")
        };

        let matched_cn = is_potential_header && re_cn.is_match(line);
        let matched_en = !matched_cn && is_potential_header && re_en.is_match(line);

        if matched_cn || matched_en {
            // A. 处理 Intro
            if current.is_none() && !intro_lines.is_empty() {
                let intro_title = if matched_cn { "简介" } else { "Intro" };

                // 使用 mem::take 直接拿走 intro_lines 的内容，避免 clone
                let content = std::mem::take(&mut intro_lines);

                current = Some(Chapter {
                    number: 0,
                    title: intro_title.to_string(),
                    start_line: 0,
                    content,
                });
            }

            // B. 归档上一章
            if let Some(prev) = current.take() {
                chapters.push(prev);
            }

            // C. 创建新章节
            // 优化：直接读取标题行，不再解析捕获组
            let title = line.trim().to_string();
            let number = chapters.len() + 1; // 简单的计数逻辑，如果 Intro 存在则 Intro 是 0

            current = Some(Chapter {
                number,
                title,
                start_line: i,
                content: vec![line.clone()],
            });
        } else {
            let target = current
                .as_mut()
                .map(|c| &mut c.content)
                .unwrap_or(&mut intro_lines);
            target.push(line.clone());
        }
    }

    // Push last chapter
    if let Some(last) = current {
        chapters.push(last);
    } else if !intro_lines.is_empty() {
        chapters.push(Chapter {
            number: 0,
            title: "ALL".to_string(),
            start_line: 0,
            content: intro_lines,
        });
    }

    chapters
}
