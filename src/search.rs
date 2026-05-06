use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    io::{self, Write},
};
use std::collections::hash_map::Entry::Vacant;

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
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchFailure {
    HitDepthLimit,
    ExhaustedSearchSpace,
    OutOfMemory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnotResult {
    pub knot: String,
    pub starting_vertlist: DirList,
    pub search_record: Result<SearchRecord, SearchFailure>,
}

#[derive(Debug)]
struct ParentInfo {
    parent: DirList,
    move_name: String,
}

fn reconstruct_path(
    visited: &HashMap<DirList, Option<ParentInfo>>,
    mut node: DirList,
) -> Vec<String> {
    let mut moves = Vec::new();

    while let Some(Some(info)) = visited.get(&node) {
        moves.push(info.move_name.clone());
        node = info.parent.clone();
    }

    moves.reverse();
    moves
}

/// Searches for a nice knot based on a search algorithm
pub fn manual_gridstate_finder(
    vertlists: HashSet<DirList>,
    logging: &LoggingType,
    mut knot_finder: KnotFinder,
    max_knots: Option<i32>,
) -> Result<SearchRecord, SearchFailure> {
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::Single);

    let mut previous_frontier_size;
    let mut current_states = vertlists;
    let mut visited_states: HashMap<DirList, Option<ParentInfo>> = current_states
        .iter()
        .cloned()
        .map(|s| (s, None))
        .collect();
    let mut i = 0;

    while let Some((knot_finding_function, move_name, dedup)) = knot_finder.next() {
        previous_frontier_size = current_states.len();
        if let Some(max_knots) = max_knots {
            if previous_frontier_size as i32 > max_knots {
                return Err(SearchFailure::OutOfMemory);
            }
        }

        if let Some((dirlist, mut record)) = current_states
            .par_iter()
            .filter_map(|a| try_permutations(&a).map(|b| (a, b)))
            .find_any(|_| true)
        {
            let path = reconstruct_path(&visited_states, dirlist.clone());
            record.path = Some(path);
            return Ok(record);
        }

        if dedup {
            let candidates: Vec<(DirList, DirList, String)> = current_states
                .par_iter()
                .flat_map_iter(|r| {
                    knot_finding_function(r)
                        .into_iter()
                        .map(|(child, name)| (child, r.clone(), name))
                })
                .collect();

            let mut next_states = HashSet::new();

            for (child, parent, name) in candidates {
                if let Vacant(e) =
                    visited_states.entry(child.clone())
                {
                    e.insert(Some(ParentInfo {
                        parent,
                        move_name: name,
                    }));
                    next_states.insert(child);
                }
            }

            if next_states.is_empty() {
                return Err(SearchFailure::ExhaustedSearchSpace);
            }

            current_states = next_states;
        } else {
            let candidates: Vec<(DirList, DirList, String)> = current_states
                .par_iter()
                .flat_map_iter(|r| {
                    knot_finding_function(r)
                        .into_iter()
                        .map(|(child, name)| (child, r.clone(), name))
                })
                .collect();

            let mut next_states = HashSet::new();

            for (child, parent, name) in candidates {
                visited_states.entry(child.clone()).or_insert_with(|| {
                    Some(ParentInfo {
                        parent,
                        move_name: name,
                    })
                });

                next_states.insert(child);
            }

            current_states = next_states;
        }
        i += 1;

        if do_logging {
            gridstate_log(
                &current_states,
                i,
                previous_frontier_size,
                single_line,
                &move_name,
                dedup,
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
    print!(
        "    {} {}",
        move_name,
        if dedup {
            "de-duplicate"
        } else {
            "preserve-all"
        }
    );

    if single_line {
        print!("\r");
        let _ = io::stdout().flush();
    } else {
        print!("\n"); // Auto flushes
    }
}
