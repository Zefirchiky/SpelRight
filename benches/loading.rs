#![feature(test)]
extern crate test;
#[cfg(test)]
mod tests {
    use basic_spellchecker::old::{SpellChecker, load_words_dict};
    use test::Bencher;

    static WORDS_FILE: &str = "C:/dev/tools/basic-spellchecker/words_alpha.txt";

    #[bench]
    fn words_loading_from_file(b: &mut Bencher) {
        b.iter(|| load_words_dict(WORDS_FILE));
    }

    #[bench]
    fn words_loading_into_checker(b: &mut Bencher) {
        let words = load_words_dict(WORDS_FILE).unwrap();
        b.iter(|| {
            let mut checker = SpellChecker::new();
            checker.load_dictionary(&words);
        })
    }
}