use std::env::{self, current_exe};

use spel_right::SpellChecker;

fn main() {
    let path = current_exe().unwrap();
    let path = path.parent().unwrap();
    let path = path.join("words.txt");

    let mut checker = SpellChecker::new(path);
    checker.max_dif = 4;

    let mut return_elements = 10;
    let args: Vec<String> = env::args().collect();
    let words_to_check = args.get(1..).unwrap();
    let words_to_check: Vec<&str> = words_to_check
        .iter()
        .filter_map(|s| {
            let s = s.as_str();
            if s == "--full" {
                return_elements = 0;
                return None;
            }
            Some(s)
        })
        .collect();

    let suggestions = checker.batch_par_suggest(&words_to_check, return_elements);

    for (word, suggestion) in suggestions {
        if suggestion.is_empty() {
            println!("❌ Wrong word '{word}', no suggestions")
        } else if suggestion.len() == 1 {
            println!("✅ {word}")
        } else {
            println!("❓ {word} => {}", &suggestion.join(" "))
        }
    }
}
