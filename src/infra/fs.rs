use super::codec;
use color_eyre::Result;
use std::fs;
use std::path::Path;

/// Loads file content, attempting UTF-8 first, then falling back to auto-detection.
pub fn load_content(path: &Path) -> Result<Vec<String>> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            let bytes = fs::read(path)?;
            codec::decode_bytes(&bytes)
        }
    };

    // Split into lines immediately to form the core data structure
    Ok(content.lines().map(|s| s.to_string()).collect())
}

pub fn save_content(path: &Path, lines: &[String]) -> Result<()> {
    let content = lines.join("\n");
    fs::write(path, content)?;
    Ok(())
}
