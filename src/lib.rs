use std::{cmp::Ordering, fs, path::Path, str::from_utf8_unchecked};

use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct LenGroup {
    offset: u32,
    len: u16,
    count: u16,
}

pub struct SpellChecker {
    blob: &'static str,
    len_offsets: Vec<LenGroup>,
}

impl SpellChecker {
    pub fn new(file: impl AsRef<Path>) -> Self {
        // Very precise dataset is needed.
        // Words sorted by length, where each word is divided by \n and each group by \n\n.
        // Should also be sorted alphabeticaly.
        let (blob, offsets) = load_words_dict(file).unwrap();
        Self {
            blob,
            len_offsets: offsets,
        }
    }

    // pub fn add(&mut self, word: String) {
    //     self.words.insert(Box::leak(Box::new(word)));
    // }

    pub fn check(&self, word: &str) -> bool {
        self.find(word).is_some()
    }

    // pub fn batch_check(&self, words: &[&str]) -> 

    pub fn find(&self, word: &str) -> Option<(usize, usize)> {
        let lg = &self.len_offsets.get(word.len() - 1)?;
        if lg.count == 0 {
            return None;
        }
        let len = lg.len as usize;
        let count = lg.count as usize;
        let start = lg.offset as usize;
        let end = start + len * count;

        let word = word.as_bytes();
        let blob = &self.blob.as_bytes()[start..end];
        let offsets = Self::find_word_in_slice_binary_search(word, blob)?;
        Some((
            lg.offset as usize + offsets.0,
            lg.offset as usize + offsets.1,
        ))
    }

    fn find_word_in_slice_binary_search(word: &[u8], slice: &[u8]) -> Option<(usize, usize)> {
        let mut low = 0usize;
        let mut high = slice.len() / word.len();
        while low < high {
            let mid = low + ((high - low) / 2);
            let mid_off = mid * word.len();
            let candidate = &slice[mid_off..(mid_off + word.len())];
            match word.cmp(candidate) {
                Ordering::Equal => return Some((mid * word.len(), mid * word.len() + word.len())),
                Ordering::Less => high = mid,
                Ordering::Greater => low = mid + 1,
            }
        }
        None
    }

    /// Suggest words that are similar to the given word.
    ///
    /// # Args
    ///
    /// * `word`: The word to suggest similar words for.
    /// * `take_first_x`: The number of suggestions to take from the result.
    ///
    /// # Returns
    ///
    /// A vector of words that are similar to the given word. If `take_first_x` is 0,
    /// all suggestions are returned. Otherwise, only the first `take_first_x` suggestions
    /// are returned.
    pub fn suggest(&self, word: &str, take_first_x: usize) -> Vec<&'static str> {
        let word = word.to_lowercase();
        let word_len = word.len();

        if let Some(offset) = self.find(&word) {
            return vec![&self.blob[offset.0..offset.1]];
        }

        let min_len = word_len.saturating_sub(2).max(0);
        let max_len = (word_len + 2).min(self.len_offsets.len());
        
        let words = &self.len_offsets[min_len..max_len];

        let first_char = word.bytes().next();
        let last_char = word.bytes().last();

        let bcomp = rapidfuzz::distance::levenshtein::BatchComparator::new(word.chars());
        let mut result: Vec<(&'static str, usize)> = words
            .par_iter()
            .flat_map(|group| {
                let text = &self.blob[group.offset as usize
                    ..(group.offset+(group.len as u32*group.count as u32)) as usize];

                text
                    .as_bytes()
                    .par_chunks(group.len as usize)
                    .filter_map(|ch| {
                        if (group.len as i32 - word_len as i32).abs() == 2 {
                            if let (Some(fc), Some(lc)) = (first_char, last_char) {
                                if !ch.contains(&fc) && !ch.contains(&lc) {
                                    return None;
                                }
                            }
                        }

                        let word = unsafe{
                            from_utf8_unchecked(ch)
                        };

                        let dist = bcomp.distance(word.chars());
                        if dist <= 2 {
                            Some((word, dist))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if result.len() > 1 {
            result.par_sort_unstable_by_key(|(_, dist)| *dist);
        }

        if take_first_x == 0 {
            result.into_iter().map(|(word, _)| word).collect()
        } else {
            result.into_iter().take(take_first_x).map(|(word, _)| word).collect()
        }
    }

    /// Returns a vector of vectors of strings containing the suggestions for each of the given words.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    pub fn batch_suggest(&self, words: &[&str], take_first_x: usize) -> Vec<Vec<&'static str>> {
        words
            .iter()
            .map(|word| self.suggest(word, take_first_x))
            .collect()
    }

    /// Returns a vector of vectors of strings containing the suggestions for each of the given words.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    ///
    /// This is a parallelized version of batch_suggest.
    pub fn batch_par_suggest(&self, words: &[&str], take_first_x: usize) -> Vec<Vec<&'static str>> {
        words
            .par_iter()
            .map(|word| self.suggest(word, take_first_x))
            .collect()
    }
}

pub fn load_words_dict<T: AsRef<Path>>(
    file: T,
) -> Result<(&'static str, Vec<LenGroup>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file)?;
    
    // Pre-allocate: estimate based on content size
    let estimated_capacity = content.len();
    let mut blob = String::with_capacity(estimated_capacity);
    let mut offsets = Vec::with_capacity(64); // Most words are < 64 chars
    
    let mut current_offset = 0u32;
    let mut current_len = 0u16;
    let mut current_count = 0u16;
    let mut last_len = 0;
    
    for line in content.lines() {
        if line.is_empty() {
            // End of length group
            if current_count > 0 {
                offsets.push(LenGroup {
                    len: current_len,
                    count: current_count,
                    offset: current_offset,
                });
                current_offset += current_len as u32 * current_count as u32;
                current_count = 0;
            }
            continue;
        }
        
        let word_len = line.len() as u16;
        last_len = word_len as usize;
        
        if current_count == 0 {
            // Start new group
            current_len = word_len;
        }
        
        blob.push_str(line);
        current_count += 1;
    }
    
    // Don't forget the last group
    if current_count > 0 {
        offsets.push(LenGroup {
            len: current_len,
            count: current_count,
            offset: current_offset,
        });
    }
    
    // Fill in missing length entries
    let mut filled_offsets = Vec::with_capacity(last_len);
    let mut offset_idx = 0;
    
    for target_len in 1..=last_len {
        if offset_idx < offsets.len() && offsets[offset_idx].len == target_len as u16 {
            filled_offsets.push(offsets[offset_idx].clone());
            offset_idx += 1;
        } else {
            // Insert placeholder for missing length
            let prev_offset = if filled_offsets.is_empty() {
                0
            } else {
                let prev = &filled_offsets[filled_offsets.len() - 1];
                prev.offset + prev.len as u32 * prev.count as u32
            };
            filled_offsets.push(LenGroup {
                len: target_len as u16,
                count: 0,
                offset: prev_offset,
            });
        }
    }
    
    let blob_static: &'static str = Box::leak(blob.into_boxed_str());
    Ok((blob_static, filled_offsets))
}