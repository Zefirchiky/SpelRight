use std::env::{self, current_exe};
use std::iter::zip;

use basic_spellchecker::old::{SpellChecker, load_words_dict};

fn main() {
    let path = current_exe().unwrap();
    let path = path.parent().unwrap();
    let words = load_words_dict(path.join("words_alpha.txt")).unwrap();

    let mut checker = SpellChecker::new();
    checker.load_dictionary(&words);

    let args: Vec<String> = env::args().collect();
    let words_to_check = args.get(1..).unwrap();
    let words_to_check: Vec<&str> = words_to_check.iter().map(|s| s.as_str()).collect();

    let suggestions = checker.batch_par_suggest(&words_to_check, 10);

    for (word, suggestion) in zip(words_to_check, suggestions) {
        if suggestion.is_empty() {
            println!("❌ Wrong word '{word}', no suggestions")
        } else if suggestion.len() == 1 {
            println!("✅ {word}")
        } else {
            println!("❓ {word} => {}", &suggestion.join(" "))
        }
    }
}