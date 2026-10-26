use submission::bigger;

#[test]
fn chooses_the_larger_input_in_either_order() {
    let cases = [
        (3, 8, 8),
        (8, 3, 8),
        (-7, -2, -2),
        (-2, -7, -2),
        (-5, 0, 0),
        (0, -5, 0),
        (0, 5, 5),
        (5, 0, 5),
        (-10_000, 10_000, 10_000),
        (10_000, -10_000, 10_000),
    ];

    for (left, right, expected) in cases {
        assert_eq!(
            bigger(left, right),
            expected,
            "left = {left}, right = {right}"
        );
    }
}

#[test]
fn equal_inputs_return_the_common_value() {
    for value in [-10_000, -4, 0, 6, 10_000] {
        assert_eq!(bigger(value, value), value, "equal inputs = {value}");
    }
}

#[test]
fn matches_the_reference_for_all_pairs_in_a_bounded_grid() {
    for left in -100..=100 {
        for right in -100..=100 {
            assert_eq!(
                bigger(left, right),
                std::cmp::max(left, right),
                "left = {left}, right = {right}"
            );
        }
    }
}
