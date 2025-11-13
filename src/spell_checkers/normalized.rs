/// A group that stores the ascii words blob, whose indexes correspond to utf8 blob of words of given len.
///
/// This struct is used to store the ascii words blob and the corresponding utf8 blob of words of the same length.
/// The ascii words blob is a string of all the ascii words in the dataset, concatenated together without any delimiters.
/// The utf8 blob of words is a string of all the utf8 words in the dataset, concatenated together without any delimiters.
/// The len field stores the length of the words in utf8 blob, whereas ascii len should be stored elsewhere.
pub struct AsciiUtf8LenGroup {
    blob_ascii: String,
    blob_utf8: String,
    len: usize,
}

pub struct LenGroup {
    ascii_utf8_len_groups: Vec<AsciiUtf8LenGroup>,
    len: usize,
}

pub struct SpellChecker {
    len_groups: Vec<LenGroup>,
}