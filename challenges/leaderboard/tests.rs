use submission::{leaderboard, LeaderboardEntry, PlayerScore};

fn score(player: &str, score: i64) -> PlayerScore {
    PlayerScore {
        player: player.to_owned(),
        score,
    }
}

fn entry(rank: usize, player: &str, score: i64) -> LeaderboardEntry {
    LeaderboardEntry {
        rank,
        player: player.to_owned(),
        score,
    }
}

#[test]
fn sorts_scores_from_highest_to_lowest_and_assigns_ranks() {
    let scores = vec![score("Ada", 42), score("Grace", 99), score("Linus", 70)];

    assert_eq!(
        leaderboard(scores),
        vec![
            entry(1, "Grace", 99),
            entry(2, "Linus", 70),
            entry(3, "Ada", 42)
        ]
    );
}

#[test]
fn equal_scores_keep_their_input_order() {
    let scores = vec![score("first", 50), score("leader", 80), score("second", 50)];

    assert_eq!(
        leaderboard(scores),
        vec![
            entry(1, "leader", 80),
            entry(2, "first", 50),
            entry(3, "second", 50)
        ]
    );
}

#[test]
fn handles_empty_and_single_player_leaderboards() {
    assert_eq!(leaderboard(Vec::new()), Vec::<LeaderboardEntry>::new());
    assert_eq!(
        leaderboard(vec![score("solo", 7)]),
        vec![entry(1, "solo", 7)]
    );
}

#[test]
fn supports_zero_and_negative_scores() {
    let scores = vec![
        score("low", i64::MIN),
        score("zero", 0),
        score("negative", -1),
    ];

    assert_eq!(
        leaderboard(scores),
        vec![
            entry(1, "zero", 0),
            entry(2, "negative", -1),
            entry(3, "low", i64::MIN),
        ]
    );
}

#[test]
fn supports_the_largest_score_without_subtraction_overflow() {
    let scores = vec![score("minimum", i64::MIN), score("maximum", i64::MAX)];

    assert_eq!(
        leaderboard(scores),
        vec![entry(1, "maximum", i64::MAX), entry(2, "minimum", i64::MIN)]
    );
}
