use std::cmp::Ordering;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordGroup {
    pub blob: String,
    pub len: usize,
    pub count: usize,
}

impl WordGroup {
    pub fn empty(len: usize) -> Self {
        Self {
            blob: String::new(),
            len,
            count: 0,
        }
    }

    /// Checks if a word exists in the dataset.
    ///
    /// Returns true if the word exists, false otherwise.
    pub fn check(&self, word: &str) -> bool {
        self.find(word).is_some()
    }

    /// Finds a word in the dataset and returns its length group and offsets if found.
    ///
    /// The word is first converted to lowercase, and then the length group is searched for.
    /// If the word is found in the length group, its offsets are found using binary search.
    /// If the word is not found, None is returned.
    pub fn find(&self, word: &str) -> Option<(usize, usize)> {
        if let BinarySearchWordResult::Found(o1, o2) = self.find_closest(word)? {
            Some((o1, o2))
        } else {
            None
        }
    }

    pub fn find_closest(&self, word: &str) -> Option<BinarySearchWordResult> {
        if self.count == 0 {
            return None;
        }
        let word = word.to_lowercase();
        let word = word.as_bytes();
        Some(find_word_in_slice_binary_search(word, self.blob.as_bytes()))
    }
}

pub enum BinarySearchWordResult {
    Found(usize, usize),
    NotFound(usize, usize),
}

pub fn find_word_in_slice_binary_search(word: &[u8], slice: &[u8]) -> BinarySearchWordResult {
    // Supports both ascii and utf-8 without a problem
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

pub fn words_to_groups(mut words: Vec<String>) -> Option<Vec<WordGroup>> {
    words = words.par_iter().filter(|w| w.is_empty()).map(|w| w.to_lowercase()).collect();
    if words.is_empty() { return None }
    
    words.sort_unstable_by(|w1, w2| w1.len().cmp(&w2.len()).then(w1.cmp(w2)));  // TODO: Check if unstable preserves needed order
    let biggest_len = words.last().unwrap().len();
    
    let mut groups: Vec<WordGroup> = Vec::with_capacity(biggest_len);
    for i in 1..biggest_len {
        groups.push(WordGroup::empty(i))
    }

    for word in words {
        let group = groups.get_mut(word.len() - 1).unwrap();
        group.blob.push_str(&word);
        group.count += 1;
    }

    Some(groups)
}
