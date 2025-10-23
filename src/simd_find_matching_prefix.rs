#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn find_matching_prefix_simd_sse2(word: &[u8], candidate: &[u8]) -> usize {
    use std::arch::x86_64::*;
    
    let min_len = word.len().min(candidate.len());
    
    // Process 16 bytes at a time with SSE2
    let chunks = min_len / 16;
    let mut i = 0;
    
    for chunk in 0..chunks {
        let offset = chunk * 16;
        
        // Load 16 bytes from each string
        let w = unsafe { _mm_loadu_si128(word.as_ptr().add(offset) as *const __m128i) };
        let c = unsafe { _mm_loadu_si128(candidate.as_ptr().add(offset) as *const __m128i) };
        
        // Compare for equality
        let cmp = _mm_cmpeq_epi8(w, c);
        
        // Convert comparison result to bitmask
        let mask = _mm_movemask_epi8(cmp) as u32;
        
        // If all 16 bytes match, mask will be 0xFFFF (all bits set)
        if mask != 0xFFFF {
            // Find first mismatch position using trailing ones
            i = offset + mask.trailing_ones() as usize;
            return i;
        }
        
        i = offset + 16;
    }
    
    // Handle remaining bytes with scalar comparison
    while i < min_len && word[i] == candidate[i] {
        i += 1;
    }
    
    i
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn find_matching_prefix_simd_avx2(word: &[u8], candidate: &[u8]) -> usize {
    use std::arch::x86_64::*;
    
    let min_len = word.len().min(candidate.len());
    let chunks = min_len / 32;
    let mut i = 0;
    
    for chunk in 0..chunks {
        let offset = chunk * 32;
        
        let w = unsafe { _mm256_loadu_si256(word.as_ptr().add(offset) as *const __m256i) };
        let c = unsafe { _mm256_loadu_si256(candidate.as_ptr().add(offset) as *const __m256i) };
        
        let cmp = _mm256_cmpeq_epi8(w, c);
        let mask = _mm256_movemask_epi8(cmp);
        
        if mask != -1 {  // -1 = 0xFFFFFFFF (all 32 bits set)
            i = offset + mask.trailing_ones() as usize;
            return i;
        }
        
        i = offset + 32;
    }
    
    // Handle remaining bytes
    while i < min_len && word[i] == candidate[i] {
        i += 1;
    }
    
    i
}