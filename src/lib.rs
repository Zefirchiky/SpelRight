#![feature(exact_div)]

use std::{cmp::Ordering, path::Path, str::from_utf8_unchecked};

use rayon::prelude::*;

mod load_dict;
mod simd_find_matching_prefix;

pub use load_dict::load_words_dict;
use simd_find_matching_prefix::{find_matching_prefix_simd_avx2, find_matching_prefix_simd_sse2};

pub enum BinarySearchWordResult {
    Found(usize, usize),
    NotFound(usize, usize),
}

#[derive(Debug, Clone)]
pub struct LenGroup {
    blob: String,
    len: u16,
    count: u16,
}

#[deprecated(
    since = "0.2.2", 
    note = "The crate has been renamed. Please use the 'spel-right' crate instead."
)]
impl LenGroup {
    pub fn empty(len: u16) -> Self {
        Self {
            blob: String::new(),
            len,
            count: 0
        }
    }
}

pub struct SpellChecker {
    len_groups: Vec<LenGroup>,
    max_dif: usize,
    // added_words: Vec<String>,
    // added_words_treshhold: usize,
}

impl SpellChecker {
    /// Creates a new `SpellChecker` from the given `file`.
    ///
    /// The `file` should be formated acording to [Dataset Fixer](https://github.com/Zefirchiky/easy-spell-checker/tree/ca505359efdc0a862d3418ae3c8b9f0418a9f25e/dataset_fixer) (see also `load_words_dict()`)
    pub fn new(file: impl AsRef<Path>) -> Self {
        let len_groups = load_words_dict(file).unwrap();
        Self {
            len_groups,
            max_dif: 2,
            // added_words: vec![],
            // added_words_treshhold: 20,
        }
    }

    /// Sets the maximum difference between words to be considered similar.
    ///
    /// This value is used in the suggest method to determine how many words to suggest.
    ///
    /// A value of `0` means that only exact matches are considered similar, while a value of `1` means that words that are one `insertion`, `deletion`, or `substitution` away are also considered similar.
    ///
    /// A value of `2` (the default) means that words that are up to two `insertions`, `deletions`, or `substitutions` away are also considered similar.
    pub fn set_max_dif(&mut self, max_dif: usize) -> &mut Self {
        self.max_dif = max_dif;
        self
    }

    pub fn add(&mut self, word: String) -> &mut Self {
        // self.added_words.push(word);
        // if self.added_words.len() >= self.added_words_treshhold {
        //     self.save()
        // }
        let res = self.find_closest(&word);
        if let Some((lg, BinarySearchWordResult::NotFound(o1, _))) = res {
            let i = (lg.len - 1) as usize;
            self.len_groups.get_mut(i).unwrap().blob.insert_str(o1, &word); // FIXME: Inefficient, needs to move all the words after. It should also be responsibility of LenGroup
        }
        self
    }

    pub fn save(&mut self) {
        // let added_words = mem::take(&mut self.added_words);
        // for word in added_words {
        //     let wlen = word.len();
        //     while wlen - 1 > self.len_groups.len() {
        //         self.len_groups.push(LenGroup::empty(self.len_groups.len() as u16));
        //     }
        //     if wlen - 1 == self.len_groups.len() {
        //         self.len_groups.push(LenGroup {
        //             blob: word,
        //             len: wlen as u16,
        //             count: 1,
        //         });
        //     } else {
        //         self.len_groups[wlen-1].blob.push_str(&word);
        //     }
        // }

        // for gr in self.len_groups {
        //     gr.blob.
        // }
    }

    /// Checks if a word exists in the dataset.
    ///
    /// Returns true if the word exists, false otherwise.
    pub fn check(&self, word: &str) -> bool {
        self.find(word).is_some()
    }

    pub fn batch_check<'a>(&self, words: &'a [&str]) -> Vec<(&'a str, bool)> {
        words
            .iter()
            .map(|&word| {
                (word, self.check(word))
            })
            .collect()
    }

    pub fn batch_par_check<'a>(&self, words: &'a [&str]) -> Vec<(&'a str, bool)> {
        words
            .par_iter()
            .map(|&word| {
                (word, self.check(word))
            })
            .collect()
    }

    /// Finds a word in the dataset and returns its length group and offsets if found.
    ///
    /// The word is first converted to lowercase, and then the length group is searched for.
    /// If the word is found in the length group, its offsets are found using binary search.
    /// If the word is not found, None is returned.
    pub fn find(&self, word: &str) -> Option<(&LenGroup, (usize, usize))> {
        if let (lg, BinarySearchWordResult::Found(o1, o2)) = self.find_closest(word)? {
            Some((lg, (o1, o2)))
        } else {
            None
        }
    }

    pub fn find_closest(&self, word: &str) -> Option<(&LenGroup, BinarySearchWordResult)> {
        let word = word.to_lowercase();
        let lg = &self.len_groups.get(word.len() - 1)?;
        if lg.count == 0 {
            return None;
        }

        let word = word.as_bytes();
        let blob = lg.blob.as_bytes();
        Some((lg, Self::find_word_in_slice_binary_search(word, blob)))
    }

    /// Finds a word in a given slice of bytes using binary search.
    ///
    /// The slice should contain words of the same length, sorted alphabetically.
    ///
    /// Returns the offsets of the word in the slice if it exists, otherwise the closest offset to it.
    /// The offsets are given as a tuple of (start, end) where start is the index of the first byte of the word,
    /// and end is the index of the last byte of the word plus one.
    fn find_word_in_slice_binary_search(word: &[u8], slice: &[u8]) -> BinarySearchWordResult {  // TODO: move into LenGroup
        let mut low = 0usize;
        let mut high = slice.len().checked_div(word.len()).unwrap();
        let mut mid_off = 0;
        while low < high {
            let mid = low + ((high - low) / 2);
            mid_off = mid * word.len();
            let candidate = &slice[mid_off..(mid_off + word.len())];
            match word.cmp(candidate) {
                Ordering::Equal => return BinarySearchWordResult::Found(mid_off, mid_off + word.len()),
                Ordering::Less => high = mid,
                Ordering::Greater => low = mid + 1,
            }
        }
        BinarySearchWordResult::NotFound(mid_off, mid_off + word.len())
    }

    /// Checks if a word matches a given candidate with at most the given maximum amount of `deletions`, `insertions` and `substitution`.
    ///
    /// Returns a tuple of `(bool, u16)` where the boolean is `true` if the word matches the candidate, and the `u16` is the total number of operations done to match the two words.
    ///
    /// The algorithm first finds the matching prefix of the two words using `SIMD` if available, and then continues with a scalar algorithm from the mismatch point.
    ///
    /// The maximum amount of `deletions`, `insertions` and `substitutions` are given as mutable parameters, and are decreased by one each time an operation is done.
    ///
    /// If the word matches the candidate with at most the given maximum amount of operations, the function returns true and the total number of operations done.
    /// Otherwise, it returns `false` and `0`.
    #[inline]
    pub fn matches_single(
        word: &[u8],
        candidate: &[u8],
        mut max_deletions: u16,
        mut max_insertions: u16,
        mut max_substitutions: u16,
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
            else if max_substitutions > 0 {
                max_substitutions -= 1;
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
                max_deletions - remaining_word + max_insertions - remaining_candidate + max_substitutions,
            )
        } else {
            (false, 0)
        }
    }

    /// Finds all words in the dataset that are similar to the given `word`.
    ///
    /// Similarity is defined as the maximum number of `deletions`, `insertions`, and `substitutions` that can be done to match the two words.
    /// The maximum difference is specified by the `max_dif` field of the `SpellChecker`.
    ///
    /// The function returns a vector of tuples, where the first element of the tuple is the similar word, and the second element is the distance between the two words.
    ///
    /// The function uses a parallel iterator to search for similar words in the dataset.
    ///
    /// The function first filters out all words that are not of the same length as the given `word`, or that have a difference greater than the maximum difference.
    /// It then uses the `matches_single` function to check if each word is similar to the given `word`.
    /// If a word is similar, it is added to the result vector.
    ///
    /// The function finally collects the result vector and returns it.
    pub fn suggest_for_word(&self, word: &[u8]) -> Vec<(&str, usize)> {
        let word_len = word.len();
        
        let min_len = word_len.saturating_sub(self.max_dif - 1);
        let max_len = (word_len + self.max_dif).min(self.len_groups.len());
        
        let first_char = word[0];
        let last_char = word[word_len - 1];
        let words = &self.len_groups[min_len..max_len];
        words
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
                            ch, word, max_del, max_ins, max_chg
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
            .collect()

    }
    
    /// Suggests words for a given `word` based on the maximum difference specified in the constructor.
    ///
    /// If the `word` is found in the dataset, returns a vector with the given `word`.
    ///
    /// If the `word` is not found in the dataset, `SpellChecker::suggest_for_word()` will be used.
    ///
    /// Returns the result vector, sorted by the distance, and takes the first `take_first_x` elements.
    pub fn suggest(&self, word: &str, take_first_x: usize) -> Vec<&str> {
        let word = word.to_lowercase();
        
        if let Some((lg, offset)) = self.find(&word) {
            return vec![&lg.blob[offset.0..offset.1]];
        }
        
        let word_bytes = word.as_bytes();
        let mut result = self.suggest_for_word(word_bytes);

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

    /// Suggests words for each `word` in the given `words` vector based on the maximum difference specified in the constructor.
    ///
    /// If a `word` is found in the dataset, returns a vector with the given `word`.
    ///
    /// If a `word` is not found in the dataset, `SpellChecker::suggest_for_word()` will be used.
    ///
    /// Returns the result vector, sorted by the distance, and takes the first `take_first_x` elements.
    ///
    pub fn batch_suggest<'a>(&self, words: &'a [&str], take_first_x: usize) -> Vec<(&'a str, Vec<&str>)> {
        self.batch_suggest_iter(words, take_first_x).collect()
    }

    /// Iterates over each `word` in the given `words` vector and calls the given `callback` function with the suggestions for each word.
    ///
    /// The `callback` function will be called with two arguments: the original `word`, and a vector of suggestions for that word.
    ///
    /// The suggestions vector will contain all words that are at most `max_dif` away from the given `word`.
    ///
    /// The suggestions vector will be sorted by the distance, with the closest words first.
    /// If the `word` is found in the dataset, the suggestions vector will contain the given `word`.
    ///
    /// The `callback` function will be called for each `word` in the given `words` vector.
    pub fn batch_suggest_with<F>(&self, words: &[&str], take_first_x: usize, mut callback: F)
    where F: FnMut(&str, Vec<&str>), {
        words
            .iter()
            .for_each(move |word| {
                let suggestions = self.suggest(word, take_first_x);
                callback(word, suggestions)
        });
    }

    /// Iterates over each `word` in the given `words` vector and calls `suggest` function with the given `word` and `take_first_x`.
    ///
    /// The `suggest` function will return a vector of suggestions for each word, sorted by the distance, with the closest words first.
    ///
    /// The `suggest` function will also return the given `word` if it is found in the dataset.
    ///
    /// The `suggest` function will take the first `take_first_x` elements of the suggestions vector.
    ///
    /// The function returns an iterator over the suggestions vectors.
    pub fn batch_suggest_iter<'a>(&self, words: &'a [&str], take_first_x: usize) -> impl Iterator<Item = (&'a str, Vec<&str>)> {
        words
            .iter()
            .map(move |&word| (word, self.suggest(word, take_first_x)))
    }
    
    /// Iterates over each `word` in the given `words` vector and calls `suggest` function with the given `word` and `take_first_x`.
    ///
    /// The `suggest` function will return a vector of suggestions for each word, sorted by the distance, with the closest words first.
    ///
    /// The `suggest` function will also return the given `word` if it is found in the dataset.
    ///
    /// The `suggest` function will take the first `take_first_x` elements of the suggestions vector.
    ///
    /// The function returns an iterator over the suggestions vectors.
    ///
    /// This function is the same as `batch_suggest`, but it uses rayon's parallel iterator, which means it will use all available CPU cores in parallel to suggest words for all given words.
    ///
    /// The function returns a vector of suggestions vectors.
    ///
    /// The function is parallel, and will use all available CPU cores in parallel.
    pub fn batch_par_suggest<'a>(&self, words: &'a [&str], take_first_x: usize) -> Vec<(&'a str, Vec<&str>)> {
        self.batch_par_suggest_iter(words, take_first_x).collect()
    }

    /// Iterates over each `word` in the given `words` vector and calls the given `callback` function with the suggestions for each word.
    ///
    /// The `callback` function will be called with two arguments: the original `word`, and a vector of suggestions for that word.
    ///
    /// The suggestions vector will contain all words that are at most `max_dif` away from the given `word`.
    ///
    /// The suggestions vector will be sorted by the distance, with the closest words first.
    /// If the `word` is found in the dataset, the suggestions vector will contain the given `word`.
    ///
    /// The `callback` function will be called for each `word` in the given `words` vector.
    ///
    /// The function is parallel, and will use all available CPU cores in parallel.
    pub fn batch_par_suggest_with<F>(&self, words: &[&str], take_first_x: usize, callback: F)
    where F: FnMut(&str, Vec<&str>) + Send + Sync + Clone, {
        words
            .par_iter()
            .for_each_with(callback, move |cb, word| {
                let suggestions = self.suggest(word, take_first_x);
                cb(word, suggestions)
        });
    }

    /// Iterates over each `word` in the given `words` vector and calls `suggest` function with the given `word` and `take_first_x`.
    ///
    /// The `suggest` function will return a vector of suggestions for each word, sorted by the distance, with the closest words first.
    ///
    /// If the `word` is found in the dataset, the suggestions vector will contain the given `word`.
    ///
    /// The `suggest` function will take the first `take_first_x` elements of the suggestions vector.
    ///
    /// The function returns a parallel iterator over the suggestions vectors.
    pub fn batch_par_suggest_iter<'a>(&self, words: &'a [&str], take_first_x: usize) -> impl ParallelIterator<Item = (&'a str, Vec<&str>)> {
        words
            .par_iter()
            .map(move |&word| (word, self.suggest(word, take_first_x)))
    }
}