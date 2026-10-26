pub fn sum_with_while(limit: u32) -> u32 {
    let mut total = 0;
    let mut next = 1;
    while next <= limit {
        total += next;
        next += 1;
    }
    total
}
