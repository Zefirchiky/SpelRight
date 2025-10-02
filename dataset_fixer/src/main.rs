use std::fs;

fn main() {
    let file1 = "C:/dev/tools/basic-spellchecker/words_alpha.txt";
    let file2 = "C:/dev/tools/basic-spellchecker/words.txt";
    let dataset = fs::read_to_string(file1).unwrap();
    let mut dataset = dataset
        .lines()
        .filter_map(|l| {
            let word = l.trim().to_lowercase();
            if word.len() == 0usize {
                None
            } else {
                Some(word.to_string())
            }
        })
        .collect::<Vec<String>>();
    
    dataset.sort_by(|w1, w2| w1.len().cmp(&w2.len()).then(w1.cmp(w2)));

    let mut formatted_output = String::new();
    let mut previous_len = 0;

    for word in dataset {
        let current_len = word.len();
        
        // If the length changes AND it's not the very first word
        if current_len != previous_len && previous_len != 0 {
            formatted_output.push_str("\n");
        }
        
        formatted_output.push_str(&word);
        formatted_output.push('\n'); // Insert single newline after the word
        
        previous_len = current_len;
    }

    fs::write(file2, formatted_output).unwrap();
}