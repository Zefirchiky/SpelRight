use std::marker::PhantomData;

use filess::{Json, ModelFileTrait};
use serde::{Deserialize, Serialize};

use crate::{DictMetadata, Language, spell_checkers::{SpellCheckerTrait, ascii, normalized, utf8}};

#[derive(Serialize, Deserialize)]
pub struct SpellChecker<L: Language> {
    pub(crate) dict_meta: DictMetadata,
    pub(crate) ascii_checker: Option<ascii::SpellChecker>,
    pub(crate) norm_checker: Option<normalized::SpellChecker>,
    pub(crate) utf8_checker: Option<utf8::SpellChecker>,
    pub(crate) language: PhantomData<L>,
}

impl<L: Language> SpellChecker<L> {
    pub fn new() -> Result<Self, <Json as ModelFileTrait>::Error> {
        let file = Json::new(format!("{}.json", L::name()));

        file.load_model::<Self>()
    }

    pub fn new_with_file<F: filess::ModelFileTrait>(file: F) -> Result<Self, F::Error> {
        file.load_model::<Self>()
    }

    pub fn check(&self, word: &str) -> bool {   // FIXME: Branching may add overhead, compiling this for each language with their checkers would be best
        if let Some(checker) = &self.ascii_checker {
            return checker.check(word)
        }
        if let Some(checker) = &self.norm_checker {
            return checker.check(word)
        }
        if let Some(checker) = &self.utf8_checker {
            return checker.check(word)
        }

        false
    }
}
