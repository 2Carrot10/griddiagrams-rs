use colored::Colorize;
use plotters::style::IntoTextStyle;
use std::cmp::Reverse;
use std::{
    cmp::min,
    collections::{BinaryHeap, HashMap, HashSet},
    io::{self, Write},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::knot_core::diagram_and_state;
use crate::{
    LoggingType,
    knot_core::{
        DirList, Permutation, PermutationCloseness, TaggedDirList, WindingMatrix, try_permutations,
    },
    knot_finder_grammer::KnotFinder,
};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
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

pub fn heuristic_gridstate_finder(
    vertlists: DirList,
    logging: &LoggingType,
    mut knot_finder: KnotFinder,
) -> Result<SearchRecord, SearchFailure> {
    let vertlists = BinaryHeap::from([TaggedDirList::tag(vertlists)]);

    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::SingleLine);

    let mut previous_frontier_size;
    let mut current_states = vertlists;
    let mut visited_states: HashSet<DirList> = HashSet::new();
    let mut i = 0;

    // TODO: add deduplicate
    while let Some((knot_finding_function, move_name, dedup)) = knot_finder.next() {
        previous_frontier_size = current_states.len();

        let perm_results = current_states
            .par_iter()
            .map(|tagged_list| try_permutations(&tagged_list.dirlist));

        // let best = if let Some(current) = current_states.pop() {
        let best = if let Some(current) = current_states.pop() {
            current
        } else {
            return Err(SearchFailure::ExhaustedSearchSpace);
        };

        let next_states: Vec<_> = knot_finding_function(&best.dirlist)
            .into_iter()
            .map(|a| try_permutations(&a).map_err(|b| TaggedDirList::new(a, Some(b)))).collect();

        if let Some(a) = best.closeness {
            print!("best:\n {:?}", a.steps);
        }

        for item in next_states.iter().map(|a| {
            a.as_ref().err().map(|a| {
                let s = a.clone().closeness.unwrap().steps;
                s.to_string().on_truecolor((s * 17) as u8, 0, (s * 17) as u8)
            })
        }) {
            match item {
                Some(item) => print!(" {} ", item),
                None => print!(" {} ", "!".on_yellow().black()),
            }
        }
        println!();

        if let Some(record) = next_states.iter().filter_map(|a| a.as_ref().ok()).next() {
            let best = &record.vlist;

            let next_states = knot_finding_function(&best)
                .into_iter()
                .map(|a| try_permutations(&a).map_err(|b| TaggedDirList::new(a, Some(b))));

            for item in next_states.map(|a| {
                a.err().map(|a| {
                    let s = a.closeness.unwrap().steps;
                    s.to_string().on_truecolor((s * 17) as u8, 0, (s * 17) as u8)
                })
            }) {
                match item {
                    Some(item) => print!(" {} ", item),
                    None => print!(" {} ", "!".on_yellow().black()),
                }
            }

                println!(" / {}", best.0.len());

            return Ok(record.clone());
        }
        current_states.extend(
            next_states.into_iter()
                .filter_map(|a| a.err().clone())
                .filter(|a| !visited_states.contains(&a.dirlist))
                .collect::<Vec<_>>(),
        );

        visited_states.insert(best.dirlist.clone());

        // current_states.extend(next_states);

        // if dedup {
        //     current_states = current_states
        //         .par_iter()
        //         .flat_map(|(r)| knot_finding_function(&r))
        //         .filter(|a| !current_states.contains(a) && !visited_states.contains(a))
        //         .collect::<HashSet<_>>();

        //     if current_states.is_empty() {
        //         return Err(SearchFailure::ExhaustedSearchSpace);
        //     }

        //     visited_states.extend(current_states.clone());
        // } else {
        //     current_states = current_states
        //         .par_iter()
        //         .flat_map(|r| knot_finding_function(&r))
        //         .collect::<HashSet<_>>();
        // }
        i += 1;

        if do_logging {
            gridstate_log(
                current_states.len(),
                i,
                previous_frontier_size,
                single_line,
                &move_name,
                dedup,
                current_states.peek(),
            );
        }
    }

    let best = if let Some(current) = current_states.pop() {
        current
    } else {
        return Err(SearchFailure::ExhaustedSearchSpace);
    };
    // println!("{}", best.dirlist);

    Err(SearchFailure::HitDepthLimit)
}

pub fn gridstate_finder(
    vertlists: DirList,
    logging: &LoggingType,
    mut knot_finder: KnotFinder,
) -> Result<SearchRecord, SearchFailure> {
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::SingleLine);

    let mut previous_frontier_size;
    let mut current_states = HashSet::from([vertlists]);
    let mut visited_states = current_states.clone();
    let mut i = 0;

    while let Some((knot_finding_function, move_name, dedup)) = knot_finder.next() {
        previous_frontier_size = current_states.len();

        if let Some(record) = current_states
            .par_iter()
            .filter_map(|a| try_permutations(a).ok())
            .find_any(|_| true)
        {
            let best = &record.vlist;

            let next_states = knot_finding_function(&best).into_iter().map(|a| {
                // println!("\n{}", a );
                // diagram_and_state(a, &record.gridstate);
                try_permutations(&a).map_err(|b| TaggedDirList::new(a, Some(b)))
            });
            println!("Found:");

            for next in next_states {
                match next {
                    Ok(ok) => { 
                        println!(" {} ", "!".on_yellow().black());
                        println!("Correct adj:");
                        diagram_and_state(&ok.vlist, &ok.gridstate);
                    },
                    Err(err) => {
                        let s = err.closeness.unwrap().steps;
                        s.to_string()
                            .on_truecolor((s * 17) as u8, 0, (s * 17) as u8);
                        println!(" {} ", s);
                        println!("{}", err.dirlist);
                    },
                }
            }

            println!("Actual:");
            diagram_and_state(&record.vlist, &record.gridstate);

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
                current_states.len(),
                i,
                previous_frontier_size,
                single_line,
                &move_name,
                dedup,
                None,
            );
        }
    }

    Err(SearchFailure::HitDepthLimit)
}

fn gridstate_log(
    current_states: usize,
    iteration: i32,
    previous_states_len: usize,
    single_line: bool,
    move_name: &str,
    dedup: bool,
    best: Option<&TaggedDirList>,
) {
    print!("{:<5}  ", iteration);
    print!("Size of the frontier: {:<10}", current_states);
    let ratio = (current_states as f32) / (previous_states_len as f32);
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
    if let Some(best) = best {
        print!("    {:?}", best.closeness);
    }

    if single_line {
        print!("\r");
        let _ = io::stdout().flush();
    } else {
        print!("\n"); // Auto flushes
    }
}
