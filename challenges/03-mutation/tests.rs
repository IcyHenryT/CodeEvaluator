use submission::increment_twice;

#[test]
fn increments_negative_zero_and_positive_values_twice() {
    let cases = [
        (-10_000, -9_998),
        (-5, -3),
        (-2, 0),
        (-1, 1),
        (0, 2),
        (1, 3),
        (10, 12),
        (10_000, 10_002),
    ];

    for (start, expected) in cases {
        assert_eq!(increment_twice(start), expected, "start = {start}");
    }
}

#[test]
fn matches_the_reference_for_every_bounded_start() {
    for start in -10_000..=10_000 {
        assert_eq!(increment_twice(start), start + 2, "start = {start}");
    }
}
