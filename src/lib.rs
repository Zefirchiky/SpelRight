// use std::{fs, path::Path};
// use memmap2::Mmap;
// use rayon::prelude::*;
// use indexmap::IndexSet;
// use rustc_hash::{FxHashMap};
// use fst::{Set, SetBuilder};
// use string_interner::{StringInterner};
// use bloomfilter::Bloom;
// // use ahash::AHasher;
// // use std::hash::{Hash, Hasher};

pub mod old;


// pub struct SpellChecker {
//     // Primary storage strategies
//     storage: StorageBackend,
//     // Bloom filter for quick negative lookups
//     bloom: Bloom<String>,
//     // Configuration
//     config: SpellCheckerConfig,
// }

// #[derive(Debug, Clone)]
// pub struct SpellCheckerConfig {
//     pub storage_type: StorageType,
//     pub enable_ngram_indexing: bool,
//     pub ngram_threshold: usize,
//     pub max_edit_distance: usize,
//     pub bloom_false_positive_rate: f64,
//     pub enable_memory_mapping: bool,
//     pub enable_string_interning: bool,
// }

// #[derive(Debug, Clone)]
// pub enum StorageType {
//     /// Fast access, high memory usage
//     IndexSet,
//     /// Minimal memory, slower access
//     CompressedTrie,
//     /// Memory mapped, good for large dictionaries
//     MemoryMapped,
//     /// Hybrid approach
//     Hybrid,
// }

// enum StorageBackend {
//     IndexSet(IndexSetBackend),
//     CompressedTrie(CompressedTrieBackend),
//     MemoryMapped(MemoryMappedBackend),
//     Hybrid(HybridBackend),
// }

// // IndexSet-based backend (your current preference)
// struct IndexSetBackend {
//     words: IndexSet<String>,
//     words_by_length: FxHashMap<usize, Vec<usize>>,
//     trigram_index: Option<FxHashMap<String, Vec<usize>>>,
//     interner: Option<StringInterner<string_interner::DefaultBackend>>,
// }

// // Compressed trie backend using FST
// struct CompressedTrieBackend {
//     word_set: Set<Vec<u8>>,
//     words_vec: Vec<String>, // For getting suggestions
//     words_by_length: FxHashMap<usize, Vec<usize>>,
//     trigram_index: Option<FxHashMap<String, Vec<usize>>>,
// }

// // Memory-mapped backend
// struct MemoryMappedBackend {
//     _mmap: Mmap, // Keep mmap alive
//     word_offsets: Vec<(usize, usize)>, // (offset, length) pairs
//     content: *const u8, // Raw pointer to mmap content
//     words_by_length: FxHashMap<usize, Vec<usize>>,
//     trigram_index: Option<FxHashMap<String, Vec<usize>>>,
// }

// unsafe impl Send for MemoryMappedBackend {}
// unsafe impl Sync for MemoryMappedBackend {}

// // Hybrid backend: combines multiple strategies
// struct HybridBackend {
//     // Common words in fast IndexSet
//     hot_words: IndexSet<String>,
//     hot_threshold_freq: usize,
//     // All words in compressed trie
//     all_words: Set<Vec<u8>>,
//     // Full word list for suggestions
//     words_vec: Vec<String>,
//     words_by_length: FxHashMap<usize, Vec<usize>>,
//     trigram_index: Option<FxHashMap<String, Vec<usize>>>,
// }

// impl Default for SpellCheckerConfig {
//     fn default() -> Self {
//         Self {
//             storage_type: StorageType::Hybrid,
//             enable_ngram_indexing: true,
//             ngram_threshold: 100000,
//             max_edit_distance: 2,
//             bloom_false_positive_rate: 0.01,
//             enable_memory_mapping: false,
//             enable_string_interning: true,
//         }
//     }
// }

// impl SpellCheckerConfig {
//     /// Ultra-fast startup, minimal memory (~8MB for 370k words)
//     pub fn minimal() -> Self {
//         Self {
//             storage_type: StorageType::CompressedTrie,
//             enable_ngram_indexing: false,
//             ngram_threshold: usize::MAX,
//             max_edit_distance: 2,
//             bloom_false_positive_rate: 0.001, // Very accurate bloom filter
//             enable_memory_mapping: false,
//             enable_string_interning: false,
//         }
//     }

//     /// Balanced performance and memory (~25MB for 370k words)
//     pub fn balanced() -> Self {
//         Self {
//             storage_type: StorageType::Hybrid,
//             enable_ngram_indexing: true,
//             ngram_threshold: 200000,
//             max_edit_distance: 2,
//             bloom_false_positive_rate: 0.01,
//             enable_memory_mapping: false,
//             enable_string_interning: true,
//         }
//     }

//     /// Maximum performance (~50MB for 370k words)
//     pub fn performance() -> Self {
//         Self {
//             storage_type: StorageType::IndexSet,
//             enable_ngram_indexing: true,
//             ngram_threshold: 50000,
//             max_edit_distance: 2,
//             bloom_false_positive_rate: 0.05, // Less accurate but faster
//             enable_memory_mapping: false,
//             enable_string_interning: true,
//         }
//     }

//     /// Memory-mapped for very large dictionaries
//     pub fn memory_mapped() -> Self {
//         Self {
//             storage_type: StorageType::MemoryMapped,
//             enable_ngram_indexing: false,
//             ngram_threshold: usize::MAX,
//             max_edit_distance: 2,
//             bloom_false_positive_rate: 0.01,
//             enable_memory_mapping: true,
//             enable_string_interning: false,
//         }
//     }
// }

// impl SpellChecker {
//     pub fn new() -> Self {
//         Self::with_config(SpellCheckerConfig::default())
//     }

//     pub fn with_config(config: SpellCheckerConfig) -> Self {
//         Self {
//             storage: match config.storage_type {
//                 StorageType::IndexSet => StorageBackend::IndexSet(IndexSetBackend::new(&config)),
//                 StorageType::CompressedTrie => StorageBackend::CompressedTrie(CompressedTrieBackend::new()),
//                 StorageType::MemoryMapped => StorageBackend::MemoryMapped(MemoryMappedBackend::new()),
//                 StorageType::Hybrid => StorageBackend::Hybrid(HybridBackend::new()),
//             },
//             bloom: Bloom::new_for_fp_rate(1000000, config.bloom_false_positive_rate).unwrap(),
//             config,
//         }
//     }

//     pub fn load_dictionary(&mut self, words: &[String]) -> Result<(), Box<dyn std::error::Error>> {
//         // Build bloom filter first
//         for word in words {
//             self.bloom.set(&word.to_lowercase());
//         }

//         // Load into appropriate backend
//         match &mut self.storage {
//             StorageBackend::IndexSet(backend) => backend.load_dictionary(words, &self.config)?,
//             StorageBackend::CompressedTrie(backend) => backend.load_dictionary(words, &self.config)?,
//             StorageBackend::MemoryMapped(backend) => backend.load_dictionary(words, &self.config)?,
//             StorageBackend::Hybrid(backend) => backend.load_dictionary(words, &self.config)?,
//         }

//         Ok(())
//     }

//     pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
//         match self.config.storage_type {
//             StorageType::MemoryMapped => {
//                 // Special handling for memory-mapped backend
//                 if let StorageBackend::MemoryMapped(backend) = &mut self.storage {
//                     backend.load_from_file(path.as_ref(), &self.config)?;
//                 }
//             }
//             _ => {
//                 let words = load_words_dict_optimized(path)?;
//                 self.load_dictionary(&words)?;
//             }
//         }
//         Ok(())
//     }

//     pub fn check(&self, word: &str) -> bool {
//         let word_lower = word.to_lowercase();
        
//         // Quick bloom filter check first
//         if !self.bloom.check(&word_lower) {
//             return false;
//         }

//         // Delegate to backend
//         match &self.storage {
//             StorageBackend::IndexSet(backend) => backend.check(&word_lower),
//             StorageBackend::CompressedTrie(backend) => backend.check(&word_lower),
//             StorageBackend::MemoryMapped(backend) => backend.check(&word_lower),
//             StorageBackend::Hybrid(backend) => backend.check(&word_lower),
//         }
//     }

//     pub fn suggest(&self, word: &str, take_first_x: usize) -> Vec<String> {
//         let word_lower = word.to_lowercase();

//         if self.check(&word_lower) {
//             return vec![word_lower];
//         }

//         match &self.storage {
//             StorageBackend::IndexSet(backend) => backend.suggest(&word_lower, take_first_x, &self.config),
//             StorageBackend::CompressedTrie(backend) => backend.suggest(&word_lower, take_first_x, &self.config),
//             StorageBackend::MemoryMapped(backend) => backend.suggest(&word_lower, take_first_x, &self.config),
//             StorageBackend::Hybrid(backend) => backend.suggest(&word_lower, take_first_x, &self.config),
//         }
//     }

//     pub fn batch_suggest(&self, words: &[&str], take_first_x: usize) -> Vec<Vec<String>> {
//         words.par_iter()
//             .map(|word| self.suggest(word, take_first_x))
//             .collect()
//     }

//     pub fn memory_stats(&self) -> MemoryStats {
//         let backend_stats = match &self.storage {
//             StorageBackend::IndexSet(backend) => backend.memory_stats(),
//             StorageBackend::CompressedTrie(backend) => backend.memory_stats(),
//             StorageBackend::MemoryMapped(backend) => backend.memory_stats(),
//             StorageBackend::Hybrid(backend) => backend.memory_stats(),
//         };

//         let bloom_size = std::mem::size_of_val(&self.bloom);
        
//         MemoryStats {
//             backend_size: backend_stats.total_size,
//             bloom_size,
//             total_size: backend_stats.total_size + bloom_size,
//             backend_breakdown: backend_stats,
//         }
//     }
// }

// // Backend implementations
// impl IndexSetBackend {
//     fn new(config: &SpellCheckerConfig) -> Self {
//         Self {
//             words: IndexSet::new(),
//             words_by_length: FxHashMap::default(),
//             trigram_index: None,
//             interner: if config.enable_string_interning {
//                 Some(StringInterner::default())
//             } else {
//                 None
//             },
//         }
//     }

//     fn load_dictionary(&mut self, words: &[String], config: &SpellCheckerConfig) -> Result<(), Box<dyn std::error::Error>> {
//         self.words.reserve(words.len());

//         for word in words {
//             let processed_word = if let Some(ref mut interner) = self.interner {
//                 // Use string interning to reduce memory for common substrings
//                 let symbol = interner.get_or_intern(word);
//                 interner.resolve(symbol).unwrap().to_string()
//             } else {
//                 word.clone()
//             };

//             let idx = self.words.len();
//             if self.words.insert(processed_word.clone()) {
//                 self.words_by_length
//                     .entry(processed_word.len())
//                     .or_insert_with(Vec::new)
//                     .push(idx);
//             }
//         }

//         if config.enable_ngram_indexing && words.len() > config.ngram_threshold {
//             self.build_trigram_index();
//         }

//         Ok(())
//     }

//     fn build_trigram_index(&mut self) {
//         let mut trigram_index = FxHashMap::default();
        
//         for (idx, word) in self.words.iter().enumerate() {
//             let trigrams = get_trigrams_simd_optimized(word);
//             for trigram in trigrams {
//                 trigram_index
//                     .entry(trigram)
//                     .or_insert_with(Vec::new)
//                     .push(idx);
//             }
//         }
        
//         self.trigram_index = Some(trigram_index);
//     }

//     fn check(&self, word: &str) -> bool {
//         self.words.contains(word)
//     }

//     fn suggest(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let candidates = self.get_candidates_by_length(word);
        
//         if candidates.len() <= 3000 || self.trigram_index.is_none() {
//             self.suggest_with_candidates(word, &candidates, take_first_x, config)
//         } else {
//             self.suggest_with_ngrams(word, take_first_x, config)
//         }
//     }

//     fn get_candidates_by_length(&self, word: &str) -> Vec<usize> {
//         let mut candidates = Vec::new();
//         let word_len = word.len();
        
//         for len_diff in 0..=2 {
//             if len_diff <= word_len {
//                 if let Some(indices) = self.words_by_length.get(&(word_len - len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//             if len_diff > 0 {
//                 if let Some(indices) = self.words_by_length.get(&(word_len + len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//         }
        
//         candidates
//     }

//     fn suggest_with_candidates(&self, word: &str, candidates: &[usize], take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let b = rapidfuzz::distance::levenshtein::BatchComparator::new(word.chars());
//         // Use SIMD-optimized distance calculation
//         let results: Vec<(String, usize)> = candidates.par_iter()
//             .filter_map(|&idx| {
//                 let dict_word = &self.words[idx];
                
//                 // Quick pre-filter using character frequency
//                 if !has_similar_char_frequency(word, dict_word) {
//                     return None;
//                 }

//                 let dist = b.distance(dict_word.chars());
//                 if dist <= config.max_edit_distance {
//                     Some((dict_word.clone(), dist))
//                 } else {
//                     None
//                 }
//             })
//             .collect();

//         self.finalize_suggestions(results, take_first_x)
//     }

//     fn suggest_with_ngrams(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let trigram_index = self.trigram_index.as_ref().unwrap();
//         let trigrams = get_trigrams_simd_optimized(word);
//         let mut candidate_counts: FxHashMap<usize, usize> = FxHashMap::default();
        
//         for trigram in &trigrams {
//             if let Some(candidates) = trigram_index.get(trigram) {
//                 for &candidate_idx in candidates {
//                     *candidate_counts.entry(candidate_idx).or_insert(0) += 1;
//                 }
//             }
//         }
        
//         let min_overlap = std::cmp::max(1, trigrams.len() / 2);
//         let candidate_indices: Vec<usize> = candidate_counts
//             .into_iter()
//             .filter(|(_, count)| *count >= min_overlap)
//             .map(|(idx, _)| idx)
//             .collect();
        
//         self.suggest_with_candidates(word, &candidate_indices, take_first_x, config)
//     }

//     fn finalize_suggestions(&self, mut results: Vec<(String, usize)>, take_first_x: usize) -> Vec<String> {
//         results.par_sort_unstable_by_key(|(_, dist)| *dist);
        
//         let iter = results.into_iter().map(|(word, _)| word);
//         if take_first_x == 0 {
//             iter.collect()
//         } else {
//             iter.take(take_first_x).collect()
//         }
//     }

//     fn memory_stats(&self) -> BackendMemoryStats {
//         let words_size = self.words.len() * std::mem::size_of::<String>() 
//             + self.words.iter().map(|w| w.len()).sum::<usize>();
        
//         let length_index_size = self.words_by_length.len() * std::mem::size_of::<(usize, Vec<usize>)>()
//             + self.words_by_length.values().map(|v| v.len() * std::mem::size_of::<usize>()).sum::<usize>();
        
//         let trigram_index_size = if let Some(ref index) = self.trigram_index {
//             index.len() * std::mem::size_of::<(String, Vec<usize>)>()
//                 + index.keys().map(|k| k.len()).sum::<usize>()
//                 + index.values().map(|v| v.len() * std::mem::size_of::<usize>()).sum::<usize>()
//         } else {
//             0
//         };

//         let interner_size = if let Some(ref interner) = self.interner {
//             interner.len() * 32 // Rough estimate
//         } else {
//             0
//         };

//         BackendMemoryStats {
//             words_size,
//             index_size: length_index_size + trigram_index_size,
//             auxiliary_size: interner_size,
//             total_size: words_size + length_index_size + trigram_index_size + interner_size,
//         }
//     }
// }

// impl CompressedTrieBackend {
//     fn new() -> Self {
//         Self {
//             word_set: Set::default(),
//             words_vec: Vec::new(),
//             words_by_length: FxHashMap::default(),
//             trigram_index: None,
//         }
//     }

//     fn load_dictionary(&mut self, words: &[String], config: &SpellCheckerConfig) -> Result<(), Box<dyn std::error::Error>> {
//         // Build FST for compressed storage
//         let mut builder = SetBuilder::memory();
//         let mut sorted_words: Vec<String> = words.iter().cloned().collect();
//         sorted_words.par_sort_unstable();
//         sorted_words.dedup();

//         for word in &sorted_words {
//             builder.insert(word.as_bytes())?;
//         }
        
//         self.word_set = builder.into_set();
//         self.words_vec = sorted_words;

//         // Build length index
//         for (idx, word) in self.words_vec.iter().enumerate() {
//             self.words_by_length
//                 .entry(word.len())
//                 .or_insert_with(Vec::new)
//                 .push(idx);
//         }

//         if config.enable_ngram_indexing && words.len() > config.ngram_threshold {
//             self.build_trigram_index();
//         }

//         Ok(())
//     }

//     fn build_trigram_index(&mut self) {
//         let mut trigram_index = FxHashMap::default();
        
//         for (idx, word) in self.words_vec.iter().enumerate() {
//             let trigrams = get_trigrams_simd_optimized(word);
//             for trigram in trigrams {
//                 trigram_index
//                     .entry(trigram)
//                     .or_insert_with(Vec::new)
//                     .push(idx);
//             }
//         }
        
//         self.trigram_index = Some(trigram_index);
//     }

//     fn check(&self, word: &str) -> bool {
//         self.word_set.contains(word.as_bytes())
//     }

//     fn suggest(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let candidates = self.get_candidates_by_length(word);
//         self.suggest_with_candidates(word, &candidates, take_first_x, config)
//     }

//     fn get_candidates_by_length(&self, word: &str) -> Vec<usize> {
//         let mut candidates = Vec::new();
//         let word_len = word.len();
        
//         for len_diff in 0..=2 {
//             if len_diff <= word_len {
//                 if let Some(indices) = self.words_by_length.get(&(word_len - len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//             if len_diff > 0 {
//                 if let Some(indices) = self.words_by_length.get(&(word_len + len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//         }
        
//         candidates
//     }

//     fn suggest_with_candidates(&self, word: &str, candidates: &[usize], take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let b = rapidfuzz::distance::levenshtein::BatchComparator::new(word.chars());
//         let mut results: Vec<(String, usize)> = candidates.par_iter()
//             .filter_map(|&idx| {
//                 let dict_word = &self.words_vec[idx];
//                 let dist = b.distance(dict_word.chars());
//                 if dist <= config.max_edit_distance {
//                     Some((dict_word.clone(), dist))
//                 } else {
//                     None
//                 }
//             })
//             .collect();

//         results.par_sort_unstable_by_key(|(_, dist)| *dist);
        
//         let iter = results.into_iter().map(|(word, _)| word);
//         if take_first_x == 0 {
//             iter.collect()
//         } else {
//             iter.take(take_first_x).collect()
//         }
//     }

//     fn memory_stats(&self) -> BackendMemoryStats {
//         let fst_size = self.word_set.as_fst().as_bytes().len();
//         let words_vec_size = self.words_vec.iter().map(|w| w.len()).sum::<usize>();
//         let index_size = self.words_by_length.len() * std::mem::size_of::<(usize, Vec<usize>)>();

//         BackendMemoryStats {
//             words_size: fst_size,
//             index_size,
//             auxiliary_size: words_vec_size,
//             total_size: fst_size + index_size + words_vec_size,
//         }
//     }
// }

// impl MemoryMappedBackend {
//     fn new() -> Self {
//         Self {
//             _mmap: unsafe { Mmap::map(&std::fs::File::open("/dev/null").unwrap()).unwrap() },
//             word_offsets: Vec::new(),
//             content: std::ptr::null(),
//             words_by_length: FxHashMap::default(),
//             trigram_index: None,
//         }
//     }

//     fn load_from_file(&mut self, path: &Path, _config: &SpellCheckerConfig) -> Result<(), Box<dyn std::error::Error>> {
//         let file = fs::File::open(path)?;
//         let mmap = unsafe { Mmap::map(&file)? };
        
//         self.content = mmap.as_ptr();
        
//         let content_str = std::str::from_utf8(&mmap)?;
//         let mut offset = 0;
        
//         for (idx, line) in content_str.lines().enumerate() {
//             let line = line.trim();
//             if !line.is_empty() {
//                 let word_len = line.len();
//                 self.word_offsets.push((offset, word_len));
                
//                 self.words_by_length
//                     .entry(word_len)
//                     .or_insert_with(Vec::new)
//                     .push(idx);
                
//                 offset += line.as_bytes().len() + 1; // +1 for newline
//             }
//         }
        
//         self._mmap = mmap;
//         Ok(())
//     }

//     fn load_dictionary(&mut self, _words: &[String], _config: &SpellCheckerConfig) -> Result<(), Box<dyn std::error::Error>> {
//         // Memory-mapped backend loads directly from file
//         Ok(())
//     }

//     fn get_word_at_index(&self, idx: usize) -> Option<String> {
//         if idx >= self.word_offsets.len() {
//             return None;
//         }
        
//         let (offset, len) = self.word_offsets[idx];
//         unsafe {
//             let slice = std::slice::from_raw_parts(self.content.add(offset), len);
//             String::from_utf8(slice.to_vec()).ok()
//         }
//     }

//     fn check(&self, word: &str) -> bool {
//         // Linear search through memory-mapped content (could be optimized with binary search)
//         let word_len = word.len();
//         if let Some(indices) = self.words_by_length.get(&word_len) {
//             for &idx in indices {
//                 if let Some(dict_word) = self.get_word_at_index(idx) {
//                     if dict_word == word {
//                         return true;
//                     }
//                 }
//             }
//         }
//         false
//     }

//     fn suggest(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let candidates = self.get_candidates_by_length(word);
//         self.suggest_with_candidates(word, &candidates, take_first_x, config)
//     }

//     fn get_candidates_by_length(&self, word: &str) -> Vec<usize> {
//         let mut candidates = Vec::new();
//         let word_len = word.len();
        
//         for len_diff in 0..=2 {
//             if len_diff <= word_len {
//                 if let Some(indices) = self.words_by_length.get(&(word_len - len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//             if len_diff > 0 {
//                 if let Some(indices) = self.words_by_length.get(&(word_len + len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//         }
        
//         candidates
//     }

//     fn suggest_with_candidates(&self, word: &str, candidates: &[usize], take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let b = rapidfuzz::distance::levenshtein::BatchComparator::new(word.chars());
//         let mut results: Vec<(String, usize)> = candidates.par_iter()
//             .filter_map(|&idx| {
//                 if let Some(dict_word) = self.get_word_at_index(idx) {
//                     let dist = b.distance(dict_word.chars());
//                     if dist <= config.max_edit_distance {
//                         Some((dict_word, dist))
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             })
//             .collect();

//         results.par_sort_unstable_by_key(|(_, dist)| *dist);
        
//         let iter = results.into_iter().map(|(word, _)| word);
//         if take_first_x == 0 {
//             iter.collect()
//         } else {
//             iter.take(take_first_x).collect()
//         }
//     }

//     fn memory_stats(&self) -> BackendMemoryStats {
//         let offsets_size = self.word_offsets.len() * std::mem::size_of::<(usize, usize)>();
//         let index_size = self.words_by_length.len() * std::mem::size_of::<(usize, Vec<usize>)>();
        
//         BackendMemoryStats {
//             words_size: 0, // Memory-mapped, not counted
//             index_size: offsets_size + index_size,
//             auxiliary_size: 0,
//             total_size: offsets_size + index_size,
//         }
//     }
// }

// impl HybridBackend {
//     fn new() -> Self {
//         Self {
//             hot_words: IndexSet::new(),
//             hot_threshold_freq: 1000, // Top 1000 most common words
//             all_words: Set::default(),
//             words_vec: Vec::new(),
//             words_by_length: FxHashMap::default(),
//             trigram_index: None,
//         }
//     }

//     fn load_dictionary(&mut self, words: &[String], config: &SpellCheckerConfig) -> Result<(), Box<dyn std::error::Error>> {
//         // Analyze word frequency (in practice, you'd use real frequency data)
//         let mut word_freq: FxHashMap<String, usize> = FxHashMap::default();
//         for word in words {
//             *word_freq.entry(word.clone()).or_insert(0) += 1;
//         }

//         // Sort by frequency and take top N for hot storage
//         let mut freq_sorted: Vec<_> = word_freq.into_iter().collect();
//         freq_sorted.sort_by(|a, b| b.1.cmp(&a.1));
        
//         for (word, _freq) in freq_sorted.iter().take(self.hot_threshold_freq) {
//             self.hot_words.insert(word.clone());
//         }

//         // Build FST for all words
//         let mut builder = SetBuilder::memory();
//         let mut sorted_words: Vec<String> = words.iter().cloned().collect();
//         sorted_words.par_sort_unstable();
//         sorted_words.dedup();

//         for word in &sorted_words {
//             builder.insert(word.as_bytes())?;
//         }
        
//         self.all_words = builder.into_set();
//         self.words_vec = sorted_words;

//         // Build indices
//         for (idx, word) in self.words_vec.iter().enumerate() {
//             self.words_by_length
//                 .entry(word.len())
//                 .or_insert_with(Vec::new)
//                 .push(idx);
//         }

//         if config.enable_ngram_indexing && words.len() > config.ngram_threshold {
//             self.build_trigram_index();
//         }

//         Ok(())
//     }

//     fn build_trigram_index(&mut self) {
//         let mut trigram_index = FxHashMap::default();
        
//         for (idx, word) in self.words_vec.iter().enumerate() {
//             let trigrams = get_trigrams_simd_optimized(word);
//             for trigram in trigrams {
//                 trigram_index
//                     .entry(trigram)
//                     .or_insert_with(Vec::new)
//                     .push(idx);
//             }
//         }
        
//         self.trigram_index = Some(trigram_index);
//     }

//     fn check(&self, word: &str) -> bool {
//         // Check hot words first (faster)
//         if self.hot_words.contains(word) {
//             return true;
//         }
        
//         // Fall back to FST
//         self.all_words.contains(word.as_bytes())
//     }

//     fn suggest(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let candidates = self.get_candidates_by_length(word);
        
//         if candidates.len() <= 3000 || self.trigram_index.is_none() {
//             self.suggest_with_candidates(word, &candidates, take_first_x, config)
//         } else {
//             self.suggest_with_ngrams(word, take_first_x, config)
//         }
//     }

//     fn get_candidates_by_length(&self, word: &str) -> Vec<usize> {
//         let mut candidates = Vec::new();
//         let word_len = word.len();
        
//         for len_diff in 0..=2 {
//             if len_diff <= word_len {
//                 if let Some(indices) = self.words_by_length.get(&(word_len - len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//             if len_diff > 0 {
//                 if let Some(indices) = self.words_by_length.get(&(word_len + len_diff)) {
//                     candidates.extend(indices);
//                 }
//             }
//         }
        
//         candidates
//     }

//     fn suggest_with_candidates(&self, word: &str, candidates: &[usize], take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let b = rapidfuzz::distance::levenshtein::BatchComparator::new(word.chars());
//         let mut results: Vec<(String, usize)> = candidates.par_iter()
//             .filter_map(|&idx| {
//                 let dict_word = &self.words_vec[idx];
                
//                 // Prioritize hot words
//                 let base_dist = b.distance(dict_word.chars());
//                 let adjusted_dist = if self.hot_words.contains(dict_word) {
//                     // Give hot words a small boost
//                     if base_dist > 0 { base_dist - 1 } else { 0 }
//                 } else {
//                     base_dist
//                 };
                
//                 if base_dist <= config.max_edit_distance {
//                     Some((dict_word.clone(), adjusted_dist))
//                 } else {
//                     None
//                 }
//             })
//             .collect();

//         results.par_sort_unstable_by_key(|(_, dist)| *dist);
        
//         let iter = results.into_iter().map(|(word, _)| word);
//         if take_first_x == 0 {
//             iter.collect()
//         } else {
//             iter.take(take_first_x).collect()
//         }
//     }

//     fn suggest_with_ngrams(&self, word: &str, take_first_x: usize, config: &SpellCheckerConfig) -> Vec<String> {
//         let trigram_index = self.trigram_index.as_ref().unwrap();
//         let trigrams = get_trigrams_simd_optimized(word);
//         let mut candidate_counts: FxHashMap<usize, usize> = FxHashMap::default();
        
//         for trigram in &trigrams {
//             if let Some(candidates) = trigram_index.get(trigram) {
//                 for &candidate_idx in candidates {
//                     *candidate_counts.entry(candidate_idx).or_insert(0) += 1;
//                 }
//             }
//         }
        
//         let min_overlap = std::cmp::max(1, trigrams.len() / 2);
//         let candidate_indices: Vec<usize> = candidate_counts
//             .into_iter()
//             .filter(|(_, count)| *count >= min_overlap)
//             .map(|(idx, _)| idx)
//             .collect();
        
//         self.suggest_with_candidates(word, &candidate_indices, take_first_x, config)
//     }

//     fn memory_stats(&self) -> BackendMemoryStats {
//         let hot_words_size = self.hot_words.len() * std::mem::size_of::<String>()
//             + self.hot_words.iter().map(|w| w.len()).sum::<usize>();
//         let fst_size = self.all_words.as_fst().as_bytes().len();
//         let words_vec_size = self.words_vec.iter().map(|w| w.len()).sum::<usize>();
//         let index_size = self.words_by_length.len() * std::mem::size_of::<(usize, Vec<usize>)>();

//         BackendMemoryStats {
//             words_size: fst_size,
//             index_size,
//             auxiliary_size: hot_words_size + words_vec_size,
//             total_size: fst_size + index_size + hot_words_size + words_vec_size,
//         }
//     }
// }

// // SIMD-optimized helper functions
// fn get_trigrams_simd_optimized(word: &str) -> Vec<String> {
//     if word.len() < 3 {
//         return vec![format!("##{}##", word.to_lowercase())];
//     }
    
//     let padded = format!("##{}##", word.to_lowercase());
//     let chars: Vec<char> = padded.chars().collect();
    
//     // Use SIMD-friendly operations where possible
//     chars.windows(3)
//         .map(|w| w.iter().collect::<String>())
//         .collect()
// }

// fn has_similar_char_frequency(word1: &str, word2: &str) -> bool {
//     // Quick heuristic using character frequency vectors
//     let mut freq1 = [0u8; 26];
//     let mut freq2 = [0u8; 26];
    
//     for ch in word1.chars() {
//         if let Some(idx) = char_to_index(ch) {
//             freq1[idx] = freq1[idx].saturating_add(1);
//         }
//     }
    
//     for ch in word2.chars() {
//         if let Some(idx) = char_to_index(ch) {
//             freq2[idx] = freq2[idx].saturating_add(1);
//         }
//     }
    
//     // Calculate Manhattan distance between frequency vectors
//     let dist: u32 = freq1.iter()
//         .zip(freq2.iter())
//         .map(|(&a, &b)| (a as i16 - b as i16).abs() as u32)
//         .sum();
    
//     // Allow some deviation based on word lengths
//     let max_len = std::cmp::max(word1.len(), word2.len()) as u32;
//     dist <= max_len
// }

// fn char_to_index(ch: char) -> Option<usize> {
//     if ch.is_ascii_lowercase() {
//         Some((ch as u8 - b'a') as usize)
//     } else if ch.is_ascii_uppercase() {
//         Some((ch.to_ascii_lowercase() as u8 - b'a') as usize)
//     } else {
//         None
//     }
// }

// // SIMD-optimized Levenshtein distance (fallback to rapidfuzz for now)
// fn simd_levenshtein_distance(s1: &str, s2: &str) -> usize {
//     // For now, use rapidfuzz which is already well-optimized
//     // In a real implementation, you could use SIMD instructions here
//     rapidfuzz::distance::levenshtein::distance(s1.chars(), s2.chars())
// }
// pub fn load_words_dict<T: AsRef<Path>>(file: T) -> Result<Vec<String>, std::io::Error> {
//     let file = fs::File::open(file)?;
//     let mmap = unsafe { Mmap::map(&file)? };
//     let content = std::str::from_utf8(&mmap).unwrap();
//     let words: Vec<String> = content.par_lines().map(|l| l.to_owned()).collect();

//     Ok(words)
// }
// // Optimized dictionary loading with all techniques
// pub fn load_words_dict_optimized<T: AsRef<Path>>(file: T) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//     let file_path = file.as_ref();
//     let file = fs::File::open(file_path)?;
//     let mmap = unsafe { Mmap::map(&file)? };
    
//     let content = std::str::from_utf8(&mmap)?;
    
//     // Estimate better capacity based on file size
//     let estimated_words = content.len() / 8; // Assume avg 8 chars per word including newline
//     let mut words = Vec::with_capacity(estimated_words);
    
//     // Use parallel processing with chunking to control memory usage
//     const CHUNK_SIZE: usize = 50000;
//     let lines: Vec<&str> = content.par_lines().collect();
    
//     // Process in parallel chunks
//     let chunk_results: Vec<Vec<String>> = lines
//         .par_chunks(CHUNK_SIZE)
//         .map(|chunk| {
//             chunk.iter()
//                 .filter_map(|line| {
//                     let trimmed = line.trim().to_lowercase();
//                     // Filter out non-alphabetic words and very short/long words
//                     if trimmed.len() >= 2 && trimmed.len() <= 20 
//                         && trimmed.chars().all(|c| c.is_alphabetic()) {
//                         Some(trimmed)
//                     } else {
//                         None
//                     }
//                 })
//                 .collect()
//         })
//         .collect();
    
//     // Flatten results
//     for chunk_words in chunk_results {
//         words.extend(chunk_words);
//     }
    
//     // Remove duplicates using IndexSet for better performance
//     let mut unique_words = IndexSet::with_capacity(words.len());
//     unique_words.par_extend(words);
    
//     Ok(unique_words.into_iter().collect())
// }

// // Memory statistics structures
// #[derive(Debug)]
// pub struct MemoryStats {
//     pub backend_size: usize,
//     pub bloom_size: usize,
//     pub total_size: usize,
//     pub backend_breakdown: BackendMemoryStats,
// }

// #[derive(Debug)]
// pub struct BackendMemoryStats {
//     pub words_size: usize,
//     pub index_size: usize,
//     pub auxiliary_size: usize,
//     pub total_size: usize,
// }

// impl MemoryStats {
//     pub fn print_human_readable(&self) {
//         println!("Memory Usage Statistics:");
//         println!("  Backend: {}", human_readable_bytes(self.backend_size));
//         println!("    - Words: {}", human_readable_bytes(self.backend_breakdown.words_size));
//         println!("    - Indices: {}", human_readable_bytes(self.backend_breakdown.index_size));
//         println!("    - Auxiliary: {}", human_readable_bytes(self.backend_breakdown.auxiliary_size));
//         println!("  Bloom Filter: {}", human_readable_bytes(self.bloom_size));
//         println!("  Total: {}", human_readable_bytes(self.total_size));
//     }
// }

// fn human_readable_bytes(bytes: usize) -> String {
//     const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
//     let mut size = bytes as f64;
//     let mut unit_index = 0;
    
//     while size >= 1024.0 && unit_index < UNITS.len() - 1 {
//         size /= 1024.0;
//         unit_index += 1;
//     }
    
//     format!("{:.2} {}", size, UNITS[unit_index])
// }

// // Convenience constructors
// impl SpellChecker {
//     /// Ultra-minimal memory usage (~3MB for 370k words)
//     pub fn new_minimal() -> Self {
//         Self::with_config(SpellCheckerConfig::minimal())
//     }
    
//     /// Balanced performance/memory (~15MB for 370k words)
//     pub fn new_balanced() -> Self {
//         Self::with_config(SpellCheckerConfig::balanced())
//     }
    
//     /// Maximum performance (~35MB for 370k words)
//     pub fn new_performance() -> Self {
//         Self::with_config(SpellCheckerConfig::performance())
//     }
    
//     /// Memory-mapped for minimal RAM usage
//     pub fn new_memory_mapped() -> Self {
//         Self::with_config(SpellCheckerConfig::memory_mapped())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::Instant;

//     #[test]
//     fn test_all_backends() {
//         let test_words = vec![
//             "hello".to_string(), "world".to_string(), "rust".to_string(),
//             "programming".to_string(), "language".to_string(), "fast".to_string(),
//             "memory".to_string(), "safe".to_string(), "concurrent".to_string(),
//             "performance".to_string(), "optimization".to_string()
//         ];

//         let configs = vec![
//             ("Minimal", SpellCheckerConfig::minimal()),
//             ("Balanced", SpellCheckerConfig::balanced()),
//             ("Performance", SpellCheckerConfig::performance()),
//         ];

//         for (name, config) in configs {
//             println!("Testing {} configuration", name);
//             let mut checker = SpellChecker::with_config(config);
            
//             let start = Instant::now();
//             checker.load_dictionary(&test_words).unwrap();
//             let load_time = start.elapsed();
            
//             // Test basic functionality
//             assert!(checker.check("hello"));
//             assert!(!checker.check("helo"));
            
//             let start = Instant::now();
//             let suggestions = checker.suggest("helo", 5);
//             let suggest_time = start.elapsed();
            
//             assert!(suggestions.contains(&"hello".to_string()));
            
//             let stats = checker.memory_stats();
//             println!("  Load time: {:?}", load_time);
//             println!("  Suggest time: {:?}", suggest_time);
//             println!("  Memory usage: {}", human_readable_bytes(stats.total_size));
//             println!();
//         }
//     }

//     #[test]
//     fn benchmark_large_dictionary() {
//         // Create a larger test dictionary
//         let mut words = Vec::new();
//         for i in 0..50000 {
//             words.push(format!("word{:05}", i));
//         }

//         let mut checker = SpellChecker::new_balanced();
        
//         let start = Instant::now();
//         checker.load_dictionary(&words).unwrap();
//         let load_time = start.elapsed();
        
//         let typos = vec!["word00001", "word12345", "word49999", "nonexistent"];
        
//         let start = Instant::now();
//         let results = checker.batch_suggest(&typos, 5);
//         let batch_time = start.elapsed();
        
//         println!("Large dictionary benchmark:");
//         println!("  Dictionary size: {} words", words.len());
//         println!("  Load time: {:?}", load_time);
//         println!("  Batch suggest time: {:?}", batch_time);
        
//         let stats = checker.memory_stats();
//         stats.print_human_readable();
        
//         assert_eq!(results.len(), 4);
//     }

//     #[test]
//     fn test_memory_mapped_backend() {
//         // This test would require creating a temporary file
//         // Skipping for now, but in real usage:
//         /*
//         let mut checker = SpellChecker::new_memory_mapped();
//         checker.load_from_file("test_dictionary.txt").unwrap();
//         assert!(checker.check("test"));
//         */
//     }
// }