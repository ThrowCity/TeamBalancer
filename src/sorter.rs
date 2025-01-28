use mvutils::unsafe_utils::Unsafe;
use crate::player::Player;

pub fn sort_players_into_teams(mut players: Vec<Player>) -> Vec<Vec<Player>> {
    let mut v1 = Vec::new();
    let mut v2 = Vec::new();
    let mut v3 = Vec::new();
    let mut v4 = Vec::new();

    for p in players.drain(..) {
        match p.availability {
            1 => v1.push(p),
            2 => v2.push(p),
            3 => v3.push(p),
            4 => v4.push(p),
            _ => {
                log::error!("Invalid availability value at player: {}", p.name);
                std::process::exit(1);
            },
        }
    }

    v1.sort_by_key(|p| std::cmp::Reverse(p.elo));
    v2.sort_by_key(|p| std::cmp::Reverse(p.elo));
    v3.sort_by_key(|p| std::cmp::Reverse(p.elo));
    v4.sort_by_key(|p| std::cmp::Reverse(p.elo));

    let mut teams = Vec::new();

    while v1.len() >= 5 {
        let team = v1.drain(0..5).collect();
        teams.push(team);
    }

    let take_team_of_five = |group: &mut Vec<Player>| -> Option<Vec<Player>> {
        if group.len() < 5 {
            return None;
        }
        let mut team = Vec::new();
        let front = 0;
        let mut back = group.len() - 1;
        while team.len() < 5 {
            if team.len() % 2 == 0 {
                team.push(group.remove(front));
                back = group.len().saturating_sub(1);
            } else {
                team.push(group.remove(back));
                back = group.len().saturating_sub(1);
            }
        }
        Some(team)
    };

    let v2s = unsafe { Unsafe::cast_static(&v2) };
    let v3s = unsafe { Unsafe::cast_static(&v2) };
    let v4s = unsafe { Unsafe::cast_static(&v2) };
    let take_mixed_team = |vx: &mut Vec<Player>, vy: &mut Vec<Player>| -> Option<Vec<Player>> {
        if vx.is_empty() && vy.is_empty() {
            return None;
        }
        let mut team = Vec::with_capacity(5);

        while team.len() < 5 {
            let x_elo = vx.first().map(|p| p.elo).unwrap_or(0);
            let y_elo = vy.first().map(|p| p.elo).unwrap_or(0);

            if x_elo >= y_elo {
                if vx.is_empty() {
                    if vy.is_empty() {
                        break;
                    }
                    team.push(vy.remove(0));
                } else {
                    team.push(vx.remove(0));
                }
            } else if vy.is_empty() {
                if vx.is_empty() {
                    break;
                }
                team.push(vx.remove(0));
            } else {
                team.push(vy.remove(0));
            }


            if vx.is_empty() && vy.is_empty() && team.len() < 5 {
                break;
            }
        }

        if team.len() == 5 {
            Some(team)
        } else {
            for p in team.into_iter().rev() {
                if p.availability == vx_availability(v2s, v3s, v4s, &p) {
                    match p.availability {
                        2 => vx.insert(0, p),
                        3 => vx.insert(0, p),
                        4 => vx.insert(0, p),
                        _ => {}
                    }
                } else {
                    match p.availability {
                        2 => vy.insert(0, p),
                        3 => vy.insert(0, p),
                        4 => vy.insert(0, p),
                        _ => {}
                    }
                }
            }
            None
        }
    };

    fn vx_availability(
        _v2: &[Player],
        _v3: &[Player],
        _v4: &[Player],
        p: &Player
    ) -> u8 {
        p.availability
    }

    while v2.len() + v3.len() >= 5 {
        if let Some(team) = take_mixed_team(&mut v2, &mut v3) {
            if team.len() == 5 {
                teams.push(team);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    while v3.len() + v4.len() >= 5 {
        if let Some(team) = take_mixed_team(&mut v3, &mut v4) {
            if team.len() == 5 {
                teams.push(team);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let maybe_form_single_avail_teams = |group: &mut Vec<Player>, teams: &mut Vec<Vec<Player>>| {
        while group.len() >= 5 {
            if let Some(team) = take_team_of_five(group) {
                teams.push(team);
            } else {
                let chunk = group.drain(0..5).collect();
                teams.push(chunk);
            }
        }
    };

    maybe_form_single_avail_teams(&mut v2, &mut teams);
    maybe_form_single_avail_teams(&mut v3, &mut teams);
    maybe_form_single_avail_teams(&mut v4, &mut teams);

    if v2.len() + v3.len() + v4.len() > 0 {
        let mut final_team = Vec::new();
        final_team.append(&mut v2);
        final_team.append(&mut v3);
        final_team.append(&mut v4);
        teams.push(final_team);
    }

    if !v1.is_empty() {
        log::error!("Players with `1` availability must end up as a multiple of 5");
        std::process::exit(1);
    }

    teams
}