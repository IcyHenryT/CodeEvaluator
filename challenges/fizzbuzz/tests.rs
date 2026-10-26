use submission::fizzbuzz;

#[test]
fn zero_is_empty() {
    assert_eq!(fizzbuzz(0), Vec::<String>::new());
}

#[test]
fn values_before_first_replacement_are_numbers() {
    assert_eq!(fizzbuzz(2), vec!["1", "2"]);
}

#[test]
fn replaces_multiples_of_three_and_five() {
    assert_eq!(
        fizzbuzz(16),
        vec![
            "1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz", "11", "Fizz", "13",
            "14", "FizzBuzz", "16",
        ],
    );
}

#[test]
fn handles_multiple_common_multiples() {
    let values = fizzbuzz(30);
    assert_eq!(values[14], "FizzBuzz");
    assert_eq!(values[29], "FizzBuzz");
}
