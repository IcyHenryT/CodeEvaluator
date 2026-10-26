pub fn is_palindrome(input: &str) -> bool {
    let normalized: Vec<char> = input
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();

    normalized.iter().eq(normalized.iter().rev())
}
