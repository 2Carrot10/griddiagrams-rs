use crate::knot_core::tagged_v_to_h;
use crate::knot_core::{DirList, v_to_h};
use std::cmp::max;
use std::cmp::min;

/// The direction of stabilization.
/// For instance, this is NW stab on a ✗ as the north west cell is empty following the stabilization:
/// ✗   ->  ·✗
///         ✗○
pub enum StabDir {
    NorthWest,
    SouthWest,
    NorthEast,
    SouthEast,
}

/// All possible combinations of stabilization that can be completed on a given column.
/// A stabilization can be completed on a x or o square, in any of the four stabilization
/// directions.
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
    pub fn is_north(&self) -> bool {
        match self {
            StabDir::NorthWest => true,
            StabDir::NorthEast => true,
            StabDir::SouthWest => false,
            StabDir::SouthEast => false,
        }
    }

    pub fn is_west(&self) -> bool {
        match self {
            StabDir::NorthWest => true,
            StabDir::NorthEast => false,
            StabDir::SouthWest => true,
            StabDir::SouthEast => false,
        }
    }
}

/// Generalizes all movements between adjacent rows/columns (the only two moves being switch and
/// commute). This function only operates on the provided segments; it does not rotate the
/// grid diagram.
pub fn adj_elementwise_move_on_predicate(
    input_list: &DirList,
    preciate: impl Fn((i32, i32), (i32, i32)) -> bool,
    name: String,
) -> Vec<(DirList, String)> {
    let mut result = vec![];
    let mut seen = vec![];

    let n = input_list.0.len();

    for i in 0..(n - 1) {
        if preciate(input_list.0[i], input_list.0[i + 1]) {
            let mut swapped_list = input_list.clone();
            let a = swapped_list.0[i + 1];
            let b = swapped_list.0[i];
            swapped_list.0[i] = a;
            swapped_list.0[i + 1] = b;

            if !seen.contains(&swapped_list.0) {
                seen.push(swapped_list.0.clone());
                result.push((swapped_list.clone(), format!("{}{}:{}", name, i, i + 1)));
            }
        }
    }

    // Try wrap-around comparison. (the grid diagram exists on a torus, not a plane)
    let index = input_list.0.len() - 1;
    if preciate(input_list.0[0], input_list.0[index]) {
        let mut swapped_list = input_list.clone();

        let a = swapped_list.0[index];
        let b = swapped_list.0[0];
        swapped_list.0[0] = a;
        swapped_list.0[index] = b;

        if !seen.contains(&swapped_list.0) {
            seen.push(swapped_list.0.clone());
            result.push((swapped_list.clone(), format!("{}{}:{}", name, 0, index)));
        }
    }
    result
}

/// Generalizes all movements between adjacent rows/columns (the only two moves being switch and
/// commute). This function rotates the `vertlist` in order to try the predicate on all rows and
/// columns.
pub fn knot_column_and_row_predicate_move(
    vertlist: &DirList,
    predicate: impl Fn((i32, i32), (i32, i32)) -> bool + 'static,
    name: String,
) -> Vec<(DirList, String)> {
    let v_list = adj_elementwise_move_on_predicate(vertlist, &predicate, name.clone());
    let h_list = adj_elementwise_move_on_predicate(&v_to_h(vertlist), &predicate, name); // Bad clone
    let mut h_to_v_commutations: Vec<(DirList, String)> =
        h_list.into_iter().map(|a| tagged_v_to_h(&a)).collect();

    h_to_v_commutations.extend(v_list);

    h_to_v_commutations
}

/// Computes all possible switches for all orientations of the dirlist (both vertical
/// segments and horizontal segments)
/// `dirlist` - the grid diagram upon which the move will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move.
pub fn knot_switch(vertlist: &DirList) -> Vec<(DirList, String)> {
    knot_column_and_row_predicate_move(vertlist, can_switch, String::from("switch"))
}

/// Computes all possible commutations for all orientations of the dirlist (both vertical
/// segments and horizontal segments)
/// `dirlist` - the grid diagram upon which the move will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move.
pub fn knot_commute(vertlist: &DirList) -> Vec<(DirList, String)> {
    knot_column_and_row_predicate_move(vertlist, can_commute, String::from("commute"))
}

/// Epsilon represents the 'do nothing' move.
/// `dirlist` - the grid diagram upon which the move (no effect) will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move. The list is
/// guaranteed to have a length of 1, containing only the input `dirlist`.
pub fn knot_epsilon(vertlist: &DirList) -> Vec<(DirList, String)> {
    vec![(vertlist.clone(), String::from("epsilon"))]
}

/// Computes all possible destabilizations for all orientations of the dirlist (both vertical
/// segments and horizontal segments)
/// `dirlist` - the grid diagram upon which the move will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move.
pub fn knot_stab(input_list: &DirList) -> Vec<(DirList, String)> {
    let mut grid_stab_combos = vec![];
    for segment in input_list.0.clone() {
        for (index, dir) in STAB_COMBINATIONS {
            grid_stab_combos.push((segment, dir, index));
        }
    }
    grid_stab_combos
        .into_iter()
        .map(|(segment, dir, index)| stabilize(input_list.clone(), segment, dir, index))
        .collect()
}

/// Checks if two adjacent columns or rows can undergo the reidemeister move called 'commutation'
/// `t1` - the indicies of the x and o for the given slice of the dirlist.
/// `t2` - the indicies of the x and o for the given slice of the dirlist.
/// Note that `t1` and `t2` must be adjacent columns.
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

/// Checks if two adjacent columns or rows can undergo the reidemeister move called 'switch'
/// `t1` - the indicies of the x and o for the given slice of the dirlist.
/// `t2` - the indicies of the x and o for the given slice of the dirlist.
/// Note that `t1` and `t2` must be adjacent columns.
pub fn can_switch(t1: (i32, i32), t2: (i32, i32)) -> bool {
    let (a, b) = t1;
    let (c, d) = t2;
    (a == d) || (c == b)
}

/// Computes all possible destabilizations for all orientations of the dirlist (both vertical
/// segments and horizontal segments)
/// `dirlist` - the grid diagram upon which the move will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move.
pub fn knot_destab(dirlist: &DirList) -> Vec<(DirList, String)> {
    let v_commutations = destab_move(dirlist);
    let h_commutations = destab_move(&v_to_h(dirlist));
    let mut h_to_v_commutations: Vec<(DirList, String)> = h_commutations
        .into_iter()
        .map(|a| tagged_v_to_h(&a))
        .collect();

    h_to_v_commutations.extend(v_commutations);

    h_to_v_commutations
}

/// Computes all possible destabilizations for only vertical segments.
/// `vertlist` - the vertlist upon which the move will be accomplished.
/// return: `Vec<(DirList, String)>` - a list of the results of the move, each element in the list
/// containing the resulting dirlist in addition to a string representation of the move.
fn destab_move(vertlist: &DirList) -> Vec<(DirList, String)> {
    let mut result = vec![];

    let n = vertlist.0.len();

    for i in 0..n {
        if can_destab(vertlist.0[i]) {
            let mut swapped_list = vertlist.clone();
            let (x, o) = swapped_list.0[i];
            let m = min(x, o);
            swapped_list.0.remove(i);
            for j in 0..(n - 1) {
                if swapped_list.0[j].0 > m {
                    swapped_list.0[j].0 -= 1;
                }

                if swapped_list.0[j].1 > m {
                    swapped_list.0[j].1 -= 1;
                }
            }

            let elem = (swapped_list.clone(), format!("Destab{}", i));
            if !result.contains(&elem) {
                result.push(elem);
            }
        }
    }

    result
}

/// Checks if a column or row can be destabilized.
fn can_destab((x, o): (i32, i32)) -> bool {
    // Interestingly, adjacency of the x and o values is the only check necessary for this
    // otherwise complicated operation. This is because it would be possible to repeatedly
    // commute the row/column upon which the pair are placed until they reach a third element,
    // at which point a triangle shape will be created and a destabilization can be accomplished
    //
    //   adjacent x and o identified
    //   |          commutations can always be performed
    //   |          |          destabilization can now be performed
    //   |          |          |         the result
    //   v          v          v         |
    //   ○··✗      ·○·✗      ··○✗        v
    //   ✗···  ->  ·✗··  ->  ··✗·  ->  ··○

    (x - o).abs() == 1
}

/// Takes in a vertlist and stabilization parameters, returning the resulting grid diagram after stabilization
/// `vertlist` - the grid diagram to be operated upon
/// `loc` - the indices of the x and o of the vertical segment to be operated upon. Note, this
/// element will be unique due to the definition for a grid diagram
/// `direction` - the direction in which the stabilization will occur
/// `tuple_index` - either `0` or `1`, representing having either x or o as the center of the
/// stabilization. Note, the vertical segment has already been selected by `loc`, leaving two
/// options for the center of stabilization.
pub fn stabilize(
    vertlist: DirList,
    loc: (i32, i32),
    direction: StabDir,
    tuple_index: usize,
) -> (DirList, String) {
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

    (
        DirList(temp.iter().map(|[a, b]| (*a, *b)).collect()),
        format!("stabilize{},{}{}{}", k, loc[tuple_index], if is_north { "n" } else { "s" }, if is_west {"w"} else {"e"}),
    )
}
