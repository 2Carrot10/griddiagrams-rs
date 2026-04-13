use crate::knot_core::{DirList, h_to_v, v_to_h};
use std::cmp::max;
use std::cmp::min;

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
pub fn adj_elementwise_move_on_predicate(
    input_list: &DirList,
    preciate: impl Fn((i32, i32), (i32, i32)) -> bool,
) -> Vec<DirList> {
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
                result.push(swapped_list.clone());
            }
        }
    }

    // Try wrap-around comparison
    let index = input_list.0.len() - 1;
    if preciate(input_list.0[0], input_list.0[index]) {
        let mut swapped_list = input_list.clone();

        let a = swapped_list.0[index];
        let b = swapped_list.0[0];
        swapped_list.0[0] = a;
        swapped_list.0[index] = b;

        if !seen.contains(&swapped_list.0) {
            seen.push(swapped_list.0.clone());
            result.push(swapped_list.clone());
        }
    }
    result
}

pub fn knot_column_and_row_predicate_move(
    vertlist: &DirList,
    predicate: impl Fn((i32, i32), (i32, i32)) -> bool + 'static,
) -> Vec<DirList> {
    let v_list = adj_elementwise_move_on_predicate(vertlist, &predicate);
    let h_list = adj_elementwise_move_on_predicate(&v_to_h(vertlist), &predicate); // Bad clone
    let mut h_to_v_commutations: Vec<DirList> =
        h_list.into_iter().map(|a| h_to_v(&a)).collect();

    h_to_v_commutations.extend(v_list);

    h_to_v_commutations
}

// All of these can contain duplicates
pub fn knot_switch(vertlist: &DirList) -> Vec<DirList> {
    knot_column_and_row_predicate_move(vertlist, can_switch)
}

pub fn knot_commute(vertlist: &DirList) -> Vec<DirList> {
    knot_column_and_row_predicate_move(vertlist, can_commute)
}


pub fn knot_epsilon(vertlist: &DirList) -> Vec<DirList> {
    vec![vertlist.clone()]
}

pub fn knot_stab(input_list: &DirList) -> Vec<DirList> {
    let mut grid_stab_combos = vec![];
    for segment in input_list.0.clone() {
        for (index, dir) in STAB_COMBINATIONS {
            grid_stab_combos.push((segment, dir, index));
        }
    }
    // TODO: fix suboptimal HashSet -> Vec conversion
    grid_stab_combos
        .into_iter()
        .map(|(segment, dir, index)| stabilize(input_list.clone(), segment, dir, index))
        .collect()
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

// Arguments t1 and t2 must be next to each other
pub fn can_switch(t1: (i32, i32), t2: (i32, i32)) -> bool {
    let (a, b) = t1;
    let (c, d) = t2;
    (a == d) || (c == b)
}

pub fn knot_destab(vertlist: &DirList) -> Vec<DirList> {
    let v_commutations = destab_move(vertlist);
    let h_commutations = destab_move(&v_to_h(vertlist));
    let mut h_to_v_commutations: Vec<DirList> =
        h_commutations.into_iter().map(|a| h_to_v(&a)).collect();

    h_to_v_commutations.extend(v_commutations);

    h_to_v_commutations
}

fn destab_move(vertlist: &DirList) -> Vec<DirList> {
    let mut result = vec![];

    let n = vertlist.0.len();

    for i in 0..n {
        if can_destab(vertlist.0[i]) {
            let mut swapped_list = vertlist.clone();
            let (x, o) = swapped_list.0[i];
            let m = min(x, o);
            swapped_list.0.remove(i);
            for j in 0..(n-1) {
                if swapped_list.0[j].0 > m {
                    swapped_list.0[j].0 -= 1;
                }

                if swapped_list.0[j].1 > m {
                    swapped_list.0[j].1 -= 1;
                }
            }

            if !result.contains(&swapped_list) {
                result.push(swapped_list.clone());
            }
        }
    }

    result
}

fn can_destab((x, o): (i32, i32)) -> bool {
    (x - o).abs() == 1
}

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
