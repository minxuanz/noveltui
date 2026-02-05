use crate::cmd::args::Options;
use regex::Regex;
use std::ops::Range;
use std::sync::OnceLock;

static RE_CN: OnceLock<Regex> = OnceLock::new();
static RE_EN: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct ChapterMetadata {
    pub title: Box<str>,
    pub range: Range<usize>,
}

impl ChapterMetadata {
    pub fn parse_chapters(lines: &[String], args: &Options) -> Vec<Self> {
        // 1. Get regex reference, prioritize user-defined
        let re_cn_owned = args.regex.as_ref().and_then(|s| Regex::new(s).ok());
        let re_cn: &Regex = match re_cn_owned.as_ref() {
            Some(r) => r,
            None => RE_CN.get_or_init(|| {
                Regex::new(r"^\s*第\s*[0-9零一二三四五六七八九十百千两〇]+\s*(章|话)").unwrap()
            }),
        };

        let re_en_owned = args.regex.as_ref().and_then(|s| Regex::new(s).ok());
        let re_en: &Regex = match re_en_owned.as_ref() {
            Some(r) => r,
            None => RE_EN.get_or_init(|| Regex::new(r"(?i)^\s*chapter\s*(?:\d+|[a-z]+)").unwrap()),
        };

        let mut chapters = Vec::new();
        let mut last_idx = 0;

        for (i, line) in lines.iter().enumerate() {
            // Length check optimization: prioritize excluding long lines to reduce regex overhead
            if line.len() <= 100 && (re_cn.is_match(line) || re_en.is_match(line)) {
                if i > last_idx {
                    let title = if chapters.is_empty() {
                        "Intro".into()
                    } else {
                        let real_title = lines[last_idx].trim().into();
                        last_idx = last_idx.saturating_add(1);
                        real_title
                    };

                    chapters.push(ChapterMetadata {
                        title,
                        range: last_idx..i,
                    });
                }
                last_idx = i;
            }
        }

        // Handle the last chapter or full text
        if !lines.is_empty() {
            let title = if chapters.is_empty() {
                "Full Content".into()
            } else {
                let real_title = lines[last_idx].trim().into();
                last_idx = last_idx.saturating_add(1);
                real_title
            };

            chapters.push(ChapterMetadata {
                title,
                range: last_idx..lines.len(),
            });
        }

        chapters
    }
}

#[test]
fn test_chapter_parsing() {
    use crate::cmd::args::Options;

    let args = Options::default();
    let lines = vec![
        "Some intro text.".to_string(),
        "Chapter 1: The Beginning".to_string(),
        "This is the first chapter.".to_string(),
        "Chapter 2: The Continuation".to_string(),
        "This is the second chapter.".to_string(),
    ];

    let chapters = ChapterMetadata::parse_chapters(&lines, &args);
    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].title.as_ref(), "Intro");
    assert_eq!(chapters[0].range, 0..1);
    assert_eq!(chapters[1].title.as_ref(), "Chapter 1: The Beginning");
    assert_eq!(chapters[1].range, 2..3);
    assert_eq!(chapters[2].title.as_ref(), "Chapter 2: The Continuation");
    assert_eq!(chapters[2].range, 4..5);
}
