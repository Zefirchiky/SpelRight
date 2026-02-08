use serde::{Deserialize, Serialize};

use crate::{WordId, spell_checkers::{SpellCheckerTrait, simple_len_group::WordGroup}};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpellChecker {
    groups: Vec<WordGroup>,
}

impl SpellChecker {
    pub fn new(word_groups: Vec<WordGroup>) -> Self {
        Self {
            groups: word_groups,
            // added_words: vec![],
            // added_words_treshhold: 20,
        }
    }
}

impl SpellCheckerTrait for SpellChecker {
    fn get(&self, word: WordId) -> Option<&str> {
        let wg = self.groups.get(word.len)?;
        if word.offset >= wg.blob.len() {
            None
        } else {
            Some(&wg.blob[word.offset..word.offset + word.len])
        }
    }

    fn get_unchecked(&self, word: WordId) -> &str {
        let lg = self
            .groups
            .get(word.len)
            .expect(&format!("LenGroup of len {} should exist", word.len));
        &lg.blob[word.offset..word.offset + word.len]
    }

    fn check(&self, word: &str) -> bool {
        let group = self.groups.get(word.len());
        match group {
            Some(wg) => wg.check(word),
            None => false,
        }
    }
}
