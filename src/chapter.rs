use regex::Regex;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub number: usize,
    pub title: String,
    pub start_line: usize,
    pub content: Vec<String>,
}

pub fn parse_lines(lines: &[String]) -> Vec<Chapter> {
    let re_cn = Regex::new(r"^\s*第\s*(\d+)\s*章(?:\s*(.*))?$").unwrap();
    let re_en = Regex::new(r"(?i)^\s*chapter\s*(\d+)\s*[:\.\s-]*\s*(.*)?$").unwrap();
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current: Option<Chapter> = None;

    for (i, line) in lines.iter().enumerate() {
        let caps = re_cn.captures(line).or_else(|| re_en.captures(line));
        if let Some(caps) = caps {
            // push previous
            if let Some(prev) = current.take() {
                chapters.push(prev);
            }
            let num = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()).unwrap_or(0);
            let rest = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let is_cn = re_cn.is_match(line);
            let title = if rest.is_empty() {
                if is_cn {
                    format!("第{}章", num)
                } else {
                    format!("Chapter {}", num)
                }
            } else {
                if is_cn {
                    format!("第{}章 {}", num, rest)
                } else {
                    format!("Chapter {} {}", num, rest)
                }
            };
            current = Some(Chapter {
                number: num,
                title,
                start_line: i,
                content: vec![line.clone()],
            });
        } else {
            if let Some(ref mut ch) = current {
                ch.content.push(line.clone());
            }
        }
    }

    if let Some(last) = current {
        chapters.push(last);
    }

    chapters
}