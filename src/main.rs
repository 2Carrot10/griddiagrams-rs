#![feature(if_let_guard)]
mod data;
mod plotting;
mod search;
mod search_core;
mod tests;
use clap::Parser;

use crate::data::{get_all_knot_names, get_vlist_by_name, load_knot_data};

const UNSOLVED_KNOT_NAMES: [&str; 12] = [
    "12n_79", "12n_168", "13n_282", "13n_917", "13n_1279", "13n_1281", "13n_1413", "13n_1826",
    "13n_2915", "13n_3089", "13n_3904", "13n_3932",
];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///
    #[arg(short, long)]
    output_dir: String,

    /// Which knots to target. Value must be "all", "unsolved", knot name separated by commas, a
    /// range "<start> - <end>", a percentage of the dataset to use "<percent>%".
    /// (i.e. 12n_79, 12n_168, 13n_282, 13n_917)
    #[arg(short, long, default_value_t = String::from("unsolved"))]
    knots: String,

    /// Options: stab, commute
    #[arg(short, long, default_value_t = String::from("stab"))]
    algorithm: String,

    /// Options: none, single, multi
    #[arg(long, default_value_t = String::from("single"))]
    logging: String,

    /// Number of times to greet
    #[arg(short = 'n', long, default_value_t = 200)]
    depth: i32,

    #[arg(short, long)]
    threads: Option<i32>,

    #[arg(long)]
    hide_diagrams: bool,
}

pub enum LoggingType {
    None,
    SingleLine,
    MultiLine,
}

fn main() {
    let args = Args::parse();
    let csv = load_knot_data();
    let logging_type = match args.logging.as_str() {
        "none" => LoggingType::None,
        "multi" => LoggingType::MultiLine,
        "single" => LoggingType::SingleLine,
        _ => panic!("Could not read logging type"),
    };
    let knot_names = match args.knots.as_str() {
        "unsolved" => UNSOLVED_KNOT_NAMES
            .to_vec()
            .into_iter()
            .map(|a| a.to_string())
            .collect(),
        "all" => get_all_knot_names(&csv),
        string
            if let Some(parsed_num) =
                string.strip_suffix("%").map(|a| a.parse::<i32>().ok()).flatten() =>
        {
            let knots = get_all_knot_names(&csv);
            knots[..(knots.len() * (parsed_num / 100) as usize)].to_vec()
        }
        string
            if let Some((Some(start), Some(end))) = string
                .split_once("-")
                .map(|(a, b)| (a.trim().parse::<i32>().ok(), b.trim().parse::<i32>().ok())) =>
        {
            get_all_knot_names(&csv)[start as usize..end as usize].to_vec()
        }
        names => names.split(",").map(|a| a.trim().to_string()).collect(),
    };

    // Set rayon thread count global
    if let Some(t) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t as usize)
            .build_global()
            .unwrap();
    }

    for (i, knot) in knot_names.into_iter().enumerate() {
        let vertlist = get_vlist_by_name(knot.to_string(), &csv);
        if !matches!(logging_type, LoggingType::None) {
            println!("----");
            println!("#{}: {}", i, knot);
            if !args.hide_diagrams {
                println!("{}", vertlist);
            }
        }
        match args.algorithm.as_str() {
            "stab" => search_core::gridstate_finder_stab(vertlist, args.depth, &logging_type),
            "commute" => search_core::gridstate_finder_commute(vertlist, args.depth, &logging_type),
            _ => panic!("Could not read algorithm type"),
        };
    }
}
