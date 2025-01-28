use rand::prelude::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub elo: u32,
    pub availability: u8,
}

pub fn sum_elo(team: &[Player]) -> u32 {
    team.iter().map(|p| p.elo).sum()
}

pub fn is_valid_team(team: &[Player]) -> bool {
    if team.len() != 5 {
        return false;
    }

    let unique_avails: std::collections::HashSet<u8> =
        team.iter().map(|p| p.availability).collect();

    if unique_avails.contains(&1) {
        return unique_avails.len() == 1;
    }

    if unique_avails.len() == 1 {
        return true;
    }

    if unique_avails.len() == 2 {
        if unique_avails.contains(&2) && unique_avails.contains(&3) {
            return true;
        }
        if unique_avails.contains(&3) && unique_avails.contains(&4) {
            return true;
        }
        return false;
    }

    false
}

pub fn measure_imbalance(teams: &[Vec<Player>]) -> u32 {
    let sums: Vec<u32> = teams.iter().map(|t| sum_elo(t)).collect();
    let min_sum = sums.iter().min().unwrap_or(&0);
    let max_sum = sums.iter().max().unwrap_or(&0);
    max_sum.saturating_sub(*min_sum)
}

pub fn get_range(teams: &[Vec<Player>]) -> u32 {
    let mut min = u32::MAX;
    let mut max = 0;
    for team in teams {
        let elo = sum_elo(team);
        if elo < min {
            min = elo;
        }
        if elo > max {
            max = elo;
        }
    }
    max - min
}

pub fn shuffle(teams: &mut [Vec<Player>], rng: &mut impl Rng) {
    teams.shuffle(rng);
    for team in teams.iter_mut() {
        team.shuffle(rng);
    }
}