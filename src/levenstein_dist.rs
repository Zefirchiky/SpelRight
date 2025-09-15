pub fn levenstein_dist(s1: &str, s2: &str) -> usize {
    let ch1: Vec<char> = s1.chars().collect();
    let ch2: Vec<char> = s2.chars().collect();
    let m = ch1.len();
    let n = ch2.len();

    let mut prev_row: Vec<usize> = (0..=n).collect();
    let mut cur_row = vec![0; n + 1];

    for i in 1..=m {
        cur_row[0] = i;

        for j in 1..=n {
            let cost = if ch1[i - 1] == ch2[j - 1] { 0 } else { 1 };
            cur_row[j] = (cur_row[j - 1] + 1) // Deletion
                .min(prev_row[j] + 1)       // Insertion
                .min(prev_row[j - 1] + cost);   // Substitution
        }

        prev_row.clone_from_slice(&cur_row);
    }

    cur_row[n]
}