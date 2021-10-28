// change all of the .expect()'s
// also potentially change the API here to
// be more in line with agg_values

use crate::utils::ranks;
use crate::utils::ranks::Ranks;
use anyhow::Result;
use serde_json::Value;

// can these be dynamically generated?

#[derive(Clone)]
pub struct RawAssemblyLevel {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub value: String,
}
#[derive(Clone)]
pub struct RawAssemblySpan {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawChromosomeNumber {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: Vec<Option<u64>>,
}

#[derive(Clone)]
pub struct RawHaploid {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: Vec<Option<u64>>,
}

#[derive(Clone)]
pub struct RawCValue {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: f64,
}

#[derive(Clone)]
pub struct RawGenomeSize {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawBuscoCompleteness {
    pub ranks: Ranks,
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: f64,
}

// this is the struct to gather the concurrent results for the raw values.
#[derive(Clone)]
pub struct RawRecords {
    pub level: Vec<RawAssemblyLevel>,
    pub span: Vec<RawAssemblySpan>,
    pub busco_completeness: Vec<RawBuscoCompleteness>,
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
            busco_completeness: Vec::new(),
            c_value: Vec::new(),
            chromosome_number: Vec::new(),
            genome_size: Vec::new(),
            haploid: Vec::new(),
        }
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
                                            self.level.push(RawAssemblyLevel {
                                                ranks: Ranks(ranks.clone()),
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
                                                ranks: Ranks(ranks.clone()),
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
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                // look at the underlying source here
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
                            "genome_size" => {
                                let raw_values = value["rawValues"].as_array();
                                match raw_values {
                                    Some(rv) => {
                                        for el in rv {
                                            self.genome_size.push(RawGenomeSize {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                // look at the underlying source here
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
                                            self.chromosome_number.push(RawChromosomeNumber {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                // bug was here because chromosome numbers can be in
                                                // an array.
                                                value: chrom_num,
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
                                            self.haploid.push(RawHaploid {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: hap_num,
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
                                            self.busco_completeness.push(RawBuscoCompleteness {
                                                ranks: Ranks(ranks.clone()),
                                                taxon_name: taxon_name.to_string(),
                                                taxon_ncbi: taxon_id.to_string(),
                                                source: el["source"].as_str().unwrap().to_string(),
                                                value: el["value"].as_f64().unwrap(),
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
