#![warn(dead_code)]

use std::time::Instant;
use std::fs;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use memmap2::Mmap;


struct SpellChecker {
    // string_pool: Vec<String>,
    words: FxHashSet<String>
}

impl SpellChecker {
    pub fn new() -> Self {
        SpellChecker { words: FxHashSet::default() }
    }

    pub fn load_dictionary(&mut self, dict: Vec<String>) {
        self.words.extend(dict);
    }

    pub fn check(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        let word = word.to_lowercase();

        let result = self.words
            .par_iter()
            .filter(|dict_word| ((dict_word.len() as i32 - word.len() as i32).abs()) <= 1)
            .filter_map(|dict_word| {
                let dist = levenstein_dist(dict_word, &word);
                if dist <= 2 {
                    Some((dict_word.clone(), dist))
                } else {
                    None
                }
            });
        
        let mut result: Vec<(String, usize)> = result.collect();

        result.par_sort_by_key(|(_, dist)| *dist);
        
        result.into_par_iter()
            .take(10)
            .map(|(dict_word, _)| dict_word)
            .collect()
    }
}


pub fn levenstein_dist(s1: &str, s2: &str) -> usize {
    let ch1: Vec<char> = s1.chars().collect();
    let ch2: Vec<char> = s2.chars().collect();
    let m = ch1.len();
    let n = ch2.len();

    let mut prev_row: Vec<usize> = (0..=n).collect();
    let mut cur_row = vec![0; n+1];

    for i in 1..=m {
        cur_row[0] = i;
        for j in 1..=n {
            if ch1[i-1] == ch2[j-1] {
                cur_row[j] = prev_row[j-1]
            } else {
                cur_row[j] = 1 + (cur_row[j-1])
                    .min(prev_row[j])
                    .min(prev_row[j-1]);
            }
        }

        prev_row = cur_row.clone();
    };

    cur_row[n]
}

fn load_words_dict(file: &str) -> Result<Vec<String>, std::io::Error> {
    let file = fs::File::open(file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap();
    let words = content
        .par_lines()
        .map(|line| line.to_owned())
        .collect();

    Ok(words)
}

fn main() {
    let start = Instant::now();

    let words = load_words_dict("words_alpha.txt").unwrap();
    println!("Loading dict took: {:?}", start.elapsed());
    println!("Dict: {}", words.len());

    let mut checker = SpellChecker::new();
    checker.load_dictionary(words);
    println!("Load in checker: {:?}", start.elapsed());

    println!("{}", checker.check("Hello"));
    println!("{}", checker.check("Hell"));
    println!("{}", checker.check("Algorithm"));
    println!("Check: {:?}", start.elapsed());

    for _ in 1..21 {
        let start = Instant::now();
        let v = checker.suggest("hell");
        println!("Suggest took: {:?}", start.elapsed());
        println!("{:?}", v);
    }
}
