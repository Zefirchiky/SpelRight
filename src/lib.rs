use std::{fs, path::Path};

use memmap2::Mmap;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use rapidfuzz::distance::levenshtein;


pub struct SpellChecker {
    words: FxHashSet<String>
}

impl SpellChecker {
    pub fn new() -> Self {
        SpellChecker { words: FxHashSet::default() }
    }

    pub fn load_dictionary(&mut self, dict: &Vec<String>) {
        self.words.par_extend(dict.clone());
    }

    pub fn add(&mut self, word: String) {
        self.words.insert(word);
    }

    pub fn check(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        let word = word.to_lowercase();

        if self.check(&word) { return vec![word] }

        let bcomp = levenshtein::BatchComparator::new(word.chars());
        let result = self.words
            .par_iter()
            .filter(|dict_word| ((dict_word.len() as i32 - word.len() as i32).abs()) <= 1)
            .filter_map(|dict_word| {
                let dist = bcomp.distance(dict_word.chars().into_iter());
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

    pub fn batch_suggest(&self, words: &[&str]) -> Vec<Vec<String>> {
        words
            .par_iter()
            .map(|word| self.suggest(word))
            .collect()
    }
}


pub fn load_words_dict<T: AsRef<Path>>(file: T) -> Result<Vec<String>, std::io::Error> {
    let file = fs::File::open(file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap();
    let words: Vec<String> = content
        .par_lines()
        .map(|line| line.to_owned())
        .collect();

    Ok(words)
}