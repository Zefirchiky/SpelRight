use std::path::Path;

pub struct LenGroup {
    blob: String,
    len: usize,
    count: usize,
}

pub struct SpellChecker {
    len_groups: Vec<LenGroup>,
    pub max_dif: usize,
}

impl SpellChecker {
    pub fn new(len_groups: Vec<LenGroup>) -> Self {
        Self {
            len_groups,
            max_dif: 2,
            // added_words: vec![],
            // added_words_treshhold: 20,
        }
    }


}