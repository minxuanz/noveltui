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
        // 1. 获取正则引用，优先使用用户定义的
        let re_cn_owned = args.regex.as_ref()
            .and_then(|s| Regex::new(s).ok());
        let re_cn: &Regex = match re_cn_owned.as_ref() {
            Some(r) => r,
            None => RE_CN.get_or_init(|| {
                Regex::new(r"^\s*第\s*[0-9零一二三四五六七八九十百千两〇]+\s*(章|话)").unwrap()
            }),
        }; 

        let re_en_owned = args.regex.as_ref()
            .and_then(|s| Regex::new(s).ok());
        let re_en: &Regex = match re_en_owned.as_ref() {
            Some(r) => r,
            None => RE_EN.get_or_init(|| {
                Regex::new(r"(?i)^\s*chapter\s*(?:\d+|[a-z]+)").unwrap()
            }),
        };

        let mut chapters = Vec::new();
        let mut last_idx = 0;

        for (i, line) in lines.iter().enumerate() {
            // 长度检查优化：优先排除长行，减少正则开销
            if line.len() <= 100 && (re_cn.is_match(line) || re_en.is_match(line)) {
                
                if i > last_idx {
                    let title = if chapters.is_empty() {
                        last_idx = last_idx - 1;
                        "Intro".into()
                    } else {
                        lines[last_idx].trim().into()
                    };
                    
                    chapters.push(ChapterMetadata {
                        title,
                        range: (last_idx + 1)..i,
                    });
                }
                last_idx = i;
            }
        }

        // 处理最后一个章节或全文
        if !lines.is_empty() {
            let title = if chapters.is_empty() {
                last_idx = last_idx - 1;
                "Full Content".into()
            } else {
                lines[last_idx].trim().into()
            };
            
            chapters.push(ChapterMetadata {
                title,
                range: (last_idx + 1)..lines.len(),
            });
        }

        chapters
    }
}