use std::env::{self, current_exe};
use std::iter::zip;

use basic_spellchecker::{SpellChecker, load_words_dict};


fn main() {
    let path = current_exe().unwrap();
    let path = path.parent().unwrap();
    let words = load_words_dict(path.join("words_alpha.txt")).unwrap();
    println!("Words loaded: {}", words.len());

    let mut checker = SpellChecker::new();
    checker.load_dictionary(&words);

    let args: Vec<String> = env::args().collect();
    let words_to_check = args.get(1..).unwrap();
    let words_to_check: Vec<&str> = words_to_check.iter().map(|s| s.as_str()).collect();

    let suggestions = checker.batch_suggest(&words_to_check);

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


#[cfg(test)]
mod tests {
    use crate::{SpellChecker, load_words_dict};

    #[test]
    fn test_batch_suggestions_speed() {
        use std::time::{Instant, Duration};

        let start = Instant::now();
        
        // let path = current_exe().unwrap();
        // let path = path.parent().unwrap();
        // println!("{path:?}");
        let words = load_words_dict("C:\\dev\\learninig\\Rust\\basic-spellchecker\\words_alpha.txt").unwrap();
        println!("Loading dict took: {:?}", start.elapsed());
        println!("Dict: {}", words.len());
        
        let start = Instant::now();
        let mut checker = SpellChecker::new();
        checker.load_dictionary(&words);
        println!("Load in checker: {:?}", start.elapsed());
        
        let start = Instant::now();
        println!("{}", checker.check("Hello"));
        println!("{}", checker.check("Hell"));
        println!("{}", checker.check("Algorithm"));
        println!("Check: {:?}", start.elapsed());

        // let f_words = vec!("hell", "beeuty", "chill", "fucts", "chungus", "mayonese");
        let benchmark_words = vec![
            "hello", "wrold", "programing", "rust", "langauge", "spellchecker",
            "performance", "algoritm", "recieve", "seperate", "necessary", "beautiful",
            "definately", "occured", "begining", "accomodate", "wierd", "teh",
            "fone", "nite", "thru", "correct", "xyz123", "supercalifragilisticexpialidocious",
        ];

        println!();
        let mut total: Duration = Duration::new(0, 0);
        for _ in 0..100 {
            let start = Instant::now();
            let _v = checker.batch_suggest(&benchmark_words);
            // println!("{:?}", v);
            total += start.elapsed();
        }
        println!("Batch suggest took: {:?}", total / 100);
    }
}