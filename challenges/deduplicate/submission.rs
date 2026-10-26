pub fn deduplicate<T: PartialEq>(values: Vec<T>) -> Vec<T> {
    let mut unique = Vec::new();

    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }

    unique
}
