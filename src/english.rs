use crate::Language;

pub struct English;

impl Language for English {
    fn name() -> &'static str {
        "en_us"
    }

    fn full_name() -> &'static str {
        "English (US)"
    }
}