#![feature(test)]
extern crate test;
#[cfg(test)]
mod tests {
    use mangahub_spellchecker::{load_words_dict};
    use test::Bencher;

    static WORDS_FILE: &str = "C:/dev/tools/basic-spellchecker/words.txt";

    #[bench]
    fn words_loading_from_file(b: &mut Bencher) {
        b.iter(|| load_words_dict(WORDS_FILE));
    }
}
