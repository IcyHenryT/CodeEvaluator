#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerScore {
    pub player: String,
    pub score: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub player: String,
    pub score: i64,
}
