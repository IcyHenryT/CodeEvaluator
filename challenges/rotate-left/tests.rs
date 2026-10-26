use submission::rotate_left;

#[test]
fn rotates_values_to_the_left() {
    assert_eq!(rotate_left(vec![1, 2, 3, 4, 5], 2), vec![3, 4, 5, 1, 2]);
}

#[test]
fn zero_rotation_preserves_the_input() {
    assert_eq!(rotate_left(vec![1, 2, 3], 0), vec![1, 2, 3]);
}

#[test]
fn full_and_oversized_rotations_wrap_around() {
    assert_eq!(rotate_left(vec![1, 2, 3, 4], 4), vec![1, 2, 3, 4]);
    assert_eq!(rotate_left(vec![1, 2, 3, 4], 10), vec![3, 4, 1, 2]);
}

#[test]
fn handles_empty_and_single_value_inputs() {
    assert_eq!(rotate_left(Vec::<i32>::new(), usize::MAX), Vec::<i32>::new());
    assert_eq!(rotate_left(vec!["only"], usize::MAX), vec!["only"]);
}

#[test]
fn supports_non_copy_values() {
    let values = vec![String::from("a"), String::from("b"), String::from("c")];

    assert_eq!(
        rotate_left(values, 1),
        vec![String::from("b"), String::from("c"), String::from("a")]
    );
}
