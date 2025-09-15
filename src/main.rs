#![warn(dead_code)]

mod levenstein_dist;
use levenstein_dist::levenstein_dist;

use std::time::{Duration, Instant};
use std::fs;

use rayon::prelude::*;
use rustc_hash::FxHashSet;
use memmap2::Mmap;

mod bk_tree;


pub struct SpellChecker {
    words: FxHashSet<String>
}

impl SpellChecker {
    pub fn new() -> Self {
        SpellChecker { words: FxHashSet::default() }
    }

    pub fn load_dictionary(&mut self, dict: &Vec<String>) {
        self.words.extend(dict.clone());
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

    pub fn batch_suggest(&self, words: Vec<&str>) -> Vec<Vec<String>> {
        words
            .par_iter()
            .map(|word| self.suggest(word))
            .collect()
    }
}


fn load_words_dict(file: &str) -> Result<Vec<String>, std::io::Error> {
    let file = fs::File::open(file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap();
    let words: Vec<String> = content
        .par_lines()
        .map(|line| line.to_owned())
        .collect();

    println!("Words loaded: {}", words.len());
    Ok(words)
}

fn main() {
    let start = Instant::now();

    let words = load_words_dict("words_alpha.txt").unwrap();
    println!("Loading dict took: {:?}", start.elapsed());
    println!("Dict: {}", words.len());

    let mut checker = SpellChecker::new();
    checker.load_dictionary(&words);
    println!("Load in checker: {:?}", start.elapsed());

    println!("{}", checker.check("Hello"));
    println!("{}", checker.check("Hell"));
    println!("{}", checker.check("Algorithm"));
    println!("Check: {:?}", start.elapsed());

    let mut total: Duration = Duration::new(0, 0);
    for _ in 0..5 {
        let start = Instant::now();
        let v = checker.suggest("hell");
        println!("{:?}", v);
        let v = checker.suggest("beeuty");
        println!("{:?}", v);
        let v = checker.suggest("chill");
        println!("{:?}", v);
        let v = checker.suggest("fucts");
        println!("{:?}", v);
        let v = checker.suggest("chungus");
        println!("{:?}", v);
        let v = checker.suggest("mayonese");
        println!("{:?}", v);
        total += start.elapsed();
    }
    println!("Suggest took: {:?}", total / 5);
    
    println!();
    let mut total: Duration = Duration::new(0, 0);
    for _ in 0..5 {
        let start = Instant::now();
        let v = checker.batch_suggest(vec!("hell", "beeuty", "chill", "fucts", "chungus", "mayonese"));
        println!("{:?}", v);
        total += start.elapsed();
    }
    println!("Batch suggest took: {:?}", total / 5);
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
}
