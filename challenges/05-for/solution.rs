pub fn sum_with_for(limit: u32) -> u32 {
    let mut total = 0;
    for number in 1..=limit {
        total += number;
    }
    total
}
