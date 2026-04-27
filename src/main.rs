#![feature(if_let_guard)]
mod data;
mod knot_core;
mod knot_finder_grammer;
mod reidemiester;
mod search;
mod display;
mod vertlist;

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::{get_all_knot_names, get_vlist_by_name, load_knot_data},
    search::{manual_gridstate_finder, KnotResult, SearchFailure}, vertlist::string_to_vertmap,
};

use crate::knot_finder_grammer::{commute_search, read_to_knot_finder, stab_search};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version)]
#[command(about = "A tool for quickly finding nice fibered knots, based on https://github.com/paulitzlinger/griddiagrams")]
// #[command(long_about = "This program demonstrates using clap derive for a CLI with a longer description, \
// including more details and instructions.")]
struct Args {
    /// store record of search successes and failures in <file>
    #[arg(short, long, value_name = "file")]

    output: Option<String>,

    /// search knots using failures in <file>; can only be used in combination with `--knots rest`
    #[arg(short, long, value_name = "file")]
    input: Option<String>,

    /// which knots to target ("all", comma separated knot names "<start index> - <end
    /// index>", "rest" (in combination with --input), or a vertlist in the format [(1,0), (0,1)].
    #[arg(short, long, default_value_t = String::from("all"))]
    knots: String,

    /// which algorithm to use ("stabilize", "commute", or a path to an algorithm file)
    #[arg(short, long, default_value_t = String::from("stab"))]
    algorithm: String,

    /// how to format lines of logging when searching for a solution for a knot (none, single, multi)
    #[arg(long, default_value_t = String::from("single"))]
    logging: String,

    /// which results to display (positives, negatives, both, neither)
    #[arg(long, default_value_t = String::from("both"))]
    result_type: String,

    #[arg(long)]
    hide_analytics: bool,

    #[arg(short = 'n', long, default_value_t = 200)]
    depth: i32,

    /// Will default to the number of cores available on the system
    #[arg(short, long)]
    threads: Option<i32>,

    #[arg(long)]
    hide_diagrams: bool,

    /// store nice diagram instead of just weather or not it exists
    #[arg(long)]
    verbose_output: bool,
}

pub enum LoggingType {
    None,
    Single,
    Multiline,
}

fn main() {
    let args = Args::parse();
    let csv = load_knot_data();
    let logging_type = match args.logging.as_str() {
        "none" => LoggingType::None,
        "multi" => LoggingType::Multiline,
        "single" => LoggingType::Single,
        _ => panic!("Could not read logging type"),
    };
    let (log_positives, log_negatives) = match args.result_type.as_str() {
        "positives" => (true, false),
        "negatives" => (false, true),
        "both" => (true, true),
        "neither" => (false, false),
        _ => panic!("Could not read result type"),
    };

    let knot_list: Vec<_> = match args.knots.as_str() {
        "all" => get_all_knot_names(&csv)
            .into_iter()
            .map(|name| (get_vlist_by_name(&name, &csv), name))
            .collect(),
        "rest" => get_rest_from_results(if let Some(input) = args.input {
            input
        } else if let Some(output) = &args.output {
            output.clone()
        } else {
            panic!("Rest requires an input or output file")
        })
        .into_iter()
        .map(|name| (get_vlist_by_name(&name, &csv), name))
        .collect(),
        string
            if let Some((Some(start), Some(end))) = string
                .split_once("-")
                .map(|(a, b)| (a.trim().parse::<i32>().ok(), b.trim().parse::<i32>().ok())) =>
        {
            get_all_knot_names(&csv)[start as usize..end as usize]
                .to_vec()
                .into_iter()
                .map(|name| (get_vlist_by_name(&name, &csv), name))
                .collect()
        }
        string if string.chars().next() == Some('[') => {
            vec![(
                string_to_vertmap(string.to_string()),
                String::from("Custom knot"),
            )]
        }
        names => names
            .split(",")
            .map(|a| a.trim().to_string())
            .map(|name| (get_vlist_by_name(&name, &csv), name))
            .collect(),
    };

    // Set rayon thread count global
    if let Some(t) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t as usize)
            .build_global()
            .unwrap();
    }

    let mut results = vec![];

    let knot_finder = match args.algorithm.as_str() {
        "stab" | "stabilize" => stab_search(args.depth),
        "commute" => commute_search(args.depth),
        filename => read_to_knot_finder(filename.to_string()),
    };

    let total_length = knot_list.len();
    for (i, (vertlist, knot)) in knot_list.into_iter().enumerate() {
        if !matches!(logging_type, LoggingType::None) {
            println!("----");
            println!("# {}: {}", i, knot);
            if !args.hide_diagrams {
                println!("{}", vertlist);
            }
        }

        let mut search_record = manual_gridstate_finder(
            HashSet::from([vertlist]),
            &logging_type,
            knot_finder.clone(),
        );
        if matches!(logging_type, LoggingType::Single) {
            println!();
        }

        match &mut search_record {
            Ok(ok) => {
                if log_positives {
                    println!("Found nice knot for {}, #{}: {:?}", knot, i, ok.vlist.0);

                    if !matches!(logging_type, LoggingType::None) {
                        if !args.hide_diagrams {
                            println!("{}", ok.vlist);
                        }
                    }
                }
            }
            Err(SearchFailure::HitDepthLimit) => {
                if log_negatives {
                    println!(
                        "Could not find nice knot for {}, #{} (depth limit error)",
                        knot, i
                    );
                }
            }
            Err(SearchFailure::ExhaustedSearchSpace) => {
                if log_negatives {
                    println!(
                        "Could not find nice knot for {}, #{} (search space error)",
                        knot, i
                    );
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
    if let Some(output_dir) = args.output {
        if args.verbose_output { 
        save_results_verbose(output_dir, &results);
        } else {
        save_results(output_dir, &results);
        }
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
                search_record: Err(SearchFailure::ExhaustedSearchSpace),
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
    let mut map: HashMap<String, String> = HashMap::new();
    for result in results {
        map.insert(
            result.knot.clone(),
            match result {
                KnotResult {
                    search_record: Ok(_),
                    knot: _,
                } => "found diagram",
                KnotResult {
                    search_record: Err(SearchFailure::HitDepthLimit),
                    knot: _,
                } => "depth error",
                KnotResult {
                    search_record: Err(SearchFailure::ExhaustedSearchSpace),
                    knot: _,
                } => "space exhausted error",
            }
            .to_string(),
        );
    }

    let _ = fs::write(file_name, &serde_json::to_string_pretty(&map).unwrap());
}

fn get_rest_from_results(file_name: String) -> Vec<String> {
    let json_string = serde_json::from_str::<serde_json::Map<_, _>>(
        str::from_utf8(&fs::read(file_name).unwrap()).unwrap(),
    )
    .unwrap();

    let keys: Vec<String> = json_string
        .keys()
        .filter_map(|key| match json_string.get(key).and_then(|v| v.as_str()) {
            Some("space exhausted error") | Some("depth error") => Some(key.clone()),
            Some(_) => None,
            None => None,
        })
        .collect();

    return keys;
}

#[allow(dead_code)]
fn save_results_verbose(file_name: String, results: &Vec<KnotResult>) {
    let mut map: HashMap<String, serde_json::Value> = HashMap::new();

    for result in results {
        map.insert(result.knot.clone(), match result {
            KnotResult {
                search_record: Ok(record),
                knot: _,
            } => 
                json!({
                    "vlist": record.vlist,
                    "winding-matrix": record.matrix,
                    "gridstate": record.gridstate
                }),
            KnotResult {
                search_record: Err(SearchFailure::HitDepthLimit),
                knot: _,
            } => serde_json::from_str("\"depth error\"").unwrap(),
            KnotResult {
                search_record: Err(SearchFailure::ExhaustedSearchSpace),
                knot: _,
            } => serde_json::from_str("\"space exhausted error\"").unwrap(),
        }
        );
    }

    let _ = fs::write(file_name, &serde_json::to_string(&map).unwrap());
}
