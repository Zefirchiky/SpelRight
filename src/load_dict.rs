use std::{path::Path, fs};
use crate::LenGroup;

/// Loads a words dictionary from a file into a static string and a vector of length groups.
///
/// The file should contain a dataset of words, sorted by their byte length, where each word is divided by \n
/// and each group by \n\n. The dataset should also be sorted alphabetically.
///
/// Returns a static reference to the loaded blob and a vector of length groups.
///
/// Each length group contains the length of the words in that group, the count of the words in that group,
/// and the offset of the first word of that group in the blob.
///
/// The length groups are filled in so that every possible word length from 1 to the maximum length
/// in the dataset has a corresponding length group. If a word length is missing from the dataset, a placeholder
/// length group is inserted with a count of 0.
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if the file is not in the correct format.
pub fn load_words_dict<T: AsRef<Path>>(
    file: T,
) -> Result<Vec<LenGroup>, Box<dyn std::error::Error>> {    // TODO: Still pretty slow, may be can be improved.
    // About 2 ms
    let content = fs::read_to_string(file)?;
    
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