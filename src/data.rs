use std::fs::read_to_string;
use csv;
use serde;
use crate::search_core::{gridnotation_to_gridlist, vlist, DirList, GridNotation, GridNotationContainer};
type Csv = Vec<Record>;

#[derive(Debug, serde::Deserialize)]
struct Record {
    city: String,
    region: String,
    name: String,
    fibered: String, // Should be Y or N (all of this dataset are N)
    crossing_number: i32,
    genus_3d: i32,
    arc_index: i32,
    gridnotation: GridNotation,
    country: Option<u64>,
}

pub fn load_knot_data(path: String) -> Vec<Record> {
    let mut rdr = csv::Reader::from_path(path).expect("Could not read csv");
    let mut csv = vec![];

    for result in rdr.deserialize() {
        let record: Record = result.expect("Csv element not valid");
        csv.push(record);
    }

    csv
}

pub fn get_all_knot_names(records: &Csv) -> Vec<String> {
    records.iter().map(|a| a.name.clone()).collect()
}

pub fn get_grid_notation(name: String, csv: &Csv) -> GridNotation {
    csv.iter().find(|elem| elem.name == name).expect("Could not find name.")
    .gridnotation.clone()
}

pub fn get_vlist_by_name(name: String, csv: &Csv) -> DirList {
    vlist(gridnotation_to_gridlist(get_grid_notation(name, csv)))
}
