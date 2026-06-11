#![feature(if_let_guard)]
mod data;
mod knot_core;
mod knot_finder_grammar;
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
    data::{get_all_knot_names, get_vlist_by_name, load_knot_data}, search::{manual_gridstate_finder, KnotResult, SearchFailure}, vertlist::string_to_vertmap
};

use crate::knot_finder_grammar::{commute_search, read_to_knot_finder, stab_search};

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

    /// How many knots can exist in memory at the same time (used to prevent the thread from being
    /// killed due to memory usage)
    #[arg(long)]
    max_knots: Option<i32>,

    /// Will default to the number of cores available on the system
    #[arg(short, long)]
    threads: Option<i32>,

    #[arg(long)]
    hide_diagrams: bool,

    /// store nice diagram instead of just whether or not it exists
    #[arg(long)]
    verbose_output: bool,

    /// only return nice diagrams if they are the same size as the input diagram
    #[arg(long)]
    small_only: bool,
}

/// The format in which an ongoing search for a nice grid diagram should be printed to the terminal
pub enum LoggingType {
    None,
    Single,
    Multiline,
}

/// Handles and parses user input, then executes each knot using the chosen algorithm and prints
/// relevant analytics
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
        "stab" | "stabilize" => { if args.small_only {
            panic!("Stabilize search will not return small diagrams")
        } stab_search(args.depth)},
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
            HashSet::from([vertlist.clone()]),
            &logging_type,
            knot_finder.clone(),
            args.max_knots,
            args.small_only
        );
        if matches!(logging_type, LoggingType::Single) {
            println!();
        }

        match &mut search_record {
            Ok(ok) => {
                if log_positives {
                    println!("Found nice knot for {}, #{}: {:?}", knot, i, ok.vlist.0);
                    println!("Used the following path: {}", ok.path.clone().unwrap().join(", "));

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
            Err(SearchFailure::OutOfMemory) => {
                if log_negatives {
                    println!(
                        "Could not find nice knot for {}, #{} (out of memory error)",
                        knot, i
                    );
                }
            }

        }
        results.push(KnotResult {
            search_record,
            knot,
            starting_vertlist: vertlist
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

/// Following a series of searches, calculuates the totals of all possible `[KnotResult]` values.  
/// `results` - all of the results from the searches that have just been accomplished.
fn compute_analytics(results: &Vec<KnotResult>) {
    println!("========= Analytics =========");
    let total_length = results.len();
    println!("Total: {}", total_length);

    let (positive_results, frontier_error, depth_error, out_of_memory_error) = results
        .iter()
        .map(|result| match result {
            KnotResult {
                search_record: Ok(_),
                knot: _,
                starting_vertlist: _
            } => (1, 0, 0, 0),
            KnotResult {
                search_record: Err(SearchFailure::ExhaustedSearchSpace),
                knot: _,
                starting_vertlist: _
            } => (0, 1, 0, 0),
            KnotResult {
                search_record: Err(SearchFailure::HitDepthLimit),
                knot: _,
                starting_vertlist: _
            } => (0, 0, 1, 0),
            KnotResult {
                search_record: Err(SearchFailure::OutOfMemory),
                knot: _,
                starting_vertlist: _
            } => (0, 0, 0, 1),
        })
        .reduce(|(w1, x1, y1, z1), (w2, x2, y2, z2)| (w1 + w2, x1 + x2, y1 + y2, z1 + z2))
        .unwrap();

    println!(
        "Positive results: {}   {}%",
        positive_results,
        (100 * positive_results) / total_length
    );
    println!(
        "Negative results: {}   {}%",
        frontier_error + depth_error,
        (100 * (frontier_error + depth_error + out_of_memory_error)) / total_length
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
    println!(
        "   Out of memory error: {}   {}%",
        out_of_memory_error,
        (100 * out_of_memory_error) / total_length
    );
}

/// Converts the results into json, and saves them to a file. Only stores a record stating
/// either that there exists a nice diagram, or the error that caused a nice diagram to not
/// be found. This does not store what the diagram actually is, the path to get to the diagram
/// or any other values.
/// `file_name` - the file to which the results should be saved
/// `results` - the results that should be saved
fn save_results(file_name: String, results: &Vec<KnotResult>) {
    let mut map: HashMap<String, String> = HashMap::new();
    for result in results {
        map.insert(
            result.knot.clone(),
            match result {
                KnotResult {
                    search_record: Ok(_),
                    knot: _,
                    starting_vertlist: _
                } => "found diagram",
                KnotResult {
                    search_record: Err(SearchFailure::HitDepthLimit),
                    knot: _,
                    starting_vertlist: _
                } => "depth error",
                KnotResult {
                    search_record: Err(SearchFailure::ExhaustedSearchSpace),
                    knot: _,
                    starting_vertlist: _
                } => "space exhausted error",
                KnotResult {
                    search_record: Err(SearchFailure::OutOfMemory),
                    knot: _,
                    starting_vertlist: _
                } => "out of memory",
            }
            .to_string(),
        );
    }

    let _ = fs::write(file_name, &serde_json::to_string_pretty(&map).unwrap());
}

/// Obtains a list of knot names that have not been solved, based on an output file
/// Used to attempt to solve only the knots that do not yet have a solution.
fn get_rest_from_results(file_name: String) -> Vec<String> {
    let json_string = serde_json::from_str::<serde_json::Map<_, _>>(
        str::from_utf8(&fs::read(file_name).unwrap()).unwrap(),
    )
    .unwrap();

    let keys: Vec<String> = json_string
        .keys()
        .filter_map(|key| match json_string.get(key).and_then(|v| v.as_str()) {
            Some("space exhausted error") | Some("depth error") | Some("out of memory error") => Some(key.clone()),
            Some(_) => None,
            None => None,
        })
        .collect();

    return keys;
}

/// Converts the results into json, and saves them to a file. Similar to the `[save_results]`
/// function, but stores various important values in the case of a success: startingvertlist, ending
/// vertlist, the series of moves necessary to get from the start to the end. 
/// `file_name` - the file to which the results should be saved
/// `results` - the results that should be saved
#[allow(dead_code)]
fn save_results_verbose(file_name: String, results: &Vec<KnotResult>) {
    let mut map: HashMap<String, serde_json::Value> = HashMap::new();

    for result in results {
        map.insert(result.knot.clone(), match result {
            KnotResult {
                search_record: Ok(record),
                knot: _,
                starting_vertlist
            } => 
                json!({
                    "starting_vlist": starting_vertlist,
                    "ending_vlist": record.vlist,
                    "path": record.path
                }),
            KnotResult {
                search_record: Err(SearchFailure::HitDepthLimit),
                knot: _,
                starting_vertlist: _
            } => serde_json::from_str("\"depth error\"").unwrap(),
            KnotResult {
                search_record: Err(SearchFailure::ExhaustedSearchSpace),
                knot: _,
                starting_vertlist: _
            } => serde_json::from_str("\"space exhausted error\"").unwrap(),
            KnotResult {
                search_record: Err(SearchFailure::OutOfMemory),
                knot: _,
                starting_vertlist: _
            } => serde_json::from_str("\"out of memory error\"").unwrap(),
        }
        );
    }

    let _ = fs::write(file_name, &serde_json::to_string(&map).unwrap());
}
