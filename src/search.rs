use std::{cmp::min, collections::HashSet, io::{self, Write}};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{knot_core::{try_permutations, DirList, Permutation, WindingMatrix}, reidemiester::{knot_commute, stabilize, STAB_COMBINATIONS}, LoggingType};


/// Find unique perfect grid states through commutation moves.
///   
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Initial vertical segment list.
/// n : int
///     Maximum number of commutation iterations.
///
/// Returns
/// -------
/// Optional[Dict]
///     Dictionary containing grid state information if found, None otherwise.
///     
/// Notes
/// -----
/// We use a breadth-first search approach to explore commutation space.
pub fn gridstate_finder_commute(
    vertlist: DirList,
    n: i32,
    logging: &LoggingType,
) -> Result<SearchRecord, SearchFailure> {
    gridstate_finder_commute_with_visited(HashSet::from([vertlist]), n, logging)
}

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
    ExaustedSearchSpace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnotResult {
    pub knot: String,
    pub search_record: Result<SearchRecord, SearchFailure>,
}


/// Helper function: gridstate_finder_commute that respects a global visited set.
pub fn gridstate_finder_commute_with_visited(
    vertlists: HashSet<DirList>,
    n: i32,
    logging: &LoggingType,
) -> Result<SearchRecord, SearchFailure> {
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::SingleLine);

    let mut current_states = vertlists;
    let mut previous_states = HashSet::new(); // Only keeps the last iteration
    for i in 0..n {
        if let Some(record) = current_states
            .par_iter()
            .filter_map(try_permutations)
            .find_any(|_| true)
        {
            return Ok(record);
        }

        if do_logging {
            gridstate_log(&current_states, i, previous_states.len(), single_line);
        }

        current_states = current_states
            .par_iter()
            .flat_map(|r| knot_commute(&r))
            .filter(|a| !current_states.contains(a) && !previous_states.contains(a))
            .collect::<HashSet<_>>();

        if current_states.is_empty() {
            return Err(SearchFailure::ExaustedSearchSpace);
        }

        previous_states.extend(current_states.clone());
    }

    Err(SearchFailure::HitDepthLimit)
}

pub fn gridstate_finder_stab(
    vertlist: DirList,
    n: i32,
    logging: &LoggingType,
) -> Result<SearchRecord, SearchFailure> {
    let mut grid_stab_combos = vec![];
    for segment in vertlist.0.clone() {
        for (index, dir) in STAB_COMBINATIONS {
            grid_stab_combos.push((segment, dir, index));
        }
    }
    let gridstates_after_stab: HashSet<_> = grid_stab_combos
        .into_iter()
        .map(|(segment, dir, index)| stabilize(vertlist.clone(), segment, dir, index))
        .collect();

    let mut result = gridstate_finder_commute_with_visited(gridstates_after_stab, n, logging);

    if let Ok(ref mut record) = result {
        record.stabilizations = 1;
    }

    result
}

fn gridstate_log(
    current_states: &HashSet<DirList>,
    iteration: i32,
    previous_states_len: usize,
    single_line: bool,
) {
    print!("{:<5}  ", iteration);
    print!("Size of the frontier: {:<10}", current_states.len());
    // print!("Size of the global: {:<10}", global_visited.len());
    let ratio = (current_states.len() as f32) / (previous_states_len as f32);
    let format_blocks = min(30, (ratio * 10.0) as usize);
    print!(
        "  [{}{}]  ",
        "▒".repeat(format_blocks),
        "-".repeat(30 - format_blocks)
    );
    print!("Ratio change: {:.2}%", 100.0 * ratio);

    if single_line {
        print!("\r");
        let _ = io::stdout().flush();
    } else {
        print!("\n"); // Auto flushes
    }
}
