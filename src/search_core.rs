use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::Display;
use std::io::{self, Write};
use std::iter;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use serde::{Deserialize, Deserializer};

use crate::LoggingType;

pub type GridNotation = Vec<Vec<i32>>;

#[derive(Debug, Clone)]
pub struct GridNotationContainer(pub GridNotation);
pub type GridList = Vec<i32>;
impl<'de> Deserialize<'de> for GridNotationContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let s = s.trim();

        if !s.starts_with('[') || !s.ends_with(']') {
            panic!("missing outer brackets");
        }

        let inner = &s[1..s.len() - 1];

        let mut result = Vec::new();

        for chunk in inner.split("];") {
            let chunk = chunk.trim().trim_start_matches('[').trim_end_matches(']');

            if chunk.is_empty() {
                continue;
            }

            let mut row = Vec::new();

            for num in chunk.split(';') {
                let val = num
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| panic!("missing outer brackets"))?;
                row.push(val);
            }

            result.push(row);
        }

        Ok(GridNotationContainer(result))
    }
}

// Either VertList or HorzList
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DirList(Vec<(i32, i32)>);

pub enum StabDir {
    NorthWest,
    SouthWest,
    NorthEast,
    SouthEast,
}

pub enum Dir {
    Horz,
    Vert,
}

pub const STAB_COMBINATIONS: [(usize, StabDir); 8] = [
    (0, StabDir::NorthWest),
    (1, StabDir::NorthWest),
    (0, StabDir::NorthEast),
    (1, StabDir::NorthEast),
    (0, StabDir::SouthWest),
    (1, StabDir::SouthWest),
    (0, StabDir::SouthEast),
    (1, StabDir::SouthEast),
];

impl StabDir {
    fn is_north(&self) -> bool {
        match self {
            StabDir::NorthWest => true,
            StabDir::NorthEast => true,
            StabDir::SouthWest => false,
            StabDir::SouthEast => false,
        }
    }

    fn is_west(&self) -> bool {
        match self {
            StabDir::NorthWest => true,
            StabDir::NorthEast => false,
            StabDir::SouthWest => true,
            StabDir::SouthEast => false,
        }
    }
}

type Permutation = Vec<usize>;
type WindingMatrix = Vec<Vec<i32>>;

pub fn gridnotation_to_gridlist(mut gridnotation: GridNotation) -> GridList {
    if gridnotation.len() == 0 {
        panic!("Grid notation cannot be empty");
    }

    let mut temp = vec![gridnotation[0][1]];
    let mut current_tuple = gridnotation[0].clone();
    while temp.len() < gridnotation.len() {
        if temp.len() % 2 == 1 {
            if gridnotation.len() == 0 {
                panic!("Invalid grid notation: no matching segment found");
            }

            for segment in &mut gridnotation {
                // Negative indexing
                if segment[1] == temp[temp.len() - 1] && segment[0] != current_tuple[0] {
                    temp.push(segment[0].clone());
                    current_tuple = segment.clone();
                    break;
                }
            }
        } else {
            if gridnotation.len() == 0 {
                panic!("Invalid grid notation: no matching segment found");
            }

            for segment in &mut gridnotation {
                if segment[0] == temp[temp.len() - 1] && segment[1] != current_tuple[1] {
                    temp.push(segment[1]);
                    current_tuple = segment.clone();
                    break;
                }
            }
        }
    }
    return temp.into_iter().map(|x| x - 1).collect();
}

/// Convert grid list to horizontal segment list.
/// Parameter
/// ----------
/// gridlist : List[int]
///     Grid list representation.
///
/// Returns
/// -------
/// List[Tuple[int, int]]
///     List of tuples representing oriented horizontal segments.    
pub fn hlist(gridlist: GridList) -> DirList {
    let mut extended_grid = gridlist.clone();
    extended_grid.push(gridlist[0]);
    extended_grid.push(gridlist[1]);

    let n = extended_grid.len();
    let mut x = n + 1;
    let mut hsegments = vec![None; 2 * n + 1];
    hsegments[x] = Some((extended_grid[1], extended_grid[3]));

    for i in (3..extended_grid.len() - 2).step_by(2) {
        x = x + (extended_grid[i + 1] as usize - extended_grid[i - 1] as usize);
        if 0 <= x && x < hsegments.len() {
            hsegments[x] = Some((extended_grid[i], extended_grid[i + 2]));
        } else {
            panic!("Calculated index is out of bounds!");
        }
    }

    // Filter out None values and return
    DirList(hsegments.into_iter().flatten().collect())
}

/// Convert grid list to vertical segment list.
///
/// Parameters
/// ----------
/// gridlist : List[int]
///     Grid list representation.
///
/// Returns
/// -------
/// List[Tuple[int, int]]
///     List of tuples representing oriented vertical segments.
pub fn vlist(gridlist: GridList) -> DirList {
    let mut extended_grid = gridlist.clone();
    extended_grid.push(gridlist[0]);

    let n = extended_grid.len();

    let mut x = n as i32 + 1;
    let mut vsegments = vec![None; 2 * n + 1];
    vsegments[x as usize] = Some((extended_grid[0], extended_grid[2]));

    for i in (2..extended_grid.len() - 2).step_by(2) {
        x = x + (extended_grid[i + 1] - extended_grid[i - 1]);
        if (x as usize) < vsegments.len() {
            vsegments[x as usize] = Some((extended_grid[i], extended_grid[i + 2]));
        } else {
            panic!("Calculated index is out of bounds!");
        }
    }

    // Filter out None values and return
    DirList(vsegments.into_iter().flatten().collect())
}

/// Convert vertical segment list to horizontal segment list.
///
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Vertical segment list.
///
/// Returns
/// -------
/// List[Tuple[int, int]]
///     Horizontal segment list.
pub fn v_to_h(vertlist: DirList) -> DirList {
    let n = vertlist.0.len();
    let mut horzlist = vec![];
    for i in 0..n as i32 {
        let mut segment_indicies: (i32, i32) = (-1, -1);
        // TODO: possibly add check (value can get rewritten)
        for j in 0..n {
            if vertlist.0[j as usize].0 == i {
                segment_indicies.0 = j as i32;
            } else if vertlist.0[j as usize].1 == i {
                segment_indicies.1 = j as i32;
            }
        }

        horzlist.push(segment_indicies);
    }
    DirList(horzlist)
}

// The two functions are equivalent
pub fn h_to_v(horzlist: DirList) -> DirList {
    v_to_h(horzlist)
}

pub fn can_commute(t1: (i32, i32), t2: (i32, i32)) -> bool {
    let (a, b) = t1;
    let (c, d) = t2;

    let max1 = max(a, b);
    let min1 = min(a, b);
    let max2 = max(c, d);
    let min2 = min(c, d);

    (max1 <= min2)
        || (min1 >= max2)
        || (max1 >= max2 && min1 <= min2)
        || (max2 >= max1 && min2 <= min1)
}

pub fn c_move(input_list: DirList) -> Vec<DirList> {
    let mut result = vec![];
    let mut seen = HashSet::new();

    let n = input_list.0.len();

    for i in 0..(n - 1) {
        if can_commute(input_list.0[i], input_list.0[i + 1]) {
            let mut swapped_list = input_list.clone();
            let a = swapped_list.0[i + 1];
            let b = swapped_list.0[i];
            swapped_list.0[i] = a;
            swapped_list.0[i + 1] = b;

            if !seen.contains(&swapped_list.0) {
                seen.insert(swapped_list.0.clone());
                result.push(swapped_list.clone());
            }
        }
    }

    // Try wrap-around commutation
    if can_commute(input_list.0[0], input_list.0[input_list.0.len() - 1]) {
        let mut swapped_list = input_list.clone();
        let a = swapped_list.0[swapped_list.0.len() - 1].clone();
        let b = swapped_list.0[0].clone();
        swapped_list.0[0] = a;
        let index = swapped_list.0.len() - 1;
        swapped_list.0[index] = b;

        if !seen.contains(&swapped_list.0) {
            seen.insert(swapped_list.0.clone());
            result.push(swapped_list.clone());
        }
    }
    result
}

// fn destab_move(input_list: DirList) -> Vec<DirList> {
//     todo!()
// }

// fn can_destab(t1: (i32, i32), t2: (i32, i32)) -> bool {
//     todo!()
// }

pub fn knot_commute(vertlist: DirList) -> HashSet<DirList> {
    let v_commutations = c_move(vertlist.clone());
    let h_commutations = c_move(v_to_h(vertlist));
    let mut h_to_v_commutations: HashSet<DirList> =
        h_commutations.into_iter().map(h_to_v).collect();

    h_to_v_commutations.extend(v_commutations);
    h_to_v_commutations
}

/// Calculate the winding matrix for a knot grid diagram.
///
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Vertical segment list.
///
/// Returns
/// -------
/// np.ndarray
///     Winding matrix of grid diagram represneted by Vertical segment list.
///     
/// Notes
/// -----
/// The winding number increases by 1 when crossing an upward segment
/// and decreases by 1 when crossing a downward segment.
pub fn w_matrix(vertlist: DirList) -> WindingMatrix {
    let size = vertlist.0.len();
    let mut result = vec![];
    for i in 0..size {
        let mut row: Vec<i32> = vec![0];
        for j in 0..(size - 1) {
            let (tail, head) = vertlist.0[j];

            let prev = row[row.len() - 1];
            if tail <= (i as i32) && (i as i32) < head {
                row.push(prev + 1);
            } else if head <= (i as i32) && head < tail {
                row.push(prev - 1);
            } else {
                row.push(prev);
            }
        }

        result.push(row);
    }

    result
}

// Find a unique horizontal or vertical type-0 permutation (i.e. a unique row-perfect grid state) if it exists.
//
// Parameters
// ----------
// matrix : np.ndarray
//     Winding matrix.
//
// Returns
// -------
// List[int] or str
//     Permutation if unique one exists, error message otherwise.
pub fn type_0_permutation(matrix: WindingMatrix, direction: Dir) -> Result<Permutation, String> {
    let n = matrix.len();
    let type_string = match direction {
        Dir::Horz => "h-type-0",
        Dir::Vert => "v-type-0",
    };
    let mut min_indices: Vec<HashSet<usize>> = matrix
        .into_iter()
        .map(|row| {
            let min = row.iter().min().unwrap();
            row.iter()
                .enumerate()
                .filter(|(_, val)| val == &min)
                .map(|(index, _)| index)
                .collect()
        })
        .collect();
    let mut result = vec![None; n];

    while min_indices.iter().any(|s| !s.is_empty()) {
        match min_indices
            .iter()
            .enumerate()
            .filter(|(_, val)| val.len() == 1)
            .map(|(index, _)| index)
            .next()
        {
            None => {
                return Err(String::from(format!(
                    "No unique {} permutation exists.",
                    type_string
                )));
            }
            Some(first_elem) => {
                let x = min_indices[first_elem].iter().next().unwrap().clone();
                result[first_elem] = Some(x.clone());

                for s in &mut min_indices {
                    s.remove(&x);
                }

                if min_indices.is_empty() {
                    return Err(String::from(format!(
                        "No unique {} permutation exists.",
                        type_string
                    )));
                }
            }
        }
    }

    if result.is_empty() {
        return Err(String::from(
            "This should never happen, check code if it does.",
        ));
    }
    return Ok(result.into_iter().flatten().collect());
}

/// Reverse the orientation of a knot diagram.
///
/// Parameters
/// ----------
/// input_list : List[Tuple[int, int]]
///     Vertical or horizontal segment list.
///
/// Returns
/// -------
/// List[Tuple[int, int]]
///     Segment list with reversed orientation.
pub fn rev(input_list: DirList) -> DirList {
    DirList(input_list.0.into_iter().map(|(a, b)| (b, a)).collect())
}

/// Calculate the Alexander grading for a grid state.
///
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Vertical segment list.
/// matrix : np.ndarray
///     Winding matrix.
/// permutation : List[int]
///     Permutation representing the grid state.
///
/// Returns
/// -------
/// int
///     Alexander grading of the grid state.
pub fn a_grading(vertlist: &DirList, matrix: &WindingMatrix, permutation: &Permutation) -> i32 {
    let n = vertlist.0.len();
    let m = ((n - 1) / 2) as i32;

    let last_col = n - 1;

    let mut a_sum: i32 = 0;

    for (col, tpl) in vertlist.0.iter().enumerate() {
        let upper_row = tpl.0.min(tpl.1) as usize;
        let lower_row = tpl.0.max(tpl.1) as usize;

        let upper_sum: i32;
        if upper_row == 0 && col != last_col {
            upper_sum = matrix[0][col] + matrix[0][col + 1];
        } else if upper_row != 0 && col != last_col {
            upper_sum = matrix[upper_row - 1][col]
                + matrix[upper_row - 1][col + 1]
                + matrix[upper_row][col]
                + matrix[upper_row][col + 1];
        } else if upper_row == 0 && col == last_col {
            upper_sum = matrix[0][col];
        } else {
            // upper_row != 0 && col == last_col
            upper_sum = matrix[upper_row - 1][col] + matrix[upper_row][col];
        }

        let lower_sum: i32;
        if col != last_col {
            lower_sum = matrix[lower_row - 1][col]
                + matrix[lower_row - 1][col + 1]
                + matrix[lower_row][col]
                + matrix[lower_row][col + 1];
        } else {
            lower_sum = matrix[lower_row - 1][col] + matrix[lower_row][col];
        }

        a_sum += upper_sum + lower_sum;
    }

    // integer division (same semantics as Python int(...) after float)
    a_sum /= 8;

    let mut w_sum: i32 = 0;

    for (j, &i) in permutation.iter().enumerate() {
        w_sum += matrix[j][i as usize];
    }

    -w_sum + a_sum - m
}

/// Convert vertical permutation to horizontal permutation.
///
/// Parameters
/// ----------
/// vperm : List[int]
///     Vertical permutation.
///
/// Note
/// -----
/// This is only used to have all found grid states in the same format
///
/// Returns
/// -------
/// List[int]
///     Horizontal permutation.
pub fn vperm_to_hperm(vperm: Permutation) -> Permutation {
    let mut indexed_perm: Vec<(usize, usize)> = (0..vperm.len()).map(|i| (vperm[i], i)).collect();
    indexed_perm.sort_by_key(|a| a.0);
    indexed_perm.iter().map(|(_, index)| *index).collect()
}

/// Try to find a unique perfect grid state for a diagram.
///
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Vertical segment list.
///
/// Returns
/// -------
/// Optional[Tuple]
///     If found: (vertlist, type, permutation, matrix, alexander_grading)
///     If not found: None
///     
/// Notes
/// -----
/// This function tries both the original and reversed orientation,
/// checking for both horizontal and vertical perfect grid states.
pub fn try_permutations(vertlist: &DirList) -> Option<SearchRecord> {
    // try_instance_permutations Does not attempt reversed
    fn try_instance_permutations(vertlist: DirList) -> Option<SearchRecord> {
        let matrix = w_matrix(vertlist.clone());
        let rowsum = matrix
            .clone()
            .into_iter()
            .map(|a| a.into_iter().min().unwrap())
            .sum::<i32>();


        let colsum = transpose(matrix.clone())
            .into_iter()
            .map(|a| a.into_iter().min().unwrap())
            .sum::<i32>();

        if rowsum >= colsum {
            if let Ok(h_perm) = type_0_permutation(matrix.clone(), Dir::Horz) {
                return Some(SearchRecord {
                    stabilizations: 0,
                    vlist: vertlist.clone(),
                    alexander_grading: a_grading(&vertlist, &matrix, &h_perm),
                    matrix: matrix.clone(),
                    gridstate: h_perm.clone(),
                    perm_type: String::from("h_type_0"),
                    knot: None
                });
            }
        }

        if colsum >= rowsum {
            if let Ok(v_perm) = type_0_permutation(matrix.clone(), Dir::Vert) {
                return Some(SearchRecord {
                    stabilizations: 2,
                    vlist: vertlist.clone(),
                    alexander_grading: a_grading(&vertlist, &matrix, &v_perm),
                    matrix: matrix.clone(),
                    gridstate: v_perm.clone(),
                    perm_type: String::from("v_type_0"),
                    knot: None
                });
            }
        }

        None
    }

    if let Some(p) = try_instance_permutations(vertlist.clone()) {
        Some(p)
    } else if let Some(p) = try_instance_permutations(rev(vertlist.clone())) {
        Some(p)
    } else {
        None
    }
}

pub fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|row| row.into_iter()).collect();

    (0..len)
        .map(|_| iters.iter_mut().map(|row| row.next().unwrap()).collect())
        .collect()
}

/// Convert vertical list to X and O permutations.
///
/// Note
/// ----
/// This is only used for plotting
///
/// Parameters
/// ----------
/// vertlist : List[Tuple[int, int]]
///     Vertical segment list.
///
/// Returns
/// -------
/// Tuple[List[int], List[int]]
///     X and O permutations.
pub fn vlist_to_xo(vertlist: DirList) -> (Vec<i32>, Vec<i32>) {
    let n = (vertlist.0.len() as i32) - 1;
    let tempx: Vec<i32> = vertlist.0.iter().map(|a| a.0).collect();
    let tempo: Vec<i32> = vertlist.0.iter().map(|a| a.1).collect();

    let x = tempx.iter().map(|x| n - x).collect();
    let o = tempo.iter().map(|x| n - x).collect();

    return (x, o);
}

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
) -> Option<SearchRecord> {
    _gridstate_finder_commute_with_visited(HashSet::from([vertlist]), n, logging)
}

#[derive(Debug)]
pub struct SearchRecord {
    stabilizations: i32,
    vlist: DirList,
    matrix: WindingMatrix,
    gridstate: Permutation,
    perm_type: String,
    alexander_grading: i32,
    pub knot: Option<String>,
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
    /*
    if commutations_num.len() == 0 {
        print!("   Average commutations: NaN");
    } else {
        print!(
            "   Average commutations: {}",
            commutations_num.iter().sum::<f32>() / commutations_num.len() as f32
        );
    }
    */

    if single_line {
        print!("\r");
        io::stdout().flush();
    } else {
        print!("\n"); // Autoflushes
    }
}

/// Helper function: gridstate_finder_commute that respects a global visited set.
pub fn _gridstate_finder_commute_with_visited(
    vertlists: HashSet<DirList>,
    n: i32,
    logging: &LoggingType,
) -> Option<SearchRecord> {
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::SingleLine);

    let mut current_states = vertlists;
    let mut previous_states = HashSet::new(); // Only keeps the last iteration
    for i in 0..n {
        if do_logging {
            gridstate_log(&current_states, i, previous_states.len(), single_line);
        }


        current_states = current_states
            .clone()
            .into_par_iter()
            .flat_map(knot_commute)
            .filter(|a| !current_states.contains(a) && !previous_states.contains(a))
            .collect::<HashSet<_>>();

        if let Some(record) = current_states
            .par_iter()
            .filter_map(try_permutations)
            .find_any(|_| true)
        {
            if single_line {
                println!("");
            }
            return Some(record);
        }

        if current_states.is_empty() {
            if single_line {
                println!("");
                println!("Zero current states");
            }
            return None;
        }

        previous_states.extend(current_states.clone());
    }
    if single_line {
        println!("");
    }
    None
}

pub fn gridstate_finder_stab(
    vertlist: DirList,
    n: i32,
    logging: &LoggingType,
) -> Option<SearchRecord> {
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

    let result = _gridstate_finder_commute_with_visited(gridstates_after_stab, n, logging);

    if let Some(mut record) = result {
        record.stabilizations = 1;
        Some(record)
    } else {
        None
    }
}

// fn destabilize(vertlist: DirList, loc_index: i32, direction: StabDir, tuple_index: i32) -> DirList {
//     let is_north = direction.is_north();
//     let is_west = direction.is_west();
// }

pub fn stabilize(
    vertlist: DirList,
    loc: (i32, i32),
    direction: StabDir,
    tuple_index: usize,
) -> DirList {
    let is_north = direction.is_north();
    let is_west = direction.is_west();

    if !vertlist.0.contains(&loc) {
        let l0 = loc.0;
        let l1 = loc.1;
        panic!("Segment ({l0},{l1}) not in vertical list");
    }

    let k = vertlist.0.iter().position(|a| a == &loc).unwrap();
    let loc = [loc.0, loc.1];
    let mut temp: Vec<[i32; 2]> = vertlist.0.iter().map(|(a, b)| [*a, *b]).collect();

    for segment in &mut temp {
        for j in 0..segment.len() {
            if segment[j] > loc[tuple_index] || (!is_north && segment[j] >= loc[tuple_index]) {
                segment[j] += 1;
            }
        }
    }
    let insert_offset = if is_west { 1 } else { 0 };
    let remainder_offset = 1 - insert_offset;
    let segment = if is_north == (tuple_index == 0) {
        [loc[tuple_index], loc[tuple_index] + 1]
    } else {
        [loc[tuple_index] + 1, loc[tuple_index]]
    };
    temp.insert(k + insert_offset, segment);
    temp[k + remainder_offset][tuple_index] = loc[tuple_index] + if is_north { 1 } else { 0 };

    DirList(temp.iter().map(|[a, b]| (*a, *b)).collect())
}

impl Display for DirList {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horzlist = v_to_h(self.clone());
        let mut downward_lines = vec![false; self.0.len()];
        for (x, o) in horzlist.0 {
            for i in 0..(self.0.len() as i32) {
                if i == min(x, o) {
                    print!(
                        "{}",
                        if downward_lines[min(x, o) as usize] {
                            "╰"
                        } else {
                            "╭"
                        }
                    );
                    downward_lines[i as usize] = !downward_lines[i as usize];
                } else if i == max(x, o) {
                    print!(
                        "{}",
                        if downward_lines[max(x, o) as usize] {
                            "╯"
                        } else {
                            "╮"
                        }
                    );
                    downward_lines[i as usize] = !downward_lines[i as usize];
                } else {
                    if downward_lines[i as usize] {
                        print!("│");
                    } else if min(x, o) < i && i < max(x, o) {
                        print!("─");
                    } else {
                        print!("·");
                    }
                }
            }
            println!();
        }
        std::fmt::Result::Ok(())
    }
}

impl std::fmt::Debug for DirList {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horzlist = v_to_h(self.clone());
        let hlen = horzlist.0.len();
        for (x, o) in horzlist.0 {
            print!("{}", ".".repeat(min(x, o) as usize));
            print!("{}", if x < o { "○" } else { "✗" });
            print!("{}", "·".repeat(((x - o).abs() - 1) as usize));
            print!("{}", if x < o { "✗" } else { "○" });
            print!(
                "{}",
                iter::repeat("·")
                    .take(hlen - (max(x, o) - 1) as usize)
                    .collect::<String>()
            );
            println!();
        }

        std::fmt::Result::Ok(())
    }
}
