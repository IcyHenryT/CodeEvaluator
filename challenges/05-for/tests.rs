use submission::sum_with_for;

#[test]
fn zero_limit_returns_zero() {
    assert_eq!(sum_with_for(0), 0);
}

#[test]
fn includes_the_limit_in_the_sum() {
    let cases = [(1, 1), (2, 3), (5, 15), (10, 55), (100, 5_050)];

    for (limit, expected) in cases {
        assert_eq!(sum_with_for(limit), expected, "limit = {limit}");
    }
}

#[test]
fn handles_the_upper_test_bound() {
    assert_eq!(sum_with_for(1_000), 500_500);
}

#[test]
fn matches_the_arithmetic_reference_for_every_bounded_limit() {
    for limit in 0_u32..=1_000 {
        let expected = limit * (limit + 1) / 2;
        assert_eq!(sum_with_for(limit), expected, "limit = {limit}");
    }
}
