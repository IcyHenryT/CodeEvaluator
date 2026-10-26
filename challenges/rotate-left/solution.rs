pub fn rotate_left<T>(mut values: Vec<T>, amount: usize) -> Vec<T> {
    if !values.is_empty() {
        let amount = amount % values.len();
        values.rotate_left(amount);
    }

    values
}
