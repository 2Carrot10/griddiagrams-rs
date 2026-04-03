#![feature(if_let_guard)]
mod data;
mod plotting;
mod search;
mod search_core;
mod tests;
use std::fs;

use clap::Parser;
use serde_json::{json, Map};

use crate::{
    data::{get_all_knot_names, get_vlist_by_name, load_knot_data},
    search_core::{
        c_move, gridstate_finder_commute, gridstate_finder_stab, KnotResult, SearchFailure, SearchRecord
    },
};

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
            if let Some(parsed_num) = string
                .strip_suffix("%")
                .map(|a| a.parse::<i32>().ok())
                .flatten() =>
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

    let mut results = vec![];

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

        let mut search_record = search_function(vertlist, args.depth, &logging_type);
        if matches!(logging_type, LoggingType::SingleLine) {
            println!("");
        }

        match &mut search_record {
            Ok(_) => {
                if log_positives {
                    println!("Found nice knot for: {}, #{}", knot, i);
                }
            }
            Err(SearchFailure::HitDepthLimit) => {
                if log_negatives {
                    println!("Could not find nice knot for {}, #{} (depth limit error)", knot, i);
                }
            }
            Err(SearchFailure::ExaustedSearchSpace) => {
                if log_negatives {
                    println!("Could not find nice knot for {}, #{} (search space error)", knot, i);
                }
            }
        }
        results.push(KnotResult {
            search_record,
            knot,
        });
    }

    if !args.hide_analytics && total_length != 1 {
        compute_analytics(&results);
    }
    if let Some(output_dir) = args.output_dir {
        save_results(output_dir, &results);
    }
}

fn compute_analytics(results: &Vec<KnotResult>) {
    println!("========= Analytics =========");
    let total_length = results.len();
    println!("Total: {}", total_length);

    let (positive_results, frontier_error, depth_error) = results
        .iter()
        .map(|result| match result {
            KnotResult {
                search_record: Ok(_),
                knot: _,
            } => (1, 0, 0),
            KnotResult {
                search_record: Err(SearchFailure::ExaustedSearchSpace),
                knot: _,
            } => (0, 1, 0),
            KnotResult {
                search_record: Err(SearchFailure::HitDepthLimit),
                knot: _,
            } => (0, 0, 1),
        })
        .reduce(|(x1, y1, z1), (x2, y2, z2)| (x1 + x2, y1 + y2, z1 + z2))
        .unwrap();

    println!(
        "Positive results: {}   {}%",
        positive_results,
        (100 * positive_results) / total_length
    );
    println!(
        "Negative results: {}   {}%",
        frontier_error + depth_error,
        (100 * (frontier_error + depth_error)) / total_length
    );
    println!(
        "   Depth error: {}   {}%",
        depth_error,
        (100 * depth_error) / total_length
    );
    println!(
        "   Frontier error: {}   {}%",
        frontier_error,
        (100 * frontier_error) / total_length
    );
}

fn save_results(file_name: String, results: &Vec<KnotResult>) {
    let mut ok_vec = vec![];
    let mut depth_err_vec = vec![];
    let mut space_err_vec = vec![];
    for result in results {
        match result {
            KnotResult {
                search_record: Ok(_),
                knot: _,
            } => ok_vec.push(result),
            KnotResult {
                search_record: Err(SearchFailure::HitDepthLimit),
                knot,
            } => depth_err_vec.push(knot),
            KnotResult {
                search_record: Err(SearchFailure::ExaustedSearchSpace),
                knot,
            } => space_err_vec.push(knot),
        }
    };
    let command = std::env::args().collect::<Vec<String>>().join(" ");
    let map = json!({
        "positives": ok_vec,
        "positives": ok_vec,
        "depth_error": depth_err_vec,
        "search_space_exahusted_error": space_err_vec ,
        "command": command
    });

    let _ = fs::write(file_name, &serde_json::to_string_pretty(&map).unwrap());
}
