pub fn leaderboard(mut scores: Vec<PlayerScore>) -> Vec<LeaderboardEntry> {
    scores.sort_by(|left, right| right.score.cmp(&left.score));

    scores
        .into_iter()
        .enumerate()
        .map(|(index, score)| LeaderboardEntry {
            rank: index + 1,
            player: score.player,
            score: score.score,
        })
        .collect()
}
