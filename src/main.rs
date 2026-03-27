mod plotting;
mod search;
mod search_core;
mod tests;
mod data;
use clap::Parser;

use crate::data::{get_all_knot_names, get_vlist_by_name, load_knot_data, RAW_CSV};

const UNSOLVED_KNOT_NAMES: [&str; 12] = [
    "12n_79", "12n_168", "13n_282", "13n_917", "13n_1279", "13n_1281", "13n_1413", "13n_1826",
    "13n_2915", "13n_3089", "13n_3904", "13n_3932",
];

#[derive(Clone)]
enum KnotGroup {
    Unsolved, All, Custom(String)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 
    #[arg(short, long)]
    output_dir: String,

    /// Which knots to target. Value must be "all", "unsolved", or knot name separated by commas
    /// (such as 12n_79, 12n_168, 13n_282, 13n_917)
    #[arg(short, long, default_value_t = String::from("unsolved"))]
    target_knots: String,

    ///
    #[arg(short, long, default_value_t = String::from("gridstate_finder_stab"))]
    algorithm: String,

    /// Number of times to greet
    #[arg(short='n', long, default_value_t = 50)]
    depth: i32,
}

fn main() {
    let args = Args::parse();
    let csv = load_knot_data();
    let knot_names = match args.target_knots.as_str() {
        "unsolved" => UNSOLVED_KNOT_NAMES.to_vec().into_iter().map(|a| a.to_string()).collect(),
        "all" => get_all_knot_names(&csv),
        names => names.split(",").map(|a| a.trim().to_string()).collect(),
    };

    for knot in knot_names {
        println!("----");
        let vertlist = get_vlist_by_name(knot.to_string(), &csv);
        println!("*** {}", knot);
        println!("{}", vertlist);
        search_core::gridstate_finder_commute(vertlist, args.depth);
    }
}
