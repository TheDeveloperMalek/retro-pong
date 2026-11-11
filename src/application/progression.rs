use crate::persistence::{Difficulty, HighScore, SaveData, SavedMatchType};
use chrono::Utc;

pub fn record_match(data: &mut SaveData, player_score: u32, opponent_score: u32, difficulty: Difficulty, won: bool, match_type: SavedMatchType) {
    data.schema_version = 1;
    let previous_best = data.high_scores.iter().map(|record| record.player_score).max().unwrap_or(0);
    let personal_record = data.high_scores.is_empty() || player_score > previous_best;
    if personal_record {
        for record in &mut data.high_scores { record.personal_record = false; }
    }
    data.high_scores.push(HighScore { player_score, opponent_score, difficulty, date: Utc::now().format("%Y-%m-%d").to_string(), won, personal_record, match_type });
    data.high_scores.sort_by(|a, b| b.player_score.cmp(&a.player_score).then_with(|| a.opponent_score.cmp(&b.opponent_score))
        .then_with(|| b.date.cmp(&a.date)));
    data.high_scores.truncate(1000);

    let progress = &mut data.progress;
    progress.total_matches += 1;
    progress.points_scored += player_score;
    if won {
        progress.wins += 1;
        progress.current_win_streak += 1;
        progress.best_win_streak = progress.best_win_streak.max(progress.current_win_streak);
    } else {
        progress.losses += 1;
        progress.current_win_streak = 0;
    }
    let solo_ai_win = won && match_type == SavedMatchType::SoloAi;
    let achievement_conditions = [
        ("first_match", progress.total_matches >= 1),
        ("first_victory", progress.wins >= 1),
        ("five_wins", progress.wins >= 5),
        ("streak_three", progress.best_win_streak >= 3),
        ("shutout", won && opponent_score == 0),
        
        ("hard_win", solo_ai_win && matches!(difficulty, Difficulty::Hard | Difficulty::Expert)),
        ("expert_win", solo_ai_win && difficulty == Difficulty::Expert),
        ("score_twenty", player_score >= 20),
    ];
    for (id, condition) in achievement_conditions {
        unlock(progress, id, condition);
    }

    
    
    progress.unlocked_paddle_skins = (0..13).collect();
    progress.unlocked_ball_skins = (0..13).collect();
    data.settings.paddle_skin = data.settings.paddle_skin.min(12);
    data.settings.ball_skin = data.settings.ball_skin.min(12);
}

fn unlock(progress: &mut crate::persistence::Progress, id: &str, condition: bool) -> bool {
    if condition && !progress.achievements.iter().any(|item| item == id) {
        progress.achievements.push(id.to_string());
        true
    } else {
        false
    }
}

pub fn achievement_name(id: &str) -> &'static str {
    match id {
        "first_match" => "INSERT COIN // PLAY FIRST MATCH",
        "first_victory" => "ARCADE ROOKIE // FIRST WIN",
        "five_wins" => "CONTENDER // FIVE WINS",
        "streak_three" => "ON FIRE // THREE WIN STREAK",
        "shutout" => "PERFECT ROUND // NO POINTS CONCEDED",
        "hard_win" => "VETERAN // DEFEAT HARD AI",
        "expert_win" => "LEGEND // DEFEAT EXPERT AI",
        "score_twenty" => "POINT MACHINE // SCORE 20",
        _ => "UNKNOWN ACHIEVEMENT",
    }
}


pub fn achievement_requirement(id: &str) -> &'static str {
    match id {
        "first_match" => "PLAY 1 MATCH",
        "first_victory" => "WIN 1 MATCH",
        "five_wins" => "WIN 5 MATCHES",
        "streak_three" => "WIN 3 MATCHES IN A ROW",
        "shutout" => "WIN A MATCH WITHOUT CONCEDING",
        "hard_win" => "BEAT HARD OR EXPERT AI",
        "expert_win" => "BEAT EXPERT AI",
        "score_twenty" => "SCORE 20 POINTS IN ONE MATCH",
        _ => "UNKNOWN REQUIREMENT",
    }
}

pub const ACHIEVEMENT_IDS: [&str; 8] = [
    "first_match", "first_victory", "five_wins", "streak_three",
    "shutout", "hard_win", "expert_win", "score_twenty",
];


