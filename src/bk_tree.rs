use std::intrinsics::simd;

use rustc_hash::FxHashMap;

use crate::levenstein_dist::levenstein_dist;


#[derive(Debug, Clone)]
struct BKNode {
    word: String,
    children: FxHashMap<usize, Box<BKNode>>
}

impl BKNode {
    fn new(word: String) -> Self {
        Self { word, children: FxHashMap::default() }
    }
}

pub struct BKTree {
    root: Option<Box<BKNode>>,
    f: fn(&str, &str) -> usize
}

impl BKTree {
    pub fn new(f: fn(&str, &str) -> usize) -> Self {
        Self { root: None, f }
    }

    pub fn insert(&mut self, word: String) {
        match &mut self.root {
            None => {
                self.root = Some(Box::new(BKNode::new(word)));
            }
            Some(root) => {
                Self::insert_recursive(root, word);
            }
        }
    }

    fn insert_recursive(node: &mut BKNode, word: String) {
        let distance = levenstein_dist(&node.word, &word);
        
        // If distance is 0, the word already exists - don't insert duplicate
        if distance == 0 {
            return;
        }

        match node.children.get_mut(&distance) {
            Some(child) => {
                Self::insert_recursive(child, word);
            }
            None => {
                node.children.insert(distance, Box::new(BKNode::new(word)));
            }
        }
    }

    // pub fn load_dictionary(&mut self, words: Vec<String>) {
    //     let first_node = BKNode { word: words.get(words.len() / 2) }
    // }

    pub fn search(&self, word: &str, max_distance: usize) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        if let Some(root) = &self.root {
            Self::search_recursively(root, word, max_distance, &mut result)
        }

        result
    }

    fn search_recursively(node: &BKNode, word: &str, max_distance: usize, result: &mut Vec<String>) {
        let distance = levenstein_dist(&node.word, word);

        if distance <= max_distance {
            result.push(node.word.clone());
        }

        let min_child_distance = distance.saturating_sub(max_distance);
        let max_child_distance = distance + max_distance;
        
        for (&dist, child) in &node.children {
            if dist >= min_child_distance && dist <= max_child_distance {
                Self::search_recursively(&child, word, max_distance, result);
            }
        }
    }
}