use serde::{Deserialize, Serialize};

use crate::spell_checkers::SpellCheckerTrait;

/// A group that stores the ascii words blob, whose indexes correspond to utf8 blob of words of given len.
///
/// This struct is used to store the ascii words blob and the corresponding utf8 blob of words of the same length.
/// The ascii words blob is a string of all the ascii words in the dataset, concatenated together without any delimiters.
/// The utf8 blob of words is a string of all the utf8 words in the dataset, concatenated together without any delimiters.
/// The len field stores the length of the words in utf8 blob, whereas ascii len should be stored elsewhere.
pub struct AsciiUtf8WordGroup {
    // pub blob_ascii: String,             // Ascii words of this length
    // /// Maps ascii words to utf8 by index.
    // ascii_to_utf8: Vec<usize>,
    // pub blob_utf8: String,
    pub len: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordGroup {
    // pub ascii_utf8_word_groups: Vec<AsciiUtf8WordGroup>,
    pub blob_ascii: String,
    /// Maps ascii words to utf8 by index.
    /// Index in this list by ascii word index to get corresponding utf8 index.
    ascii_to_utf8: Vec<usize>,
    pub blob_utf8: String,
    pub len: usize,
}

impl WordGroup {
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpellChecker {
    len_groups: Vec<WordGroup>,
}

impl SpellCheckerTrait for SpellChecker {
    fn check(&self, _word: &str) -> bool {
        false
    }

    fn get(&self, _word: crate::WordId) -> Option<&str> {
        None
    }

    fn get_unchecked(&self, _word: crate::WordId) -> &str {
        ""
    }
}
