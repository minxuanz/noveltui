use super::codec;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Loads file content, attempting UTF-8 first, then falling back to auto-detection.
pub fn load_content(path: &Path) -> Result<Vec<String>> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Early return if the file is strictly missing
            return Err(e.into());
        }
        Err(_) => {
            // Path exists, but it's likely not valid UTF-8
            let bytes = fs::read(path)?;
            codec::decode_bytes(&bytes)
        }
    };

    Ok(content.lines().map(|s| s.to_string()).collect())
}

pub fn save_content(path: &Path, lines: &[String]) -> Result<()> {
    let content = lines.join("\n");
    fs::write(path, content)?;
    Ok(())
}
