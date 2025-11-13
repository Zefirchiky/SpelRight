pub mod ascii;
pub mod normalized;
pub mod utf8;

pub enum SpellCheckerTypes {
    Ascii(ascii::SpellChecker),
    Normalized(normalized::SpellChecker),
    Utf8(utf8::SpellChecker),
}