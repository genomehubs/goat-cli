use crate::utils::ranks;
use crate::utils::ranks::Ranks;
use anyhow::Result;
use serde_json::Value;

#[derive(Clone)]
pub enum GoaTValueAgg {
    // display level 1
    AssemblyLevel(String),
    AssemblySpan(u64),
    BuscoCompleteness(f64),
    ChromosomeNumber(u64),
    CValue(f64),
    GenomeSize(u64),
    Haploid(u64),
    // display level 2
    MitochondrionAssemblySpan(u64),
    MitochondrionGCPercent(f64),
    PlastidAssemblySpan(u64),
    PlastidGCPercent(f64),
}

#[derive(Clone)]
pub enum MinMax {
    Minmaxf64(Option<f64>),
    Minmaxu64(Option<u64>),
}

#[derive(Clone)]
pub struct Record {
    pub ranks: Ranks,                         // optional but this is okay
    pub taxon_name: String,                   // always present
    pub taxon_id: String,                     // always present
    pub aggregation_source: String,           // always present
    pub min: MinMax,                          // only in tax_tree
    pub max: MinMax,                          // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: GoaTValueAgg,                  // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Clone)]
pub struct Records(pub Vec<Record>);

impl Records {
    // create a new Record instance
    pub fn new() -> Self {
        Records(Vec::new())
    }
    // get all the records associated with a single
    // aggregated taxon (most of GoaT)
    pub fn get_results(&mut self, v: &Value, ranks_vec: &Vec<String>) -> Result<()> {
        // how many results are there?
        let results_len_op = v["results"].as_array();
        // safely get this number out
        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };

        // loop over the indexes of these results
        for index in 0..results_len {
            // can I get the name and taxid here?
            // TODO: make the unwrap safer here.
            let taxon_name = v["results"][index]["result"]["scientific_name"]
                .as_str()
                .unwrap_or("-");
            let taxon_id = v["results"][index]["result"]["taxon_id"]
                .as_str()
                .unwrap_or("-");
            // get the map (k, v pair) of each field
            let map_of_fields_op = v["results"][index]["result"]["fields"].as_object();

            let ranks = ranks::get_ranks(v, index, ranks_vec);

            // match each field and read into Record
            match map_of_fields_op {
                Some(r) => {
                    for (key, value) in r {
                        // match on the key here?
                        match &key[..] {
                            "assembly_level" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::AssemblyLevel(
                                        value["value"].as_str().unwrap().to_string(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "assembly_span" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::AssemblySpan(
                                        value["value"].as_u64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "busco completeness" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxf64(value["min"].as_f64()),
                                    max: MinMax::Minmaxf64(value["max"].as_f64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::BuscoCompleteness(
                                        value["value"].as_f64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "c_value" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxf64(value["min"].as_f64()),
                                    max: MinMax::Minmaxf64(value["max"].as_f64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::CValue(value["value"].as_f64().unwrap()),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "chromosome_number" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::ChromosomeNumber(
                                        value["value"].as_u64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "genome_size" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::GenomeSize(
                                        value["value"].as_u64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "haploid_number" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::Haploid(value["value"].as_u64().unwrap()),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            // display level 2
                            "mitochondrion_assembly_span" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::MitochondrionAssemblySpan(
                                        value["value"].as_u64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "mitochondrion_gc_percent" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxf64(value["min"].as_f64()),
                                    max: MinMax::Minmaxf64(value["max"].as_f64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::MitochondrionGCPercent(
                                        value["value"].as_f64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "plastid_assembly_span" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxu64(value["min"].as_u64()),
                                    max: MinMax::Minmaxu64(value["max"].as_u64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::PlastidAssemblySpan(
                                        value["value"].as_u64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            "plastid_gc_percent" => {
                                let get_values = Record {
                                    ranks: Ranks(ranks.clone()),
                                    taxon_name: taxon_name.to_string(),
                                    taxon_id: taxon_id.to_string(),
                                    aggregation_source: value["aggregation_source"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    min: MinMax::Minmaxf64(value["min"].as_f64()),
                                    max: MinMax::Minmaxf64(value["max"].as_f64()),
                                    count: value["count"].as_u64().unwrap(),
                                    aggregation_taxon_id: Some(
                                        value["aggregation_taxon_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                    value: GoaTValueAgg::PlastidGCPercent(
                                        value["value"].as_f64().unwrap(),
                                    ),
                                    aggregation_method: value["aggregation_method"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(), // always present
                                    aggregation_rank: Some(
                                        value["aggregation_rank"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    ),
                                };
                                self.0.push(get_values);
                            }
                            _ => {
                                // not quite sure what to do with this arm yet.
                            }
                        }
                    }
                }
                None => {
                    // we shouldn't really reach here...
                    // but print to stderr for debugging if we do.
                    eprintln!("There were no fields for {} ({})", taxon_name, taxon_id);
                }
            };
        }
        Ok(())
    }
}
