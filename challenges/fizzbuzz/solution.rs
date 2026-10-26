pub fn fizzbuzz(limit: u32) -> Vec<String> {
    (1..=limit)
        .map(|number| match (number % 3, number % 5) {
            (0, 0) => "FizzBuzz".to_owned(),
            (0, _) => "Fizz".to_owned(),
            (_, 0) => "Buzz".to_owned(),
            _ => number.to_string(),
        })
        .collect()
}
