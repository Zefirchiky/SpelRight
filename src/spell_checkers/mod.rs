pub mod ascii;
pub mod normalized;
pub mod simple_len_group;
pub mod utf8;

pub use simple_len_group::words_to_groups;

use crate::WordId;

pub enum SpellCheckerTypes {
    Ascii(ascii::SpellChecker),
    Normalized(normalized::SpellChecker),
    Utf8(utf8::SpellChecker),
}

pub trait SpellCheckerTrait {
    fn get(&self, word: WordId) -> Option<&str>;
    fn get_unchecked(&self, word: WordId) -> &str;
    
    fn check(&self, word: &str) -> bool;
}
