use submission::is_palindrome;

#[test]
fn empty_and_ignored_only_inputs_are_palindromes() {
    assert!(is_palindrome(""));
    assert!(is_palindrome("  ,.!—"));
}

#[test]
fn accepts_single_character_and_even_length_palindromes() {
    assert!(is_palindrome("x"));
    assert!(is_palindrome("noon"));
}

#[test]
fn ignores_case_spacing_and_punctuation() {
    assert!(is_palindrome("A man, a plan, a canal: Panama!"));
    assert!(is_palindrome("No 'x' in Nixon"));
}

#[test]
fn supports_unicode_alphanumeric_characters() {
    assert!(is_palindrome("Été"));
    assert!(is_palindrome("あ い あ"));
}

#[test]
fn rejects_non_palindromes() {
    assert!(!is_palindrome("palindrome"));
    assert!(!is_palindrome("almost, tsomla!x"));
}

#[test]
fn digits_participate_in_comparison() {
    assert!(is_palindrome("12-3-21"));
    assert!(!is_palindrome("12-3-41"));
}
