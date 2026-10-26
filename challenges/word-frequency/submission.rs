use std::collections::HashMap;

pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut frequencies = HashMap::new();

    for word in text.split(|character: char| !character.is_alphanumeric()) {
        if !word.is_empty() {
            *frequencies.entry(word.to_lowercase()).or_insert(0) += 1;
        }
    }

    frequencies
}
