use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::iter;

type GridNotation = Vec<Vec<i32>>;
type GridList = Vec<i32>;

// Either VertList or HorzList
#[derive(Clone, Eq, PartialEq, Hash)]
struct DirList(Vec<(i32, i32)>);

enum StabDir {
    NorthWest,
    SouthWest,
    NorthEast,
    SouthEast,
}
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

fn main() {
    println!("Hello, world!");
}

fn gridnotation_to_gridlist(mut gridnotation: GridNotation) -> GridList {
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
fn hlist(gridlist: GridList) -> DirList {
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
fn vlist(gridlist: GridList) -> DirList {
    let mut extended_grid = gridlist.clone();
    extended_grid.push(gridlist[0]);

    let n = extended_grid.len();
    extended_grid.push(gridlist[0]);

    let mut x = n + 1;
    let mut vsegments = vec![None; 2 * n + 1];
    vsegments[x] = Some((extended_grid[1], extended_grid[2]));

    for i in (2..extended_grid.len() - 2).step_by(2) {
        x = x + (extended_grid[i + 1] as usize - extended_grid[i - 1] as usize);
        if 0 <= x && x < vsegments.len() {
            vsegments[x] = Some((extended_grid[i], extended_grid[i + 2]));
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
fn v_to_h(vertlist: DirList) -> DirList {
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
fn h_to_v(horzlist: DirList) -> DirList {
    v_to_h(horzlist)
}

fn can_commute(t1: (i32, i32), t2: (i32, i32)) -> bool {
    let (a, b) = t1;
    let (c, d) = t1;

    let max1 = max(a, b);
    let min1 = min(a, b);
    let max2 = max(c, d);
    let min2 = min(c, d);

    (max1 <= min2)
        || (min1 >= max2)
        || (max1 >= max2 && min1 <= min2)
        || (max2 >= max1 && min2 <= min1)
}

fn c_move(input_list: DirList) -> Vec<DirList> {
    let mut result = vec![];
    let mut seen = HashSet::new();

    let mut n = input_list.0.len();

    // fn add_to_result(lst: Vec<(i32, i32)>) {
    //     if seen.contains(tpl) {
    //         seen.insert(lst);
    //         result.append(lst);
    //     }
    // }

    for i in 0..n {
        if can_commute(input_list.0[i], input_list.0[i + 1]) {
            let mut swapped_list = input_list.clone();
            let a = swapped_list.0[i + 1];
            let b = swapped_list.0[i];
            swapped_list.0[i] = a;
            swapped_list.0[i + 1] = b;

            if seen.contains(&swapped_list.0) {
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

        if seen.contains(&swapped_list.0) {
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

fn knot_commute(vertlist: DirList) -> HashSet<DirList> {
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
fn w_matrix(vertlist: DirList) -> WindingMatrix {
    let size = vertlist.0.len();
    let mut result = vec![];
    for i in 0..size {
        let mut row: Vec<i32> = vec![];
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

fn h_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
    todo!()
}

fn v_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
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
fn rev(input_list: DirList) -> DirList {
    DirList(input_list.0.into_iter().map(|(a, b)| (b, a)).collect())
}

fn a_grading(vertlist: DirList, matrix: WindingMatrix, permutation: Permutation) -> i32 {
    let n = vertlist.0.len();
    let m = (n - 1) / 2;

    let mut a_sum = 0;
    for tpl in &vertlist.0 {
        let col = vertlist.0.iter().position(|r| r == tpl).unwrap();
        let upper_row = min(tpl.0, tpl.1);
        let lower_row = max(tpl.0, tpl.1);

        if upper_row == 0
            && col
                != vertlist.0
                    .iter()
                    .position(|&r| r == vertlist.0[vertlist.0.len() - 1])
                    .unwrap()
        {
            let upper_sum = matrix[0][col] + matrix[0][col + 1];
        }
    }
    todo!()
}

fn vperm_to_hperm(vperm: Permutation) -> Permutation {
    todo!()
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
fn try_permutations(
    vertlist: DirList,
) -> Option<(DirList, String, Permutation, WindingMatrix, i32)> {
    // Does not attempt reversed
    fn try_instance_permutations(
        vertlist: DirList,
    ) -> Option<(DirList, String, Permutation, WindingMatrix, i32)> {
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
                return Some((
                    vertlist.clone(),
                    String::from("h_type_0"),
                    h_perm.clone(),
                    matrix.clone(),
                    a_grading(vertlist, matrix, h_perm),
                ));
            }
        }

        if colsum >= rowsum {
            if let Ok(v_perm) = v_type_0_permutation(matrix.clone()) {
                return Some((
                    vertlist.clone(),
                    String::from("v_type_0"),
                    v_perm.clone(),
                    matrix.clone(),
                    a_grading(vertlist, matrix, v_perm),
                ));
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


fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
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
fn vlist_to_xo(vertlist: DirList) -> (Vec<i32>, Vec<i32>) {
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
fn gridstate_finder_commute<K, V>(vertlist: DirList, n: i32) -> Option<HashMap<K, V>> {
    todo!()
}

fn _gridstate_finder_commute_with_visited<K, V>(
    vertlist: DirList,
    n: i32,
) -> Option<HashMap<K, V>> {
    todo!()
}

fn gridstate_finder_stab<K, V>(vertlist: DirList, n: i32) -> Option<HashMap<K, V>> {
    todo!()
}

fn destabilize(vertlist: DirList, loc_index: i32, direction: StabDir, tuple_index: i32) -> DirList {
    let is_north = direction.is_north();
    let is_west = direction.is_west();
}

fn stabilize(vertlist: DirList, loc: (i32, i32), direction: StabDir, tuple_index: i32) -> DirList {
    let is_north = direction.is_north();
    let is_west = direction.is_west();

    if vertlist.0.contains(&loc) {
        let l0 = loc.0;
        let l1 = loc.1;
        panic!("Segment ({l0},{l1}) not in vertical list");
    }

    let k = vertlist.0.iter().position(|a| a == &loc);
    let temp: Vec<i32> = vertlist.0.iter().map(|(a, b)| [a, b]).collect();

    for segment in vertlist.0 {
        for j in temp.len() {
        }
    }

    todo!()
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
                        if downward_lines[min(x, o) as usize] {
                            "╯"
                        } else {
                            "╮"
                        }
                    );
                    downward_lines[i as usize] = !downward_lines[i as usize];
                } else {
                    if min(x, o) < i && i < max(x, o) {
                        print!("─");
                    } else if downward_lines[i as usize] {
                        print!("│")
                    } else {
                        print!("·")
                    }
                }
            }
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
        }

        std::fmt::Result::Ok(())
    }
}
