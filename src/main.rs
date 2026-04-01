#![feature(if_let_guard)]
mod data;
mod plotting;
mod search;
mod search_core;
mod tests;
use clap::Parser;

use crate::{data::{get_all_knot_names, get_vlist_by_name, load_knot_data}, search_core::{gridstate_finder_commute, gridstate_finder_stab}};

const UNSOLVED_KNOT_NAMES: [&str; 12] = [
    "12n_79", "12n_168", "13n_282", "13n_917", "13n_1279", "13n_1281", "13n_1413", "13n_1826",
    "13n_2915", "13n_3089", "13n_3904", "13n_3932",
];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///
    #[arg(short, long)]
    output_dir: Option<String>,

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

    /// Options: positives, negative, both
    #[arg(long, default_value_t = String::from("both"))]
    result_type: String,

    /// Hide analytics at the end.
    #[arg(long)]
    hide_analytics: bool,

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
    let (log_positives, log_negatives) = match args.result_type.as_str() {
        "positives" => (true, false),
        "negatives" => (false, true),
        "both" => (true, true),
        "neither" => (false, false),
        _ => panic!("Could not read result type"),
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

    let mut positive_results = 0;
    let mut negative_results = 0;

    let search_function = match args.algorithm.as_str() {
        "stab" => gridstate_finder_stab,
        "commute" => gridstate_finder_commute,
        _ => panic!("Could not read algorithm type"),
    };
    let total_length = knot_names.len();
    for (i, knot) in knot_names.into_iter().enumerate() {
        let vertlist = get_vlist_by_name(knot.to_string(), &csv);
        if !matches!(logging_type, LoggingType::None) {
            println!("----");
            println!("# {}: {}", i, knot);
            if !args.hide_diagrams {
                println!("{}", vertlist);
            }
        }

        let search_record = search_function(vertlist, args.depth, &logging_type);
        if matches!(logging_type, LoggingType::SingleLine) {
            println!("");
        }

        if let Ok(mut record) = search_record {
            positive_results += 1;
            if log_positives {
                println!("Found nice knot for: {}", knot);
                record.knot = Some(knot);
            }
        } else {
            negative_results += 1;
            if log_negatives {
                println!("Could not find nice knot for {}.", knot)
            }
        }
    }

    if !args.hide_analytics {
        println!("========= Analytics =========");
        println!("Total: {}", total_length);
        println!("Positive results: {}   {}%", positive_results, (100 * positive_results) /  total_length);
        println!("Negative results: {}   {}%", negative_results, (100 *  negative_results) / total_length);
    }
}
