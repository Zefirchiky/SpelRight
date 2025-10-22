use std::{cmp::Ordering, fs, path::Path, str::{from_utf8_unchecked}, time::{Duration, Instant}};

use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct LenGroup {
    offset: u32,
    len: u16,
    count: u16,
}

pub struct SpellChecker {
    blob: String,
    len_offsets: Vec<LenGroup>,
    max_dif: usize,
}

pub enum ComparatorWrapper<'a> {
    Levenstein(&'a rapidfuzz::distance::levenshtein::BatchComparator<&'a u8>),
    Indel(&'a rapidfuzz::distance::indel::BatchComparator<&'a u8>),
    Hamming(&'a rapidfuzz::distance::hamming::BatchComparator<&'a u8>)
    // O(&'a rapidfuzz::distance::)
}

impl<'a> ComparatorWrapper<'a> {
    pub fn distance(&self, word: &[u8]) -> usize {
        match self {
            Self::Levenstein(bcomp) => bcomp.distance(word),
            Self::Indel(bcomp) => bcomp.distance(word),
            Self::Hamming(bcomp) => bcomp.distance(word).unwrap(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Stat {
    word: String,
    offsets: Vec<u16>,
    words_checking: u32,
    words_skipped: u32,
    suggestions: u32,
    time_total: Duration,
    time_checking: Duration,
}

impl SpellChecker {
    /// Creates a new SpellChecker from the given file.
    ///
    /// The file should contain a dataset of words, sorted by their byte length, where each word is divided by \n
    /// and each group by \n\n. The dataset should also be sorted alphabeticaly.
    pub fn new(file: impl AsRef<Path>) -> Self {
        let (blob, offsets) = load_words_dict(file).unwrap();
        Self {
            blob,
            len_offsets: offsets,
            max_dif: 2,
        }
    }

    pub fn set_max_dif(&mut self, max_dif: usize) -> &mut Self {
        self.max_dif = max_dif;
        self
    }

    // pub fn add(&mut self, word: String) {
    //     self.words.insert(Box::leak(Box::new(word)));
    // }

    pub fn check(&self, word: &str) -> bool {
        self.find(word).is_some()
    }

    // pub fn batch_check(&self, words: &[&str]) -> 

    /// Finds the word in the dataset if it exists.
    ///
    /// Returns the offsets of the word in the blob if it exists, otherwise None.
    pub fn find(&self, word: &str) -> Option<(usize, usize)> {
        let word = word.to_lowercase();
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

    /// Finds a word in a given slice of bytes using binary search.
    ///
    /// The slice should contain words of the same length, sorted alphabetically.
    ///
    /// Returns the offsets of the word in the slice if it exists, otherwise None.
    /// The offsets are given as a tuple of (start, end) where start is the index of the first byte of the word,
    /// and end is the index of the last byte of the word plus one.
    fn find_word_in_slice_binary_search(word: &[u8], slice: &[u8]) -> Option<(usize, usize)> {
        let mut low = 0usize;
        let mut high = slice.len() / word.len();
        while low < high {
            let mid = low + ((high - low) / 2);
            let mid_off = mid * word.len();
            let candidate = &slice[mid_off..(mid_off + word.len())];
            match word.cmp(candidate) {
                Ordering::Equal => return Some((mid_off, mid_off + word.len())),
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
    pub fn suggest(&self, word: &str, take_first_x: usize) -> Vec<&str> {
        let st = Instant::now();
        let word = word.to_lowercase();
        let word_len = word.len();

        if let Some(offset) = self.find(&word) {
            return vec![&self.blob[offset.0..offset.1]];
        }

        let min_len = word_len.saturating_sub(self.max_dif) - 1;
        let max_len = (word_len + self.max_dif).min(self.len_offsets.len());
        
        let words = &self.len_offsets[min_len..max_len];

        let first_char = word.bytes().next();
        let last_char = word.bytes().last();

        let bcomp_ind = rapidfuzz::distance::indel::BatchComparator::new(word.as_bytes());
        let bcomp_lev = rapidfuzz::distance::levenshtein::BatchComparator::new(word.as_bytes());
        let bcomp_ham = rapidfuzz::distance::hamming::BatchComparator::new(word.as_bytes());

        let cst = Instant::now();
        let mut result: Vec<(&str, usize)> = words
            .par_iter()
            .flat_map(|group| {
                let bcomp = if (group.len as isize - word_len as isize).abs() as usize == self.max_dif {
                    ComparatorWrapper::Indel(&bcomp_ind)
                } else if group.len as usize == word_len {
                    ComparatorWrapper::Hamming(&bcomp_ham)
                } else {
                    ComparatorWrapper::Levenstein(&bcomp_lev)
                };

                let text = &self.blob[group.offset as usize
                    ..(group.offset+(group.len as u32*group.count as u32)) as usize];

                text
                    .as_bytes()
                    .par_chunks(group.len as usize)
                    .filter_map(|ch| {
                        if (group.len as isize - word_len as isize).abs() as usize == self.max_dif {
                            if let (Some(fc), Some(lc)) = (first_char, last_char) {
                                if !ch.contains(&fc) && !ch.contains(&lc) {
                                    return None;
                                }
                            }
                        }
                        
                        let dist = bcomp.distance(ch);
                        if dist <= self.max_dif {
                            // Dataset will always be valid, and chars are based on len group. Cant have invalid utf-8.
                            // Trust
                            let word = unsafe {
                                from_utf8_unchecked(ch)
                            };
                            Some((word, dist))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let cst = cst.elapsed();

        if result.len() > 1 {
            result.par_sort_unstable_by_key(|(_, dist)| *dist);
        }

        println!("{:#?}", Stat {
            word: word.into(),
            offsets: {
                let mut groups = vec![];
                for g in words {
                    groups.push(g.len);
                }
                groups
            },
            words_checking: {
                let mut len = 0u32;
                for g in words {
                    len += (g.len * g.count) as u32;
                }
                len
            },
            words_skipped: 0,
            suggestions: result.len() as u32,
            time_total: st.elapsed(),
            time_checking: cst,
        });

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
    pub fn batch_suggest(&self, words: &[&str], take_first_x: usize) -> Vec<Vec<&str>> {
        self.batch_suggest_iter(words, take_first_x).collect()
    }

    /// Calls the given callback for each word in the given slice of words, with the suggestions generated by `batch_suggest`.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    pub fn batch_suggest_with<F>(&self, words: &[&str], take_first_x: usize, mut callback: F)
    where F: FnMut(&str, Vec<&str>), {
        words
            .iter()
            .for_each(move |word| {
                let suggestions = self.suggest(word, take_first_x);
                callback(word, suggestions)
        });
    }
    
    /// Returns an iterator over the suggestions for each of the given words.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    pub fn batch_suggest_iter(&self, words: &[&str], take_first_x: usize) -> impl Iterator<Item = Vec<&str>> {
        words
            .iter()
            .map(move |word| self.suggest(word, take_first_x))
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
    pub fn batch_par_suggest(&self, words: &[&str], take_first_x: usize) -> Vec<Vec<&str>> {
        self.batch_par_suggest_iter(words, take_first_x).collect()
    }

    /// Calls the given callback for each word in the given slice of words, with the suggestions generated by `batch_par_suggest`.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    pub fn batch_par_suggest_with<F>(&self, words: &[&str], take_first_x: usize, callback: F)
    where F: FnMut(&str, Vec<&str>) + Send + Sync + Clone, {
        words
            .par_iter()
            .for_each_with(callback, move |cb, word| {
                let suggestions = self.suggest(word, take_first_x);
                cb(word, suggestions)
        });
    }

    /// Returns an iterator over the suggestions for each of the given words.
    ///
    /// This is a parallelized version of batch_suggest.
    ///
    /// The suggestions are generated by computing the Levenshtein distance between each of the given words and all the words in the dictionary.
    /// The results are sorted by the distance in ascending order.
    ///
    /// If the given word is in the dictionary, only that word is returned.
    ///
    /// If take_first_x is 0, all suggestions are returned.
    /// Otherwise, only the first take_first_x suggestions are returned.
    pub fn batch_par_suggest_iter(&self, words: &[&str], take_first_x: usize) -> impl ParallelIterator<Item = Vec<&str>> {
        words
            .par_iter()
            .map(move |word| self.suggest(word, take_first_x))
    }
}

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
) -> Result<(String, Vec<LenGroup>), Box<dyn std::error::Error>> {    // TODO: Still pretty slow, may be can be improved.
    // About 2 ms
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
    Ok((blob, filled_offsets))
}