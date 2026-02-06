use ratatui::style::Color;
use std::str::FromStr;

/// 主题配色方案
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    /// TOC（目录）高亮色
    pub toc: Color,
    /// Content（正文）高亮色
    pub content: Color,
    /// Bookmark（书签）高亮色
    pub bookmark: Color,
    /// Highlight（选中高亮）色
    pub highlight: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        // 默认使用你当前的硬编码配色
        Self {
            toc: Color::Rgb(129, 199, 212),       // #81C7D4 - 淡青色
            content: Color::Rgb(168, 216, 185),   // #A8D8B9 - 淡绿色
            bookmark: Color::Rgb(248, 195, 205),  // #F8C3CD - 淡粉色
            highlight: Color::Rgb(150, 206, 193), // #96CEC1 - 青绿色
        }
    }
}

/// 预设主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    /// 默认主题（你的原始配色）
    #[default]
    Default,
    /// 海洋主题（蓝色系）
    Ocean,
    /// 森林主题（绿色系）
    Forest,
    /// 日落主题（橙红色系）
    Sunset,
    /// 午夜主题（深色系）
    Midnight,
    /// 樱花主题（粉色系）
    Sakura,
}

impl ThemePreset {
    /// 获取主题配色
    pub fn colors(&self) -> ThemeColors {
        match self {
            ThemePreset::Default => ThemeColors::default(),
            ThemePreset::Ocean => ThemeColors {
                toc: Color::Rgb(100, 181, 246),       // #64B5F6
                content: Color::Rgb(79, 195, 247),    // #4FC3F7
                bookmark: Color::Rgb(144, 202, 249),  // #90CAF9
                highlight: Color::Rgb(129, 212, 250), // #81D4FA
            },
            ThemePreset::Forest => ThemeColors {
                toc: Color::Rgb(129, 199, 132),      // #81C784
                content: Color::Rgb(165, 214, 167),  // #A5D6A7
                bookmark: Color::Rgb(102, 187, 106), // #66BB6A
                highlight: Color::Rgb(139, 195, 74), // #8BC34A
            },
            ThemePreset::Sunset => ThemeColors {
                toc: Color::Rgb(255, 183, 77),      // #FFB74D
                content: Color::Rgb(255, 138, 101), // #FF8A65
                bookmark: Color::Rgb(255, 112, 67), // #FF7043
                highlight: Color::Rgb(255, 152, 0), // #FF9800
            },
            ThemePreset::Midnight => ThemeColors {
                toc: Color::Rgb(144, 202, 249),       // #90CAF9
                content: Color::Rgb(179, 136, 255),   // #B388FF
                bookmark: Color::Rgb(255, 138, 128),  // #FF8A80
                highlight: Color::Rgb(128, 203, 196), // #80CBC4
            },
            ThemePreset::Sakura => ThemeColors {
                toc: Color::Rgb(244, 143, 177),       // #F48FB1
                content: Color::Rgb(248, 187, 208),   // #F8BBD0
                bookmark: Color::Rgb(236, 64, 122),   // #EC407A
                highlight: Color::Rgb(255, 128, 171), // #FF80AB
            },
        }
    }
}

impl FromStr for ThemePreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(ThemePreset::Default),
            "ocean" => Ok(ThemePreset::Ocean),
            "forest" => Ok(ThemePreset::Forest),
            "sunset" => Ok(ThemePreset::Sunset),
            "midnight" => Ok(ThemePreset::Midnight),
            "sakura" => Ok(ThemePreset::Sakura),
            _ => Err(format!(
                "Unknown theme: {}. Available themes: default, ocean, forest, sunset, midnight, sakura",
                s
            )),
        }
    }
}

impl std::fmt::Display for ThemePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemePreset::Default => write!(f, "default"),
            ThemePreset::Ocean => write!(f, "ocean"),
            ThemePreset::Forest => write!(f, "forest"),
            ThemePreset::Sunset => write!(f, "sunset"),
            ThemePreset::Midnight => write!(f, "midnight"),
            ThemePreset::Sakura => write!(f, "sakura"),
        }
    }
}
