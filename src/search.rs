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
    knot_finder_grammar::KnotFinder,
};

/// A result after attempting to search for a nice grid diagram
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRecord {
    pub stabilizations: i32,
    pub vlist: DirList, // The vertlist that has been found
    pub matrix: WindingMatrix, // The associated winding matrix for the found vertlist
    pub gridstate: Permutation, // The gridstate to optimize the Alexander function
    pub perm_type: String,
    pub alexander_grading: i32,
    pub path: Option<Vec<String>>, // A list of Reidemeister moves used to find the vlist
}

/// An error after attempting to search for a nice grid diagram
#[derive(Debug, Serialize, Deserialize)]
pub enum SearchFailure {
    HitDepthLimit, // Algorithms are defined as a formula to generate a finite list of Reidemeister 
                   // moves. The length of the list is the maximum depth of a diagram. If the search
                   // progresses through to the end of that list without finding a nice diagram,
                   // this error is reported.
    ExhaustedSearchSpace, // If there are no more states to search, causing the frontier size to fall
                          // to 0, this error is reported. If the algorithm is defined as repetition
                          // of a single move, this means that the graph of all diagrams that can be
                          // reached from the starting diagram using the given move does not contain
                          // a nice diagram; interpreting the result if the algorithm is more
                          // complicated can be difficult.
    OutOfMemory, // This error is triggered if too many states are in memory at the same time, not
                 // necessarily if the memory exceeds a certain threshold
}

/// A result after searching for a nice grid diagram
#[derive(Debug, Serialize, Deserialize)]
pub struct KnotResult {
    pub knot: String, // The name of the knot (e.g. "10_100, 12n_79, etc.")
    pub starting_vertlist: DirList, // The original vertlist, from which the [`SearchRecord`] `vlist` was found
    pub search_record: Result<SearchRecord, SearchFailure>, // Either the successful result or the
                                                            // failure mode
}

/// `ParentInfo` represents the grid diagram & Reidemeister move pair that was used to find the
/// a grid diagram. Generally ParentInfo is used as a key in a hashmap, where the value is the child
/// grid diagram
#[derive(Debug)]
struct ParentInfo {
    parent: DirList,
    move_name: String,
}

/// converts a tree and a node to a Vector of strings, representing the path taken to go from the root to the given node.
/// `visited` - the search tree, where the nodes are grid diagrams and the directed edges are Reidemeister moves 
/// `node` - the node that is being pathed to. Generally a nice diagram, as it is useful to find how
/// a nice diagram is constructed for verification purposes.
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
/// * `vertlists` - a set containing all vertlists that should be used as a starting state for the
/// search. This set will often contain only one element, the vertlist for which a nice diagram is
/// wanted, although it can also be used to resume a search that has already discovered a number of
/// states.
/// * `knot_finder` - the algorithm to use when progressing the knot frontier. Refer to
/// [`LoggingType`] for more information.
/// * `logging` - the level of logging that should be sent to STDOUT when completing the search.
/// * `max_knots` - the maximum number of knots that can be stored at the same time. Knots are the
/// most important factor in this programs memory usage, so assignment of this argument can be used
/// to prevent out of memory errors.
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
        // ctrlc::set_handler(move || ());
        previous_frontier_size = current_states.len();
        if let Some(max_knots) = max_knots {
            if previous_frontier_size as i32 > max_knots {
                return Err(SearchFailure::OutOfMemory);
            }
        }

        // Check if any diagrams are nice, in which case the algorithm can halt and any nice
        // diagram can be returned.
        if let Some((dirlist, mut record)) = current_states
            .par_iter()
            .filter_map(|a| try_permutations(&a).map(|b| (a, b)))
            .find_any(|_| true)
        {
            // Find a path from the starting state to the nice diagram
            let path = reconstruct_path(&visited_states, dirlist.clone());
            record.path = Some(path);
            return Ok(record);
        }

        if dedup {

            // candidates stores tuples containing:
            // DirList - the current state
            // DirList - the parent state, which was used to find the current state
            // String - the name of the move that was used to go from parent to child
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

        // Log the current state to STDOUT after each iteration of the algorithm
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

    // The algorithm has run to completion. If no errors or successful results have been returned,
    // the algorithm must have hit it's depth limit.
    Err(SearchFailure::HitDepthLimit)
}

/// Prints the state of the search, to be used after each iteration of the search algorithm.
/// Takes in relevant information about the 1) state and 2) the user-selected logging mode, and
/// prints the associated logs to STDOUT.
/// `current_states` - the griddiagrams that are currently being considered during the search.
/// `iteration` - the number of times the selected algorithm has been iterated
/// `previous_states_len` - the number of griddiagrams in consideration in the previous state of the
/// search. Used to calculate the percentage change of active griddiagrams.
/// `single_line` - if outputs should be printed to one line, modifying it as the algorithm
/// progresses, or to multiple lines, allowing the user to see the history of the search. Corresponds
/// to the difference between CRLF versus CR.
/// `move_name` - the Reidemeister move that is being actively used to get from the previous state
/// to the current state.
/// `dedup` - if the algorithm is 'deduplicating' the states, ensuring that the current frontier
/// does not have any elements that have already been visited. 
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

    // Visualizes the percent change of the number of states
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
        print!("\r"); // Moves to the start of the line without moving down, so that the cursor is
                      // in the position to rewrite the line that has just been written.
        let _ = io::stdout().flush();
    } else {
        print!("\n"); // Auto flushes
    }
}
