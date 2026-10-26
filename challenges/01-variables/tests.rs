use submission::add_fee;

#[test]
fn adds_the_fee_to_negative_zero_and_positive_prices() {
    let cases = [
        (-10_000, -9_998),
        (-5, -3),
        (-2, 0),
        (-1, 1),
        (0, 2),
        (1, 3),
        (8, 10),
        (10_000, 10_002),
    ];

    for (price, expected) in cases {
        assert_eq!(add_fee(price), expected, "price = {price}");
    }
}

#[test]
fn matches_the_reference_for_every_bounded_price() {
    for price in -10_000..=10_000 {
        assert_eq!(add_fee(price), price + 2, "price = {price}");
    }
}
