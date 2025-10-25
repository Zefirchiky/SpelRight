use std::{path::Path, fs};
use crate::LenGroup;

/// Loads a words dictionary from a given file.
///
/// The file should be formatted as follows:
///
/// 1. Every other line, starting at `0`, should contain a `length` of `words` in the next line.
/// 2. Every other line, starting at `1`, should contain the `words` of the given `length`, concatenated together.
///
/// The function returns a vector of `LenGroup`, which contains the `blob of words` of the given `length`, the `length` of the words, and the `count` of words in the blob.
///
/// This function is io bound, and will take up to `4ms` on a low end hardware.
pub fn load_words_dict<T: AsRef<Path>>(
    file: T,
) -> Result<Vec<LenGroup>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file)?;    // About 2 ms

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(vec![]);
    }

    // Find max length from the last length line (every other line, starting at 0)
    let max_len = lines
        .iter()
        .step_by(2)
        .last()
        .and_then(|line| line.trim().parse::<u16>().ok())
        .unwrap_or(0);

    let mut group_map: Vec<Option<(String, u16)>> = vec![None; max_len as usize];

    for i in (0..lines.len()).step_by(2) {
        if let Ok(word_len) = lines[i].trim().parse::<u16>() {
            if word_len > 0 && (word_len as usize) <= max_len as usize {
                if let Some(blob_line) = lines.get(i + 1) {
                    let blob = blob_line.trim().to_string();
                    let count = (blob.len() / word_len as usize) as u16;
                    group_map[(word_len as usize) - 1] = Some((blob, count));
                }
            }
        }
    }

    let mut result = Vec::with_capacity(max_len as usize);
    for (idx, entry) in group_map.into_iter().enumerate() {
        let len = (idx + 1) as u16;
        let (blob, count) = entry.unwrap_or_else(|| (String::new(), 0));
        result.push(LenGroup { blob, len, count });
    }
    
    Ok(result)
}