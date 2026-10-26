use std::collections::HashMap;
use submission::word_frequency;

#[test]
fn empty_and_separator_only_inputs_are_empty() {
    assert_eq!(word_frequency(""), HashMap::new());
    assert_eq!(word_frequency(" -- ... \n\t"), HashMap::new());
}

#[test]
fn counts_repeated_words_case_insensitively() {
    let counts = word_frequency("Rust rust RUST safe");

    assert_eq!(counts.get("rust"), Some(&3));
    assert_eq!(counts.get("safe"), Some(&1));
    assert_eq!(counts.len(), 2);
}

#[test]
fn punctuation_and_whitespace_separate_words() {
    let counts = word_frequency("one,two;one\ntwo\tthree");

    assert_eq!(counts.get("one"), Some(&2));
    assert_eq!(counts.get("two"), Some(&2));
    assert_eq!(counts.get("three"), Some(&1));
}

#[test]
fn apostrophes_and_hyphens_are_separators() {
    let counts = word_frequency("don't stop-stop");

    assert_eq!(counts.get("don"), Some(&1));
    assert_eq!(counts.get("t"), Some(&1));
    assert_eq!(counts.get("stop"), Some(&2));
}

#[test]
fn digits_are_part_of_words() {
    let counts = word_frequency("version2 version2 2024");

    assert_eq!(counts.get("version2"), Some(&2));
    assert_eq!(counts.get("2024"), Some(&1));
}

#[test]
fn supports_unicode_words_and_case_folding() {
    let counts = word_frequency("ÉTÉ été 東京 東京");

    assert_eq!(counts.get("été"), Some(&2));
    assert_eq!(counts.get("東京"), Some(&2));
}
