use crate::player::{is_valid_team, measure_imbalance, Player};

pub fn rebalance_teams(teams: &mut [Vec<Player>]) {
    let max_iterations = 10;

    for _iter in 0..max_iterations {
        let mut improved_this_round = false;
        let base_imbalance = measure_imbalance(teams);

        for i in 0..teams.len() {
            for j in (i+1)..teams.len() {
                for p1_idx in 0..teams[i].len() {
                    for p2_idx in 0..teams[j].len() {
                        let p1 = teams[i][p1_idx].clone();
                        let p2 = teams[j][p2_idx].clone();

                        teams[i][p1_idx] = p2.clone();
                        teams[j][p2_idx] = p1.clone();

                        if is_valid_team(&teams[i]) && is_valid_team(&teams[j]) {
                            let new_imbalance = measure_imbalance(teams);
                            if new_imbalance < base_imbalance {
                                improved_this_round = true;
                            } else {
                                teams[i][p1_idx] = p1;
                                teams[j][p2_idx] = p2;
                            }
                        } else {
                            teams[i][p1_idx] = p1;
                            teams[j][p2_idx] = p2;
                        }
                    }
                }
            }
        }

        if !improved_this_round {
            break;
        }
    }
}