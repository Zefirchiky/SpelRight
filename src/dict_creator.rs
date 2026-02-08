use std::marker::PhantomData;

use crate::{DICT_VERSION, DictMetadata, Language, SC, spell_checkers::{ascii, normalized, utf8, words_to_groups}};

#[derive(Debug, Default)]
pub struct SpellCheckerBuilder<L: Language> {
    word_amount: usize,
    ascii: Option<ascii::SpellChecker>,
    norm: Option<normalized::SpellChecker>,
    utf8: Option<utf8::SpellChecker>,
    lang: PhantomData<L>,
}

impl<L: Language + Default> SpellCheckerBuilder<L> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_ascii_words(&mut self, words: Vec<String>) -> Option<&mut Self> {
        self.word_amount += words.len();
        let groups = words_to_groups(words)?;
        self.ascii = Some(ascii::SpellChecker::new(groups));
        Some(self)
    }

    /// WIP
    /// 
    /// Normalized words will be checked against their ascii counterparts
    /// and mapped to utf-8 forms
    pub fn add_norm_words(&mut self, _words: Vec<String>) -> Option<&mut Self> {
        // self.word_amount += words.len();
        // let ascii_groups = words_to_groups(
        //     words
        //         .par_iter()
        //         .map(|w: &str| {
        //             w.nfd()
        //                 .filter(|ch| ch.is_ascii())
        //                 .collect::<String>()
        //         })
        //         .collect()
        // )?;
        // for word in words {

        // }
        // self.norm = Some(words);
        Some(self)
    }

    pub fn add_utf8_words(&mut self, words: Vec<String>) -> Option<&mut Self> {
        self.word_amount += words.len();
        let groups = words_to_groups(words)?;
        self.utf8 = Some(utf8::SpellChecker::new(groups));
        Some(self)
    }

    pub fn build(&mut self) -> SC<L> {
        let mut encodings = vec![];
        if self.ascii.is_some() { encodings.push("ascii".to_owned()) }
        if self.norm.is_some() { encodings.push("normalized".to_owned()) }
        if self.utf8.is_some() { encodings.push("utf-8".to_owned()) }
        SC {
            ascii_checker: self.ascii.take(),
            norm_checker: self.norm.take(),
            utf8_checker: self.utf8.take(),
            dict_meta: DictMetadata {
                language_full: L::full_name().into(),
                language_short: L::name().into(),
                version: DICT_VERSION,
                words_amount: self.word_amount,
                included_encodings: encodings,
            },
            language: PhantomData,
        }
    }
}