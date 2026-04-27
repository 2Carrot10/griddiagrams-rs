use std::{
    cmp::min,
    collections::HashSet,
    io::{self, Write},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    LoggingType,
    knot_core::{DirList, Permutation, WindingMatrix, try_permutations},
    knot_finder_grammer::KnotFinder,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRecord {
    pub stabilizations: i32,
    pub vlist: DirList,
    pub matrix: WindingMatrix,
    pub gridstate: Permutation,
    pub perm_type: String,
    pub alexander_grading: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchFailure {
    HitDepthLimit,
    ExhaustedSearchSpace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnotResult {
    pub knot: String,
    pub search_record: Result<SearchRecord, SearchFailure>,
}

/// Searches for a nice knot based on a search algorithm
pub fn manual_gridstate_finder(
    vertlists: HashSet<DirList>,
    logging: &LoggingType,
    mut knot_finder: KnotFinder,
) -> Result<SearchRecord, SearchFailure> {
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::Single);

    let mut previous_frontier_size;
    let mut current_states = vertlists;
    let mut visited_states = current_states.clone();
    let mut i = 0;

    while let Some((knot_finding_function, move_name, dedup)) = knot_finder.next() {
        previous_frontier_size = current_states.len();

        if let Some(record) = current_states
            .par_iter()
            .filter_map(|a| try_permutations(a))
            .find_any(|_| true)
        {
            return Ok(record);
        }

        if dedup {
            current_states = current_states
                .par_iter()
                .flat_map(|r| knot_finding_function(&r))
                .filter(|a| !current_states.contains(a) && !visited_states.contains(a))
                .collect::<HashSet<_>>();

            if current_states.is_empty() {
                return Err(SearchFailure::ExhaustedSearchSpace);
            }

            visited_states.extend(current_states.clone());
        } else {
            current_states = current_states
                .par_iter()
                .flat_map(|r| knot_finding_function(&r))
                .collect::<HashSet<_>>();
        }
        i += 1;

        if do_logging {
            gridstate_log(
                &current_states,
                i,
                previous_frontier_size,
                single_line,
                &move_name,
                dedup
            );
        }
    }

    Err(SearchFailure::HitDepthLimit)
}

fn gridstate_log(
    current_states: &HashSet<DirList>,
    iteration: i32,
    previous_states_len: usize,
    single_line: bool,
    move_name: &str,
    dedup: bool,
) {
    print!("{:<5}  ", iteration);
    print!("Size of the frontier: {:<10}", current_states.len());
    let ratio = (current_states.len() as f32) / (previous_states_len as f32);
    let format_blocks = min(30, (ratio * 10.0) as usize);
    print!(
        "  [{}{}]  ",
        "▒".repeat(format_blocks),
        "-".repeat(30 - format_blocks)
    );
    print!("Ratio change: {:.2}%", 100.0 * ratio);
    print!("    {} {}", move_name, if dedup { "de-duplicate" } else { "preserve-all" });

    if single_line {
        print!("\r");
        let _ = io::stdout().flush();
    } else {
        print!("\n"); // Auto flushes
    }
}
