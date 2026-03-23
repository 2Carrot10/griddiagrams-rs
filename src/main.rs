use std::collections::HashMap;

type GridNotation = Vec<Vec<i32>>;
type GridList = Vec<i32>;

type VertList = Vec<(i32, i32)>;
type HorzList = Vec<(i32, i32)>;

enum DirList {
    VertList(VertList),
    HorzList(HorzList)
}

enum Dir {
    North,
    South,
    East,
    West
}

type Permutation = Vec<i32>;
type WindingMatrix = Vec<Vec<i32>>;

fn main() {
    println!("Hello, world!");
}

fn gridnotation_to_gridlist(gridnotation: GridNotation) -> GridList {
    todo!()
}

fn hlist(gridlist: GridList) -> HorzList {
    todo!()
}

fn vlist(gridlist: GridList) -> VertList {
    todo!()
}

fn v_to_h(vertlist: VertList) -> HorzList {
    todo!()
}

fn h_to_v(horzlist: HorzList) -> VertList {
    todo!()
}

fn can_commute(t1: Vec<(i32, i32)>, t2: Vec<(i32, i32)>) -> bool {
    todo!()
}

fn c_move(input_list: DirList) -> Vec<DirList> {
    todo!()
}

fn knot_commute(vertlist: VertList) -> Vec<VertList> {
    todo!()
}

fn w_matrix(vertlist: VertList) -> WindingMatrix {
    todo!()
}

fn h_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
    todo!()
}

fn v_type_0_permutation(matrix: WindingMatrix) -> Result<Permutation, String> {
    todo!()
}

fn rev(input_list: DirList) -> DirList {
    todo!()
}

fn a_grading(vertlist: VertList, matrix: WindingMatrix, permutation: Permutation) -> i32 {
    todo!()
}

fn vperm_to_hperm(vperm: Permutation) -> Permutation {
    todo!()
}

fn try_permutations(vertlist: VertList) -> Option<(VertList, String, Permutation, WindingMatrix, i32)> {
    todo!()
}

fn vlist_to_XO(vertlist: VertList) -> (Vec<i32>, Vec<i32>) {
    todo!()
}

fn gridstate_finder_commute<K, V>(vertlist: VertList, n: i32) -> Option<HashMap<K, V>> {
    todo!()
}

fn _gridstate_finder_commute_with_visited<K, V>(vertlist: VertList, n: i32) -> Option<HashMap<K, V>> {
    todo!()
}

fn gridstate_finder_stab<K, V>(vertlist: VertList, n: i32) -> Option<HashMap<K, V>> {
    todo!()
}

fn destabilize(vertlist: VertList, loc_index: i32, direction: Dir, tuple_index: i32) -> VertList {
    todo!()
}

fn stabilize(vertlist: VertList, loc: (i32, i32), direction: Dir, tuple_index: i32) -> VertList {
    todo!()
}

// TODO: add printing
