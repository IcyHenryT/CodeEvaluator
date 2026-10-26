use submission::{scores, Score};

fn score(rounds: u8, score: i32) -> Score {
    Score { rounds, score }
}

#[test]
fn returns_scores_from_completed_games() {
    let input = vec![score(9, 12), score(9, 27), score(9, 3)];

    assert_eq!(scores(input), vec![12, 27, 3]);
}

#[test]
fn ignores_games_that_have_not_reached_round_nine() {
    let input = vec![score(8, 20), score(9, 25), score(3, 40)];

    assert_eq!(scores(input), vec![25]);
}

#[test]
fn ignores_negative_scores() {
    let input = vec![score(9, -1), score(9, 0), score(9, 14)];

    assert_eq!(scores(input), vec![0, 14]);
}

#[test]
fn requires_both_conditions() {
    let input = vec![score(8, -10), score(8, 10), score(9, -10), score(9, 10)];

    assert_eq!(scores(input), vec![10]);
}

#[test]
fn preserves_the_input_order() {
    let input = vec![score(9, 30), score(9, 10), score(9, 20)];

    assert_eq!(scores(input), vec![30, 10, 20]);
}

#[test]
fn handles_an_empty_vector() {
    assert_eq!(scores(Vec::new()), Vec::<i32>::new());
}
