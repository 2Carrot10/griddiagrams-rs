use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::search::SearchRecord;

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
#[derive(Clone, Eq, Hash, Serialize, Deserialize)]
pub struct DirList(pub Vec<(i32, i32)>);

impl PartialEq for DirList {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
            || self.0 == v_to_h(other).0
            || self.0.clone().into_iter().rev().collect::<Vec<_>>() == other.0
            || self.0.clone().into_iter().rev().collect::<Vec<_>>() == v_to_h(other).0
    }
}

pub enum Dir {
    Horz,
    Vert,
}

pub type Permutation = Vec<usize>;
pub type WindingMatrix = Vec<Vec<i32>>;

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
                // Python-like negative indexing
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
#[allow(dead_code)]
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
        if x < hsegments.len() {
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

/// Convert vertical segment list to horizontal segment list or converts horizontal segment list to vertical segment list
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
pub fn v_to_h(vertlist: &DirList) -> DirList {
    let n = vertlist.0.len();
    let mut horzlist = vec![];
    for i in 0..n as i32 {
        let mut segment_indicies: (i32, i32) = (-1, -1);
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

pub fn tagged_v_to_h(vertlist: &(DirList, String)) -> (DirList, String) {
    let n = vertlist.0.0.len();
    let mut horzlist = vec![];
    for i in 0..n as i32 {
        let mut segment_indicies: (i32, i32) = (-1, -1);
        for j in 0..n {
            if vertlist.0.0[j as usize].0 == i {
                segment_indicies.0 = j as i32;
            } else if vertlist.0.0[j as usize].1 == i {
                segment_indicies.1 = j as i32;
            }
        }

        horzlist.push(segment_indicies);
    }
    (DirList(horzlist), format!("{}(V-H)", vertlist.1.clone()))
}

pub fn is_valid(dirlist: &DirList) -> bool {
    let len = dirlist.0.len();
    let mut x_binsum = 0;
    let mut y_binsum = 0;
    for (x, o) in &dirlist.0 {
        if (*x as usize) >= len || (*o as usize) >= len {
            return false;
        }

        x_binsum |= 1 << x;
        y_binsum |= 1 << x;
    }

    ((x_binsum + 1) == (1 << len)) && ((y_binsum + 1) == (1 << len))
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
        let mut row = vec![0];
        for j in 0..(size - 1) {
            let (tail, head) = vertlist.0[j];

            let prev = row[row.len() - 1];
            if tail <= (i as i32) && (i as i32) < head {
                row.push(prev + 1);
            } else if head <= (i as i32) && (i as i32) < tail {
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
pub fn type_0_permutation(matrix: WindingMatrix, direction: Dir) -> Option<Permutation> {
    let matrix = if let Dir::Vert = direction {
        transpose(matrix)
    } else {
        matrix
    };

    let n = matrix.len();

    let mut min_indices: Vec<Option<HashSet<usize>>> = matrix
        .into_iter()
        .map(|row| {
            let min = row.iter().min().unwrap();
            row.iter()
                .enumerate()
                .filter(|(_, val)| val == &min)
                .map(|(index, _)| Some(index))
                .collect()
        })
        .collect();
    let mut result = vec![None; n];

    while min_indices.iter().any(|s| s.is_some()) {
        let singleton_index = min_indices
            .iter()
            .enumerate()
            .filter(|(_, val)| val.as_ref().map_or(false, |a| a.len() == 1)) // Find unique entry
            .map(|(index, _)| index)
            .next();

        match singleton_index {
            None => {
                return None;
            }
            Some(singleton_index) => {
                // X is the first element in the singleton set
                let x = min_indices[singleton_index]
                    .as_ref()
                    .unwrap()
                    .iter()
                    .next()
                    .unwrap()
                    .clone();

                result[singleton_index] = Some(x.clone());

                min_indices.iter_mut().for_each(|a| {
                    if let Some(s) = a {
                        s.remove(&x);
                    }
                });

                min_indices[singleton_index] = None;

                if min_indices
                    .iter()
                    .any(|s| s.as_ref().map_or(false, |a| a.is_empty()))
                {
                    return None;
                }
            }
        }
    }

    let result: Vec<usize> = result
        .into_iter()
        .map(|a| a.expect("This should never happen, check code if it does."))
        .collect();

    return Some(result);
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
    fn try_instance_permutations(vertlist: DirList, is_reversed: bool) -> Option<SearchRecord> {
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
            match type_0_permutation(matrix.clone(), Dir::Horz) {
                Some(h_perm) => {
                    return Some(SearchRecord {
                        stabilizations: 0,
                        vlist: vertlist.clone(),
                        alexander_grading: a_grading(&vertlist, &matrix, &h_perm),
                        matrix: matrix.clone(),
                        gridstate: h_perm.clone(),
                        perm_type: if is_reversed {
                            String::from("h_type_0")
                        } else {
                            String::from("h_type_0_rev")
                        },
                        path: None
                    });
                }
                None => ()
            }
        }

        if colsum >= rowsum {
            match type_0_permutation(matrix.clone(), Dir::Vert) {
                Some(h_perm) => {
                    return Some(SearchRecord {
                        stabilizations: 0,
                        vlist: vertlist.clone(),
                        alexander_grading: a_grading(&vertlist, &matrix, &h_perm),
                        matrix: matrix.clone(),
                        gridstate: h_perm.clone(),
                        perm_type: if is_reversed {
                            String::from("v_type_0")
                        } else {
                            String::from("v_type_0_rev")
                        },
                        path: None
                    });
                }
                None => ()
            }
        }
        None
    }

    if let Some(solution) = try_instance_permutations(vertlist.clone(), false) {
        Some(solution)
    } else if let Some(solution) = try_instance_permutations(rev(vertlist.clone()), true) {
        Some(solution)
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
#[allow(dead_code)]
pub fn vlist_to_xo(vertlist: DirList) -> (Vec<i32>, Vec<i32>) {
    let n = (vertlist.0.len() as i32) - 1;
    let tempx: Vec<i32> = vertlist.0.iter().map(|a| a.0).collect();
    let tempo: Vec<i32> = vertlist.0.iter().map(|a| a.1).collect();

    let x = tempx.iter().map(|x| n - x).collect();
    let o = tempo.iter().map(|x| n - x).collect();

    return (x, o);
}
