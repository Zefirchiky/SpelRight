use super::simd_find_matching_prefix::{find_matching_prefix_simd_avx2, find_matching_prefix_simd_sse2};

/// Checks if a word matches a given candidate with at most the given maximum amount of `deletions`, `insertions` and `substitution`.
///
/// Returns a tuple of `(bool, u16)` where the boolean is `true` if the word matches the candidate, and the `u16` is the total number of operations done to match the two words.
///
/// The algorithm first finds the matching prefix of the two words using `SIMD` if available, and then continues with a scalar algorithm from the mismatch point.
///
/// The maximum amount of `deletions`, `insertions` and `substitutions` are given as mutable parameters, and are decreased by one each time an operation is done.
///
/// If the word matches the candidate with at most the given maximum amount of operations, the function returns true and the total number of operations done.
/// Otherwise, it returns `false` and `0`.
#[inline]
pub fn matches_single_bytes(
    word: &[u8],
    candidate: &[u8],
    mut max_deletions: u16,
    mut max_insertions: u16,
    mut max_substitutions: u16,
) -> (bool, u16) {
    let wlen = word.len();
    let clen = candidate.len();
    
    // Find matching prefix using SIMD
    let mut wi = 0;
    let mut ci = 0;
    
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                wi = find_matching_prefix_simd_avx2(word, candidate);
            }
        }
        else if is_x86_feature_detected!("sse2") {
            unsafe {
                wi = find_matching_prefix_simd_sse2(word, candidate);
            }
        }
    }

    let wlen = word.len();
    let clen = candidate.len();

    let mut wi = 0;
    let mut ci = 0;
    
    while wi < wlen && ci < clen {
        if word[wi] == candidate[ci] {
            wi += 1;
            ci += 1;
        }
        else if max_deletions > 0 && wi + 1 < wlen && word[wi + 1] == candidate[ci] {
            max_deletions -= 1;
            wi += 1;
        }
        else if max_insertions > 0 && ci + 1 < clen && word[wi] == candidate[ci + 1] {
            max_insertions -= 1;
            ci += 1;
        }
        else if max_substitutions > 0 {
            max_substitutions -= 1;
            wi += 1;
            ci += 1;
        }
        else {
            return (false, 0);
        }
    }
    
    let remaining_word = (wlen - wi) as u16;
    let remaining_candidate = (clen - ci) as u16;
    
    if remaining_word <= max_deletions && remaining_candidate <= max_insertions {
        (
            true,
            max_deletions - remaining_word + max_insertions - remaining_candidate + max_substitutions,
        )
    } else {
        (false, 0)
    }
}

pub fn matches_single<T: Eq>(
    word: &[T],
    candidate: &[T],
    mut max_deletions: u16,
    mut max_insertions: u16,
    mut max_substitutions: u16,
) -> (bool, u16) {
    let wlen = word.len();
    let clen = candidate.len();

    let mut wi = 0;
    let mut ci = 0;
    
    while wi < wlen && ci < clen {
        if word[wi] == candidate[ci] {
            wi += 1;
            ci += 1;
        }
        else if max_deletions > 0 && wi + 1 < wlen && word[wi + 1] == candidate[ci] {
            max_deletions -= 1;
            wi += 1;
        }
        else if max_insertions > 0 && ci + 1 < clen && word[wi] == candidate[ci + 1] {
            max_insertions -= 1;
            ci += 1;
        }
        else if max_substitutions > 0 {
            max_substitutions -= 1;
            wi += 1;
            ci += 1;
        }
        else {
            return (false, 0);
        }
    }
    
    let remaining_word = (wlen - wi) as u16;
    let remaining_candidate = (clen - ci) as u16;
    
    if remaining_word <= max_deletions && remaining_candidate <= max_insertions {
        (
            true,
            max_deletions - remaining_word + max_insertions - remaining_candidate + max_substitutions,
        )
    } else {
        (false, 0)
    }
}