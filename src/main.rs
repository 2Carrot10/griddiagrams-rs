mod plotting;
mod search;
mod search_core;
mod tests;
mod data;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    output_dir: String,

    /// Name of the person to greet
    #[arg(short, long, default_value_t = String::from("gridstate_finder_stab"))]
    algorithm: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    search_core::gridstate_finder_commute(vertlist, n);
}
