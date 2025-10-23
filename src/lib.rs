use std::{cmp::Ordering, fs, path::Path, str::{from_utf8_unchecked}, time::{Duration, Instant}};

use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct LenGroup {
    blob: String,
    len: u16,
    count: u16,
}

pub struct SpellChecker {
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
        let offsets = load_words_dict(file).unwrap();
        Self {
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
    /// Returns the LenGroup, start and end offsets of the word in the group if it exists, otherwise None.
    pub fn find(&self, word: &str) -> Option<(&LenGroup, (usize, usize))> {
        let word = word.to_lowercase();
        let lg = &self.len_offsets.get(word.len() - 1)?;
        if lg.count == 0 {
            return None;
        }

        let word = word.as_bytes();
        let blob = lg.blob.as_bytes();
        let offsets = Self::find_word_in_slice_binary_search(word, blob)?;
        Some((lg, offsets))
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

    // #[inline]
    // fn matches_single(
    //     word: &[u8],
    //     candidate: &[u8],
    //     mut max_deletions: u16,
    //     mut max_insertions: u16,
    //     mut max_changes: u16,
    // ) -> bool {
    //     let mut wi = 0;
    //     let mut ci = 0;
        
    //     while wi < word.len() && ci < candidate.len() {
    //         if word[wi] == candidate[ci] {
    //             wi += 1;
    //             ci += 1;
    //         } else if max_changes > 0 {
    //             // Substitution
    //             max_changes -= 1;
    //             wi += 1;
    //             ci += 1;
    //         } else {
    //             // No changes left, try deletion or insertion
    //             if wi + 1 < word.len() && max_deletions > 0 && word[wi + 1] == candidate[ci] {
    //                 // Deletion: skip byte in word
    //                 max_deletions -= 1;
    //                 wi += 1;
    //             } else if ci + 1 < candidate.len() && max_insertions > 0 && word[wi] == candidate[ci + 1] {
    //                 // Insertion: skip byte in candidate
    //                 max_insertions -= 1;
    //                 ci += 1;
    //             } else {
    //                 return false;
    //             }
    //         }
    //     }
        
    //     // Handle remaining bytes
    //     let remaining_word = word.len() - wi;
    //     let remaining_candidate = candidate.len() - ci;
        
    //     if remaining_word == 0 && remaining_candidate == 0 {
    //         return true;
    //     }
        
    //     if remaining_word == 0 {
    //         return remaining_candidate as u16 <= max_insertions;
    //     }
        
    //     if remaining_candidate == 0 {
    //         return remaining_word as u16 <= max_deletions;
    //     }
        
    //     false
    // }

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

        if let Some((lg, offset)) = self.find(&word) {
            return vec![&lg.blob[offset.0..offset.1]];
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

                group.blob
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