// change all of the .expect()'s
// also potentially change the API here to
// be more in line with agg_values

use anyhow::Result;
use serde_json::Value;
// These structs are all very copy-heavy.
// I think this is fine for the small (ish) data
// coming from GoaT.
#[derive(Clone)]
pub struct RawAssemblyLevel {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub value: String,
}
#[derive(Clone)]
pub struct RawAssemblySpan {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawChromosomeNumber {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawHaploid {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawCValue {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: f64,
}

#[derive(Clone)]
pub struct RawGenomeSize {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}

// this is the struct to gather the concurrent results for the raw values.
#[derive(Clone)]
pub struct RawRecords {
    pub level: Vec<RawAssemblyLevel>, // make better struct
    pub span: Vec<RawAssemblySpan>,
    // pub busco_completeness: Vec<_>
    pub c_value: Vec<RawCValue>,
    pub chromosome_number: Vec<RawChromosomeNumber>,
    pub genome_size: Vec<RawGenomeSize>,
    pub haploid: Vec<RawHaploid>,
}

impl RawRecords {
    pub fn new() -> Self {
        RawRecords {
            level: Vec::new(),
            span: Vec::new(),
            c_value: Vec::new(),
            chromosome_number: Vec::new(),
            genome_size: Vec::new(),
            haploid: Vec::new(),
        }
    }

    pub fn get_results(&mut self, v: &Value) -> Result<()> {
        // how many results are there?
        let results_len_op = v["results"].as_array();
        // safely get the number of results.
        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };

        // loop over the indexes of these results
        for index in 0..results_len {
            // TODO: make the unwrap safer here.
            let taxon_name = v["results"][index]["result"]["scientific_name"]
                .as_str()
                .unwrap();
            let taxon_id = v["results"][index]["result"]["taxon_id"].as_str().unwrap();

            // get a map of each field
            let map_of_fields_op = v["results"][index]["result"]["fields"].as_object();

            match map_of_fields_op {
                Some(r) => {
                    for (key, value) in r {
                        match &key[..] {
                            "assembly_level" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.level.push(RawAssemblyLevel {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: el["source_id"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: el["value"].as_str().unwrap().to_string(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "assembly_span" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.span.push(RawAssemblySpan {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: el["source_id"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: el["value"].as_u64().unwrap(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "c_value" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.c_value.push(RawCValue {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string(),
                                                value: el["value"].as_f64().unwrap(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "chromosome_number" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.chromosome_number.push(RawChromosomeNumber {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: el["value"].as_u64().unwrap(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "genome_size" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.genome_size.push(RawGenomeSize {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string(),
                                                value: el["value"].as_u64().unwrap(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "haploid_number" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.haploid.push(RawHaploid {
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: el["value"].as_u64().unwrap(),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            // add busco here.
                            _ => {}
                        }
                    }
                }
                None => {
                    // we shouldn't really reach here...
                    // but print to stderr for debugging if we do.
                    eprintln!("There were no fields for {} ({})", taxon_name, taxon_id);
                }
            }
        }

        Ok(())
    }
}

// might be nice to resurrect this later.

// pub struct RawAssembly {
//     pub level: Vec<RawAssemblyLevel>,
//     pub span: Vec<RawAssemblySpan>,
// }

// #[derive(Clone)]
// pub struct CombinedRawAssembly {
//     pub taxon_name: String,
//     pub taxon_ncbi: String,
//     pub source_id: String,
//     pub source: String,
//     pub assembly_type: String,
//     pub span: u64,
// }

// impl RawAssembly {
//     pub fn new() -> Self {
//         RawAssembly {
//             level: Vec::new(),
//             span: Vec::new(),
//         }
//     }
//     // I think sorting should be quicker.
//     pub fn merge(&mut self, taxon_name: &str, taxon_ncbi: &str) -> Vec<CombinedRawAssembly> {
//         // sort the vecs
//         &self.level.sort_by(|a, b| a.source_id.cmp(&b.source_id));
//         &self.span.sort_by(|a, b| a.source_id.cmp(&b.source_id));

//         let mut res = Vec::new();

//         for (el1, el2) in self.level.iter().zip(&self.span) {
//             res.push(CombinedRawAssembly {
//                 taxon_name: taxon_name.to_string(),
//                 taxon_ncbi: taxon_ncbi.to_string(),
//                 source_id: el1.source_id.clone(),
//                 source: el1.source.clone(),
//                 assembly_type: el1.value.clone(),
//                 span: el2.value,
//             });
//         }
//         res
//     }
// }
