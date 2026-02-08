#![feature(test)]
extern crate test;
#[cfg(test)]
mod tests {
    use spel_right::SpellChecker;
    use test::Bencher;

    static WORDS_FILE: &str = "words.txt";

    #[bench]
    fn batch_suggest_6_f_words(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);
        let bench_words = vec!["hell", "beeuty", "chill", "fucts", "chungus", "mayonese"];
        b.iter(|| checker.batch_par_suggest(&bench_words, 10))
    }

    #[bench]
    fn batch_suggest_30_correct_words(b: &mut Bencher) {
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
        b.iter(|| checker.batch_suggest(&bench_words, 10));
    }

    #[bench]
    fn batch_suggest_30_incorrect_words(b: &mut Bencher) {
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
        b.iter(|| checker.batch_par_suggest(&bench_words, 10))
    }

    #[bench]
    // #[ignore = "too long"]
    fn batch_suggest_1000_incorrect_words_all_suggestons(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);
        // checker.set_max_dif(4);
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

        let mut words = Vec::with_capacity(1_000);
        let len = bench_words.len();
        for i in 0..1_000 {
            words.push(bench_words[i % len]);
        }

        b.iter(|| checker.batch_par_suggest(&words, 0))
    }

    #[bench]
    fn batch_suggestions_24_mix(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);

        let bench_words = vec![
            "hello",
            "wrold",
            "programing",
            "rust",
            "langauge",
            "spellchecker",
            "performance",
            "algoritm",
            "recieve",
            "seperate",
            "necessary",
            "beautiful",
            "definately",
            "occured",
            "begining",
            "accomodate",
            "wierd",
            "teh",
            "fone",
            "nite",
            "thru",
            "correct",
            "xyz123",
            "supercalifragilisticexpialidocious",
        ];
        b.iter(|| checker.batch_par_suggest(&bench_words, 10))
    }

    #[bench]
    fn batch_suggestions_30_mix(b: &mut Bencher) {
        let checker = SpellChecker::new(WORDS_FILE);

        let bench_words: [&str; 30] = [
            "the",
            "teh",
            "quick",
            "quik",
            "brown",
            "broown",
            "fox",
            "foz",
            "jumps",
            "jumsp",
            "over",
            "oevr",
            "lazy",
            "lazey",
            "dog",
            "doog",
            "and",
            "adn",
            "sees",
            "seesd",
            "nothing",
            "nothng",
            "important",
            "imortant",
            "about",
            "abuot",
            "this",
            "tiis",
            "statement",
            "statemant",
        ];

        b.iter(|| checker.batch_par_suggest(&bench_words, 10))
    }
}
