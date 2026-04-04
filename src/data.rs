use csv;
use serde;
use crate::knot_core::{gridnotation_to_gridlist, vlist, DirList, GridNotation, GridNotationContainer};
type Csv = Vec<Record>;

pub const RAW_CSV: &str = include_str!("../data/knotinfo.csv");


#[derive(Debug, serde::Deserialize)]
pub struct Record {
    #[serde(alias = "Name")]
    pub name: String,

    #[serde(alias = "Grid Notation")]
    pub gridnotation: GridNotationContainer,
}

pub fn load_knot_data() -> Vec<Record> {
    let mut rdr = csv::Reader::from_reader(RAW_CSV.as_bytes());
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
    .gridnotation.0.clone()
}

pub fn get_vlist_by_name(name: String, csv: &Csv) -> DirList {
    vlist(gridnotation_to_gridlist(get_grid_notation(name, csv)))
}
