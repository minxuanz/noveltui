// src/core/pagination.rs
use crate::cmd::args::Options;
use clap::Parser;
use regex::Regex;
use std::ops::Range;
use std::sync::OnceLock;

static RE_CN: OnceLock<Regex> = OnceLock::new();
static RE_EN: OnceLock<Regex> = OnceLock::new();

/// Metadata defining a chapter's position in the global line vector.
#[derive(Debug, Clone)]
pub struct ChapterMetadata {
    pub title: String,
    /// The range of indices in the global `lines` vector [start, end)
    pub range: Range<usize>,
}

impl ChapterMetadata {
    /// Scans the raw lines and identifies chapter boundaries.
    pub fn parse_chapters(lines: &[String]) -> Vec<ChapterMetadata> {
        let default_re_cn = RE_CN.get_or_init(|| {
            Regex::new(r"^\s*第\s*[0-9零一二三四五六七八九十百千两〇]+\s*[章|话]").unwrap()
        });
        let default_re_en =
            RE_EN.get_or_init(|| Regex::new(r"(?i)^\s*chapter\s*(?:\d+|[a-z]+)").unwrap());

        let args: Options = Options::parse();
        let re_cn = if let Some(re_str) = args.regex.as_ref() {
            Regex::new(re_str).unwrap_or_else(|_| default_re_cn.clone())
        } else {
            default_re_cn.clone()
        };

        let re_en = if let Some(re_str) = args.regex.as_ref() {
            Regex::new(re_str).unwrap_or_else(|_| default_re_en.clone())
        } else {
            default_re_en.clone()
        };

        let mut chapters = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let is_header = if line.len() > 100 {
                false
            } else {
                re_cn.is_match(line) || re_en.is_match(line)
            };

            if is_header {
                // 如果这是发现的第一个标题，且前面有文字，将前面的内容归为 "Intro"
                if chapters.is_empty() && i > 0 {
                    chapters.push(ChapterMetadata {
                        title: "Intro".to_string(),
                        range: 0..i,
                    });
                } else if !chapters.is_empty() {
                    // 关闭上一个章节：将上一个章节的结束索引设置为当前行索引
                    if let Some(prev) = chapters.last_mut() {
                        prev.range.end = i;
                    }
                }

                // 创建新章节：起始索引为 i，结束索引暂时也设为 i（会在下个标题或循环结束时更新）
                chapters.push(ChapterMetadata {
                    title: line.trim().to_string(),
                    range: i..i,
                });
            }
        }

        // 循环结束后，关闭最后一个章节
        if let Some(last) = chapters.last_mut() {
            last.range.end = lines.len();
        } else if !lines.is_empty() {
            // 如果全文没有匹配到任何标题，则将全文视作一个章节
            chapters.push(ChapterMetadata {
                title: "Full Content".to_string(),
                range: 0..lines.len(),
            });
        }

        chapters
    }
}
