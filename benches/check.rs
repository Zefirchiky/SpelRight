#![feature(test)]
extern crate test;
#[cfg(test)]
mod tests {
    use mangahub_spellchecker::{SpellChecker};
    use test::{Bencher};

    static WORDS_FILE: &str = "C:/dev/tools/basic-spellchecker/words.txt";

    #[bench]
    #[ignore = "too long"]
    fn iter_100000_correct_words(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);
        let bench_words: [&str; 30] = [
            "the",
            "quick",
            "brown",
            "fox",
            "jumps",
            "over",
            "lazy",
            "dog",
            "and",
            "sees",
            "nothing",
            "important",
            "about",
            "this",
            "statement",
            "programming",
            "algorithm",
            "structure",
            "compiler",
            "testing",
            "application",
            "language",
            "keyboard",
            "monitor",
            "software",
            "hardware",
            "network",
            "database",
            "system",
            "function",
        ];

        let mut words = Vec::with_capacity(100_000);
        let len = bench_words.len();
        for i in 0..100_000 {
            words.push(bench_words[i % len]);
        }
        
        b.iter(|| {
            _ = words
                .iter()
                .map(|word| {
                    checker.check(word)
                })
                .collect::<Vec<bool>>();
        });
    }

    #[bench]
    #[ignore = "too long"]
    fn iter_100000_inorrect_words(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);
        let bench_words: [&str; 30] = [
            "teh",         // Transposition (the)
            "quik",        // Omission (quick)
            "broown",      // Insertion (brown)
            "foz",         // Substitution (fox)
            "jumsp",       // Transposition (jumps)
            "oevr",        // Transposition (over)
            "lazey",       // Substitution (lazy)
            "doog",        // Insertion (dog)
            "adn",         // Omission (and)
            "seesd",       // Insertion (sees)
            "nothng",      // Omission (nothing)
            "imortant",    // Omission (important)
            "abuot",       // Transposition (about)
            "tiis",        // Insertion (this)
            "statemant",   // Substitution (statement)
            "prograaming", // Insertion (programming)
            "algorthm",    // Omission (algorithm)
            "struckture",  // Substitution (structure)
            "compiller",   // Insertion (compiler)
            "tsting",      // Omission (testing)
            "applicaion",  // Omission (application)
            "laguage",     // Omission (language)
            "keybord",     // Omission (keyboard)
            "monitr",      // Omission (monitor)
            "sotware",     // Substitution (software)
            "hardwear",    // Substitution (hardware)
            "netwerk",     // Substitution (network)
            "databae",     // Omission (database)
            "sistem",      // Substitution (system)
            "funciton",    // Transposition (function)
        ];
        
        let mut words = Vec::with_capacity(100_000);
        let len = bench_words.len();
        for i in 0..100_000 {
            words.push(bench_words[i % len]);
        }
        
        b.iter(|| {
            _ = words
                .iter()
                .map(|word| {
                    checker.check(word)
                })
                .collect::<Vec<bool>>();
        });
    }
}