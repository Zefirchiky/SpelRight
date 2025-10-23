
use std::{cmp::Ordering, path::Path, str::from_utf8_unchecked};

use rayon::prelude::*;

mod load_dict;
mod simd_find_matching_prefix;

pub use load_dict::load_words_dict;
use simd_find_matching_prefix::{find_matching_prefix_simd_avx2, find_matching_prefix_simd_sse2};

#[derive(Debug, Clone)]
pub struct LenGroup {
    blob: String,
    len: u16,
    count: u16,
}

pub struct SpellChecker {
    len_groups: Vec<LenGroup>,
    max_dif: usize,
}

impl SpellChecker {
    /// Creates a new SpellChecker from the given file.
    ///
    /// The file should contain a dataset of words, sorted by their byte length, where each word is divided by \n
    /// and each group by \n\n. The dataset should also be sorted alphabeticaly.
    pub fn new(file: impl AsRef<Path>) -> Self {
        let len_groups = load_words_dict(file).unwrap();
        Self {
            len_groups,
            max_dif: 2,
        }
    }

    /// Sets the maximum difference between words to be considered similar.
    ///
    /// This value is used in the suggest method to determine how many words to suggest.
    ///
    /// A value of 0 means that only exact matches are considered similar, while a value of 1 means that words that are one insertion, deletion, or substitution away are also considered similar.
    ///
    /// A value of 2 (the default) means that words that are up to two insertions, deletions, or substitutions away are also considered similar.
    pub fn set_max_dif(&mut self, max_dif: usize) -> &mut Self {
        self.max_dif = max_dif;
        self
    }

    /// Checks if a word exists in the dataset.
    ///
    /// Returns true if the word exists, false otherwise.
    pub fn check(&self, word: &str) -> bool {
        self.find(word).is_some()
    }

    /// Finds the word in the dataset if it exists.
    ///
    /// Returns the LenGroup, start and end offsets of the word in the group if it exists, otherwise None.
    pub fn find(&self, word: &str) -> Option<(&LenGroup, (usize, usize))> {
        let word = word.to_lowercase();
        let lg = &self.len_groups.get(word.len() - 1)?;
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

    /// Checks if a word matches a given candidate with at most the given maximum amount of deletions, insertions and changes.
    ///
    /// Returns a tuple of (bool, u16) where the boolean is true if the word matches the candidate, and the u16 is the total number of operations done to match the two words.
    ///
    /// The algorithm first finds the matching prefix of the two words using SIMD if available, and then continues with a scalar algorithm from the mismatch point.
    ///
    /// The maximum amount of deletions, insertions and changes are given as mutable parameters, and are decreased by one each time an operation is done.
    ///
    /// If the word matches the candidate with at most the given maximum amount of operations, the function returns true and the total number of operations done.
    /// Otherwise, it returns false and 0.
    #[inline]
    pub fn matches_single(
        word: &[u8],
        candidate: &[u8],
        mut max_deletions: u16,
        mut max_insertions: u16,
        mut max_changes: u16,
    ) -> (bool, u16) {
        let wlen = word.len();
        let clen = candidate.len();
        
        // Find matching prefix using SIMD
        let mut wi = 0;
        let mut ci = 0;
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    wi = find_matching_prefix_simd_avx2(word, candidate);
                    ci = wi;
                }
            }
            else if is_x86_feature_detected!("sse2") {
                unsafe {
                    wi = find_matching_prefix_simd_sse2(word, candidate);
                    ci = wi;
                }
            }
        }
        
        // Continue with scalar algorithm from mismatch point
        while wi < wlen && ci < clen {
            if word[wi] == candidate[ci] {
                wi += 1;
                ci += 1;
            }
            else if max_deletions > 0 && wi + 1 < wlen && word[wi + 1] == candidate[ci] {
                max_deletions -= 1;
                wi += 1;
            }
            else if max_insertions > 0 && ci + 1 < clen && word[wi] == candidate[ci + 1] {
                max_insertions -= 1;
                ci += 1;
            }
            else if max_changes > 0 {
                max_changes -= 1;
                wi += 1;
                ci += 1;
            }
            else {
                return (false, 0);
            }
        }
        
        let remaining_word = (wlen - wi) as u16;
        let remaining_candidate = (clen - ci) as u16;
        
        if remaining_word <= max_deletions && remaining_candidate <= max_insertions {
            (
                true,
                max_deletions - remaining_word + max_insertions - remaining_candidate + max_changes,
            )
        } else {
            (false, 0)
        }
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
        let word = word.to_lowercase();
        let word_len = word.len();
        let word_bytes = word.as_bytes();

        if let Some((lg, offset)) = self.find(&word) {
            return vec![&lg.blob[offset.0..offset.1]];
        }
        
        let min_len = word_len.saturating_sub(self.max_dif) - 1;
        let max_len = (word_len + self.max_dif).min(self.len_groups.len());
        
        let first_char = word_bytes[0];
        let last_char = word_bytes[word_len - 1];
        let words = &self.len_groups[min_len..max_len];
        let mut result: Vec<(&str, usize)> = words
            .par_iter()
            .filter(|group| group.count > 0)
            .flat_map(|group| {
                let dif = group.len as isize - word_len as isize;
                let abs_dif = dif.abs() as usize;

                let max_del = dif.max(0) as u16;
                let max_ins = (-dif).max(0) as u16;
                let max_chg = (self.max_dif - abs_dif) as u16;

                group.blob
                    .as_bytes()
                    .par_chunks(group.len as usize)
                    .filter_map(|ch| {
                        if abs_dif == self.max_dif {
                            if ch[0] != first_char && ch[0] != last_char &&
                            ch[ch.len()-1] != first_char && ch[ch.len()-1] != last_char {
                                return None;
                            }
                        }
                        
                        let (is_ok, dist) = Self::matches_single(
                            ch, word_bytes, max_del, max_ins, max_chg
                        );
                        if is_ok {
                            // Dataset will always be valid, and chars are based on len group. Cant have invalid utf-8.
                            // Trust
                            Some((unsafe { from_utf8_unchecked(ch) }, dist as usize))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if result.len() > 1 {
            result.par_sort_unstable_by_key(|(_, dist)| *dist);
            result.reverse();
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