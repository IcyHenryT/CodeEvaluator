use submission::deduplicate;

#[test]
fn removes_repeated_values_and_preserves_first_occurrence_order() {
    assert_eq!(deduplicate(vec![3, 1, 3, 2, 1, 2]), vec![3, 1, 2]);
}

#[test]
fn handles_empty_and_single_value_inputs() {
    assert_eq!(deduplicate(Vec::<i32>::new()), Vec::<i32>::new());
    assert_eq!(deduplicate(vec!["only"]), vec!["only"]);
}

#[test]
fn handles_all_duplicate_and_all_unique_inputs() {
    assert_eq!(deduplicate(vec![7, 7, 7, 7]), vec![7]);
    assert_eq!(deduplicate(vec![1, 2, 3, 4]), vec![1, 2, 3, 4]);
}

#[test]
fn supports_non_copy_values() {
    let values = vec![
        String::from("rust"),
        String::from("std"),
        String::from("rust"),
        String::new(),
        String::new(),
    ];

    assert_eq!(
        deduplicate(values),
        vec![String::from("rust"), String::from("std"), String::new()]
    );
}

#[test]
fn supports_types_that_do_not_implement_hash_or_clone() {
    #[derive(Debug, PartialEq, Eq)]
    struct Value(i32);

    assert_eq!(
        deduplicate(vec![Value(2), Value(1), Value(2)]),
        vec![Value(2), Value(1)]
    );
}
