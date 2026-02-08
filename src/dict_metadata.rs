use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DictMetadata {
    pub language_full: String,
    pub language_short: String,
    pub version: usize,
    pub words_amount: usize,
    pub included_encodings: Vec<String>,
}