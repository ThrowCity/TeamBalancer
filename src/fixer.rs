use rand::Rng;
use crate::player::{is_valid_team, shuffle, Player};

pub fn fix_invalid_teams(teams: &mut [Vec<Player>], rng: &mut impl Rng, max_passes: usize, use_rng: bool) {
    log::info!("Fixing stage started");
    for pass in 0..max_passes {
        if use_rng { shuffle(teams, rng); }

        log::debug!("Starting fixing pass #{}", pass + 1);

        let mut fixed_any = false;

        for i in 0..teams.len() {
            if !is_valid_team(&teams[i]) {
                log::debug!("Found invalid team");
                let mut found_fix = false;

                for j in 0..teams.len() {
                    if j == i {
                        continue;
                    }

                    for p1_idx in 0..teams[i].len() {
                        for p2_idx in 0..teams[j].len() {
                            let p1 = teams[i][p1_idx].clone();
                            let p2 = teams[j][p2_idx].clone();

                            teams[i][p1_idx] = p2.clone();
                            teams[j][p2_idx] = p1.clone();

                            if is_valid_team(&teams[i]) && is_valid_team(&teams[j]) {
                                log::debug!("Fixed invalid team");
                                found_fix = true;
                                fixed_any = true;
                                break;
                            } else {
                                teams[i][p1_idx] = p1;
                                teams[j][p2_idx] = p2;
                            }
                        }
                        if found_fix { break; }
                    }
                    if found_fix { break; }
                }
            }
        }

        if !fixed_any {
            log::debug!("No improvements in fixing pass #{}, finishing fixing stage", pass + 1);
            break;
        }
    }

    log::info!("Fixing stage complete. Checking final validity...");
    let invalid_count = teams.iter().filter(|t| !is_valid_team(t)).count();
    if invalid_count > 0 {
        log::info!("WARNING: {} team(s) remain invalid after {} fixing passes", invalid_count, max_passes);
    }
}