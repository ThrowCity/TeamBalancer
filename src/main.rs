pub mod player;
pub mod sorter;
mod fixer;
mod balancer;

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use clap::{Parser, ArgGroup};
use log::LevelFilter;
use rand::seq::SliceRandom;
use crate::balancer::rebalance_teams;
use crate::fixer::fix_invalid_teams;
use crate::player::{get_range, shuffle, sum_elo, Player};
use crate::sorter::sort_players_into_teams;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(group(
    ArgGroup::new("verbosity")
        .args(["verbose"])
        .multiple(false)
))]
pub struct Cli {
    #[arg(value_name = "input_file")]
    pub csv_file: String,

    #[arg(short = 'v', long = "verbose", value_name = "verbose", default_value_t = false)]
    pub verbose: bool,

    #[arg(short = 'd', long = "derivative", value_name = "use_derivative_score", default_value_t = false)]
    pub use_derivative: bool,

    #[arg(short = 'r', long = "randomness", value_name = "randomness", default_value_t = false)]
    pub randomness: bool,

    #[arg(short = 'f', long = "max-fixing-iterations", value_name = "max_fixing_iterations", default_value_t = 10)]
    pub max_fixing_iterations: u32,

    #[arg(short = 'i', long = "max-iterations", value_name = "max_iterations", default_value_t = 20)]
    pub max_iterations: u32,

    #[arg(short = 'o', long = "output-file", value_name = "output")]
    pub output: Option<String>,
}

fn read(file: &str) -> String {
    let mut str = String::new();
    let mut file = OpenOptions::new().read(true).open(file).unwrap_or_else(|err| {
        println!("Error opening file: {}", err);
        std::process::exit(1);
    });
    file.read_to_string(&mut str).unwrap_or_else(|err| {
        println!("Error reading file: {}", err);
        std::process::exit(1);
    });
    str
}

fn write(file: Option<String>, teams: Vec<Vec<Player>>) {
    let mut str = String::new();
    for (index, team) in teams.iter().enumerate() {
        let elo = sum_elo(&team);
        str.push_str(format!("=== Team #{} ===\n", index + 1).as_str());
        for player in team {
            str.push_str(format!("  - {}\n", player.name).as_str());
        }
        str.push_str(format!("  ELO: {}\n", elo).as_str());
        str.push_str("\n");
    }

    let mut csv = String::new();

    for (index, team) in teams.iter().enumerate() {
        csv.push_str(format!("Team {},", index + 1).as_str());
        for player in team {
            csv.push_str(format!("{},", player.name).as_str());
        }
        csv.push_str("\n");
    }

    let filename = file.unwrap_or("teams.txt".to_string());
    let mut filepath = PathBuf::from(filename);
    filepath.set_extension("txt");

    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(filepath.clone()).unwrap_or_else(|err| {
        println!("Error writing file: {}", err);
        std::process::exit(1);
    });

    file.write_all(str.as_bytes()).unwrap_or_else(|err| {
        println!("Error writing file: {}", err);
        std::process::exit(1);
    });

    filepath.set_extension("csv");

    let mut csv_file = OpenOptions::new().write(true).create(true).truncate(true).open(filepath).unwrap_or_else(|err| {
        println!("Error writing file: {}", err);
        std::process::exit(1);
    });

    csv_file.write_all(csv.as_bytes()).unwrap_or_else(|err| {
        println!("Error writing file: {}", err);
        std::process::exit(1);
    });
}

fn main() {
    let mut rng = rand::rng();
    let args = Cli::parse();

    mvlogger::init_unformatted(std::io::stdout(), if args.verbose { LevelFilter::Debug } else { LevelFilter::Info });

    let mut players = Vec::new();

    let data = read(&args.csv_file);
    let mut lines = data.lines();
    lines.next();
    let score_index = if args.use_derivative { 20 } else { 21 };
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        let split_data = line.trim().split(",").collect::<Vec<&str>>();
        log::debug!("+ Player added: {}, Availability: {}, ELO: {}", split_data[0], split_data[3], split_data[21]);
        players.push(Player {
            name: split_data[0].to_string(),
            elo: split_data[score_index].parse().unwrap(),
            availability: split_data[3].parse().unwrap(),
        });
    }

    if args.randomness {
        players.shuffle(&mut rng);
    }

    let mut teams = sort_players_into_teams(players.clone());

    fix_invalid_teams(&mut teams, &mut rng, args.max_fixing_iterations as usize, args.randomness);

    log::debug!("=== Initial Teams ===");
    for (i, team) in teams.iter().enumerate() {
        log::debug!("Team #{} | sum = {}", i + 1, sum_elo(team));
    }

    log::info!("==> Initial balance range: {}", get_range(&teams));

    let mut range = 0;
    for iteration in 1..=args.max_iterations {
        if args.randomness {
            shuffle(&mut teams, &mut rng);
        }

        log::debug!("Balancing iteration: {}", iteration);
        rebalance_teams(&mut teams);

        log::debug!("=== Iteration {} Teams ===", iteration);
        for (i, team) in teams.iter().enumerate() {
            log::debug!("Team #{} | sum = {}", i + 1, sum_elo(team));
        }

        let new_range = get_range(&teams);
        log::info!("==> Iteration {} balance range: {}", iteration, new_range);

        if new_range == range {
            log::info!("Local minima reached, ending iterations");
            break;
        }
        range = new_range;
        if iteration == args.max_iterations {
            log::info!("Max iterations reached, ending iterations")
        }
    }

    log::debug!("=== Balanced Teams ===");
    for (i, team) in teams.iter().enumerate() {
        log::debug!("Team #{} | sum = {}", i + 1, sum_elo(team));
        for p in team {
            log::debug!("  - {} (ELO ={}, Availability={})", p.name, p.elo, p.availability);
        }
        log::debug!("");
    }

    let new_range = get_range(&teams);
    write(args.output, teams);

    log::info!("==> Final balance range: {}", new_range);
}