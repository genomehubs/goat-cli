use crate::utils::ranks;
use crate::utils::ranks::Ranks;
use anyhow::Result;
use serde_json::Value;

// perhaps a better way of doing this is defining an enum
// of the possible values e.g.
#[derive(Clone)]
pub enum GoaTValue {
    RawAssemblyLevel(String),
    RawAssemblySpan(u64),
    RawChromosomeNumber(Vec<Option<u64>>),
    RawHaploid(Vec<Option<u64>>),
    RawCValue(f64),
    RawGenomeSize(u64),
    RawBuscoCompleteness(f64),
    // display level 2
    RawMitochondrionAssemblySpan(u64),
    RawMitochondrionGCPercent(f64),
}

// and then a single struct e.g.
#[derive(Clone)]
pub struct RawRecord {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub value: GoaTValue,
}

// then we collect into a Vec of these: e.g.
#[derive(Clone)]
pub struct RawRecords(pub Vec<RawRecord>);

impl RawRecords {
    pub fn new() -> Self {
        RawRecords(Vec::new())
    }

    pub fn get_results(&mut self, v: &Value, ranks_vec: &Vec<String>) -> Result<()> {
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

            // if we wanted to add ranks, we do this here.
            // then add an Option<> to each struct above for rank
            let ranks = ranks::get_ranks(v, index, ranks_vec);

            match map_of_fields_op {
                Some(r) => {
                    for (key, value) in r {
                        match &key[..] {
                            "assembly_level" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: el["source_id"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: GoaTValue::RawAssemblyLevel(
                                                    el["value"].as_str().unwrap().to_string(),
                                                ),
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
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: el["source_id"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: GoaTValue::RawAssemblySpan(
                                                    el["value"].as_u64().unwrap(),
                                                ),
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
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: "".to_string(),
                                                source: el["source"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string(),
                                                value: GoaTValue::RawCValue(
                                                    el["value"].as_f64().unwrap(),
                                                ),
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
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: "".to_string(),
                                                // look at the underlying source here
                                                source: el["source"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string(),
                                                value: GoaTValue::RawGenomeSize(
                                                    el["value"].as_u64().unwrap(),
                                                ),
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
                                            // get chromosome numbers out of array
                                            let chrom_num_vec = el["value"].as_array();
                                            let chrom_num = match chrom_num_vec {
                                                Some(c) => c.iter().map(|x| x.as_u64()).collect(),
                                                None => vec![el["value"].as_u64()],
                                            };
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source_id: "".to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: GoaTValue::RawChromosomeNumber(chrom_num),
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
                                            // get chromosome numbers out of array
                                            let hap_num_vec = el["value"].as_array();
                                            let hap_num = match hap_num_vec {
                                                Some(c) => c.iter().map(|x| x.as_u64()).collect(),
                                                None => vec![el["value"].as_u64()],
                                            };
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                source_id: "".to_string(),
                                                value: GoaTValue::RawHaploid(hap_num),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            // why's there a space here Rich?
                            "busco completeness" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                source_id: "".to_string(),
                                                value: GoaTValue::RawBuscoCompleteness(
                                                    el["value"].as_f64().unwrap(),
                                                ),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            "mitochondrion_assembly_span" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                source_id: "".to_string(),
                                                value: GoaTValue::RawMitochondrionAssemblySpan(
                                                    el["value"].as_u64().unwrap(),
                                                ),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
                            // any other fields?
                            "mitochondrion_gc_percent" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.0.push(RawRecord {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                source_id: "".to_string(),
                                                value: GoaTValue::RawMitochondrionGCPercent(
                                                    el["value"].as_f64().unwrap(),
                                                ),
                                            })
                                        }
                                    }
                                    None => {}
                                }
                            }
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
