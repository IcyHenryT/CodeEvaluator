pub fn scores(input: Vec<Score>) -> Vec<i32> {
    input
        .into_iter()
        .filter(|score| score.rounds == 9 && score.score >= 0)
        .map(|score| score.score)
        .collect()
}
