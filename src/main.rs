#![feature(if_let_guard)]
mod data;
mod knot_core;
mod knot_finder_grammer;
mod reidemiester;
mod search;
mod tests;

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    data::{get_all_knot_names, get_vlist_by_name, load_knot_data},
    knot_core::{DirList, is_valid},
    search::{KnotResult, SearchFailure, manual_gridstate_finder},
};

use crate::knot_finder_grammer::{
        commute_search,
        read_to_knot_finder, stab_search,
    };

const UNSOLVED_KNOT_NAMES: [&str; 12] = [
    "12n_79", "12n_168", "13n_282", "13n_917", "13n_1279", "13n_1281", "13n_1413", "13n_1826",
    "13n_2915", "13n_3089", "13n_3904", "13n_3932",
];

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    output: Option<String>,

    /// Only used optionally in combination with the  `--knots rest`. If this value is undefined,
    /// `--knots rest` will refer to the output file if it already exists, mutating it in the
    /// process.
    #[arg(short, long)]
    input: Option<String>,

    /// TODO: unimplemented
    /// The file to use in place of command flags. Used to avoid repeatedly writing out long commands.
    #[arg(short, long)]
    config: Option<String>,

    /// Which knots to target. Value must be "all", "unsolved", knot name separated by commas
    /// (e.g. 12n_79, 12n_168, 13n_282, 13n_917), a range "<start> - <end>", a percentage of
    /// the dataset to use "<percent>%", "rest" representing the knots of the output which have
    /// not yet been solved, or a vertlist in the format [(1,0), (0,1)].
    #[arg(short, long, default_value_t = String::from("unsolved"))]
    knots: String,

    /// Options: stab, commute
    #[arg(short, long, default_value_t = String::from("stab"))]
    algorithm: String,

    /// Options: none, single, multi
    #[arg(long, default_value_t = String::from("single"))]
    logging: String,

    /// Options: positives, negative, both, neither
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

    let knot_list: Vec<_> = match args.knots.as_str() {
        "unsolved" => UNSOLVED_KNOT_NAMES
            .to_vec()
            .into_iter()
            .map(|a| (get_vlist_by_name(&a.to_string(), &csv), a.to_string()))
            .collect(),
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
            if let Some(parsed_num) = string
                .strip_suffix("%")
                .map(|a| a.parse::<i32>().ok())
                .flatten() =>
        {
            let knots = get_all_knot_names(&csv);
            knots[..(knots.len() * (parsed_num / 100) as usize)]
                .to_vec()
                .into_iter()
                .map(|name| (get_vlist_by_name(&name, &csv), name))
                .collect()
        }
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
        if matches!(logging_type, LoggingType::SingleLine) {
            println!();
        }

        match &mut search_record {
            Ok(_) => {
                if log_positives {
                    println!("Found nice knot for: {}, #{}", knot, i);
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
            Some(_) => {
                None
            }
            None => None,
        })
        .collect();

    return keys;
}

fn string_to_vertmap(text: String) -> DirList {
    let mut out: DirList = DirList(vec![]);

    let re = Regex::new(
        r#"(?x)
        \d+                             |
        [[[:alpha:]]\_]+                |
        '.+?'                           |
        ".*"+?                          |
        [=+*/%&|<>!?^~\#\-]+   |
        [\(\)\[\]\{\}.\:;,@]|
        \p{Letter}
        "#,
    )
    .unwrap();

    let mut tokens = re
        .find_iter(&text)
        .map(|capture| capture.as_str().to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .peekable();

    assert_eq!(tokens.next().as_deref(), Some("["));
    while tokens.peek().map(|s| s.as_str()) != Some("]") {
        assert_eq!(tokens.next().as_deref(), Some("("));
        let x = tokens.next().unwrap().parse::<i32>().unwrap();
        assert_eq!(tokens.next().as_deref(), Some(","));
        let o = tokens.next().unwrap().parse::<i32>().unwrap();
        assert_eq!(tokens.next().as_deref(), Some(")"));
        if tokens.peek().map(|s| s.as_str()) != Some("]") {
            assert_eq!(tokens.next().as_deref(), Some(","));
        }
        out.0.push((x, o));
    }
    assert!(is_valid(&out), "Diagram is not a valid knot");
    out
}
