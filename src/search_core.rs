use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::iter;
use std::io::{self, Write};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

type Permutation = Vec<i32>;
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
    println!("{}", &vertlist);
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

pub fn h_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
    todo!()
    // let n = matrix.len();
    // let min_indices = [];
    // while min_indices.iter().any() {
    //     let singleton_index = ..
    // }
}

pub fn v_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
    todo!()
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
    let mut indexed_perm: Vec<(i32, i32)> =
        (0..vperm.len()).map(|i| (vperm[i], i as i32)).collect();
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
    return None; // TODO:

    //(DirList, String, Permutation, WindingMatrix, i32)
    // Does not attempt reversed
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
            if let Ok(h_perm) = h_type_0_permutation(matrix.clone()) {
                return Some(SearchRecord {
                    stabilizations: 0,
                    vlist: vertlist.clone(),
                    alexander_grading: a_grading(&vertlist, &matrix, &h_perm),
                    matrix: matrix.clone(),
                    gridstate: h_perm.clone(),
                    perm_type: String::from("h_type_0"),
                });
            }
        }

        if colsum >= rowsum {
            if let Ok(v_perm) = v_type_0_permutation(matrix.clone()) {
                return Some(SearchRecord {
                    stabilizations: 2,
                    vlist: vertlist.clone(),
                    alexander_grading: a_grading(&vertlist, &matrix, &v_perm),
                    matrix: matrix.clone(),
                    gridstate: v_perm.clone(),
                    perm_type: String::from("v_type_0"),
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
    threads: i32,
    logging: &LoggingType,
) -> Option<SearchRecord> {
    _gridstate_finder_commute_with_visited(vertlist, n, threads, logging, HashSet::new())
}

pub struct SearchRecord {
    stabilizations: i32,
    vlist: DirList,
    matrix: WindingMatrix,
    gridstate: Permutation,
    perm_type: String,
    alexander_grading: i32,
}

/// Helper function: gridstate_finder_commute that respects a global visited set.
pub fn _gridstate_finder_commute_with_visited(
    vertlist: DirList,
    n: i32,
    threads: i32,
    logging: &LoggingType,
    mut global_visited: HashSet<DirList>,
) -> Option<SearchRecord> {
    // Try initial state
    let do_logging = !matches!(logging, LoggingType::None);
    let single_line = matches!(logging, LoggingType::SingleLine);

    if let Some(result) = try_permutations(&vertlist) {
        return Some(result);
    }

    let mut current_states = HashSet::from([vertlist]);
    let mut previous_states_len = current_states.len();
    // let mut commutations_num = vec![];
    for i in 0..n {
        if do_logging {
            print!("{:<5}  ",i);
            print!("Size of the frontier: {:<10}", current_states.len());
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
            }
            else {
                print!("\n"); // Autoflushes
            }
        }

        previous_states_len = current_states.len();

        // This set intersection is an Abelian Monoid!
        let adjacent_states: HashSet<DirList> = current_states
            .into_par_iter()
            .map(knot_commute)
            .flatten()
            .filter(|a| !global_visited.contains(a)) // Take difference
            .collect();

        if let Some(record) = adjacent_states
            .iter()
            .map(try_permutations)
            .flatten()
            .next()
        {
            return Some(record);
        }

        // Equivalent to global_visited := global_visited U adjacent_states, destroying adjacent_states in the process.
        // TODO: get rid of clone
        global_visited.extend(adjacent_states.clone());
        current_states = adjacent_states;

        if current_states.is_empty() {
            // print!("Zero current states");
            break;
        }
    }
    return None;
}

pub fn gridstate_finder_stab(
    vertlist: DirList,
    n: i32,
    threads: i32,
    logging: &LoggingType,
) -> Option<SearchRecord> {
    let global_visited = HashSet::new();
    for segment in vertlist.0.clone() {
        for (index, dir) in STAB_COMBINATIONS {
            let stab_vertlist = stabilize(vertlist.clone(), segment, dir, index);
            // Skip if we've seen this stabilized state before
            if global_visited.contains(&stab_vertlist) {
                continue;
            }

            let result = _gridstate_finder_commute_with_visited(
                stab_vertlist,
                n,
                threads,
                logging,
                global_visited.clone(),
            );

            if let Some(mut record) = result {
                record.stabilizations = 1;
                return Some(record);
            }
        }
    }
    None
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
