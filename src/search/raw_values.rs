use crate::error::error;
use anyhow::{bail, Result};
use serde_json::Value;
// These structs are all very copy-heavy.
// I think this is fine for the small (ish) data
// coming from GoaT.
pub struct RawAssemblyLevel {
    pub source_id: String,
    pub source: String,
    pub value: String,
}
pub struct RawAssemblySpan {
    pub source_id: String,
    pub source: String,
    pub value: u64,
}
pub struct RawAssembly {
    pub level: Vec<RawAssemblyLevel>,
    pub span: Vec<RawAssemblySpan>,
}

#[derive(Clone)]
pub struct CombinedRawAssembly {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source_id: String,
    pub source: String,
    pub assembly_type: String,
    pub span: u64,
}

impl RawAssembly {
    pub fn new() -> Self {
        RawAssembly {
            level: Vec::new(),
            span: Vec::new(),
        }
    }
    // I think sorting should be quicker.
    pub fn merge(&mut self, taxon_name: &str, taxon_ncbi: &str) -> Vec<CombinedRawAssembly> {
        // sort the vecs
        &self.level.sort_by(|a, b| a.source_id.cmp(&b.source_id));
        &self.span.sort_by(|a, b| a.source_id.cmp(&b.source_id));

        let mut res = Vec::new();

        for (el1, el2) in self.level.iter().zip(&self.span) {
            res.push(CombinedRawAssembly {
                taxon_name: taxon_name.to_string(),
                taxon_ncbi: taxon_ncbi.to_string(),
                source_id: el1.source_id.clone(),
                source: el1.source.clone(),
                assembly_type: el1.value.clone(),
                span: el2.value,
            });
        }
        res
    }
}

pub fn get_raw_assembly(v: &Value, assembly: &mut RawAssembly, variable: &str) -> Result<()> {
    // there might be more than one hit in 'results'.
    // this panics on the unwrap - fix this.
    let results_len_op = v["results"].as_array();

    let results_len = match results_len_op {
        Some(r) => r.len(),
        None => 0,
    };

    // iterate over the raw values and add to `assembly`
    match variable {
        "assembly_level" => {
            for index in 0..results_len {
                for i in v["results"][index]["result"]["fields"][variable]["rawValues"].as_array() {
                    for j in i {
                        assembly.level.push(RawAssemblyLevel {
                            source_id: j["source_id"]
                                .as_str()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToStr))
                                .to_string(),
                            source: j["source"]
                                .as_str()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToStr))
                                .to_string(),
                            value: j["value"]
                                .as_str()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToStr))
                                .to_string(),
                        });
                    }
                }
            }
        }
        "assembly_span" => {
            for index in 0..results_len {
                for i in v["results"][index]["result"]["fields"][variable]["rawValues"].as_array() {
                    for j in i {
                        assembly.span.push(RawAssemblySpan {
                            source_id: j["source_id"]
                                .as_str()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToStr))
                                .to_string(),
                            source: j["source"]
                                .as_str()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToStr))
                                .to_string(),
                            value: j["value"]
                                .as_u64()
                                .expect(&format!("{}", error::RetrieveRecordError::CastToU64)),
                        });
                    }
                }
            }
        }
        other => bail!(format!(
            "{}: found {}",
            error::RetrieveRecordError::MatchingRawValueError,
            other
        )),
    }
    Ok(())
}

// It's kind of lazy, but I am going to keep these as all separate structs
// (as they all have the same structure)
// in case the GoaT API changes dramatically.

#[derive(Clone)]
pub struct RawChromosomeNumberRecord {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}
#[derive(Clone)]
pub struct RawChromosomeNumbers(pub Vec<RawChromosomeNumberRecord>);

impl RawChromosomeNumbers {
    pub fn new() -> Self {
        RawChromosomeNumbers(Vec::new())
    }

    pub fn populate(&mut self, v: &Value, taxon_name: &str, taxon_ncbi: &str) {
        let results_len_op = v["results"].as_array();

        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };
        for index in 0..results_len {
            for i in
                v["results"][index]["result"]["fields"]["chromosome_number"]["rawValues"].as_array()
            {
                for j in i {
                    self.0.push(RawChromosomeNumberRecord {
                        taxon_name: taxon_name.to_string(),
                        taxon_ncbi: taxon_ncbi.to_string(),
                        source: j["source"]
                            .as_str()
                            .unwrap_or("Unknown source.")
                            .to_string(),
                        // TODO: better error handling here.
                        value: j["value"].as_u64().unwrap_or(0),
                    });
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RawHaploidNumberRecord {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}
#[derive(Clone)]
pub struct RawHaploidNumbers(pub Vec<RawHaploidNumberRecord>);

impl RawHaploidNumbers {
    pub fn new() -> Self {
        RawHaploidNumbers(Vec::new())
    }

    pub fn populate(&mut self, v: &Value, taxon_name: &str, taxon_ncbi: &str) {
        let results_len_op = v["results"].as_array();

        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };
        for index in 0..results_len {
            for i in
                v["results"][index]["result"]["fields"]["haploid_number"]["rawValues"].as_array()
            {
                for j in i {
                    self.0.push(RawHaploidNumberRecord {
                        taxon_name: taxon_name.to_string(),
                        taxon_ncbi: taxon_ncbi.to_string(),
                        source: j["source"]
                            .as_str()
                            .unwrap_or("Unknown source.")
                            .to_string(),
                        // TODO: better error handling here.
                        value: j["value"].as_u64().unwrap_or(0),
                    });
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RawCValueRecord {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: f64,
}

#[derive(Clone)]
pub struct RawCValues(pub Vec<RawCValueRecord>);

impl RawCValues {
    pub fn new() -> Self {
        RawCValues(Vec::new())
    }

    pub fn populate(&mut self, v: &Value, taxon_name: &str, taxon_ncbi: &str) {
        let results_len_op = v["results"].as_array();

        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };
        for index in 0..results_len {
            for i in v["results"][index]["result"]["fields"]["c_value"]["rawValues"].as_array() {
                for j in i {
                    self.0.push(RawCValueRecord {
                        taxon_name: taxon_name.to_string(),
                        taxon_ncbi: taxon_ncbi.to_string(),
                        source: j["source"]
                            .as_str()
                            .unwrap_or("Unknown source.")
                            .to_string(),
                        value: j["value"]
                            .as_f64()
                            .expect(&format!("{}", error::RetrieveRecordError::CastToF64)),
                    });
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RawGSRecord {
    pub taxon_name: String,
    pub taxon_ncbi: String,
    pub source: String,
    pub value: u64,
}

#[derive(Clone)]
pub struct RawGSs(pub Vec<RawGSRecord>);

impl RawGSs {
    pub fn new() -> Self {
        RawGSs(Vec::new())
    }

    pub fn populate(&mut self, v: &Value, taxon_name: &str, taxon_ncbi: &str) {
        let results_len_op = v["results"].as_array();

        let results_len = match results_len_op {
            Some(r) => r.len(),
            None => 0,
        };
        for index in 0..results_len {
            for i in v["results"][index]["result"]["fields"]["genome_size"]["rawValues"].as_array()
            {
                for j in i {
                    self.0.push(RawGSRecord {
                        taxon_name: taxon_name.to_string(),
                        taxon_ncbi: taxon_ncbi.to_string(),
                        source: j["source"]
                            .as_str()
                            .unwrap_or("Unknown source.")
                            .to_string(),
                        value: j["value"]
                            .as_u64()
                            .expect(&format!("{}", error::RetrieveRecordError::CastToU64)),
                    });
                }
            }
        }
    }
}

// TODO BUSCO completeness?

// this is the struct to gather the concurrent results for the raw values.
#[derive(Clone)]
pub struct AggRawFetches {
    pub combined_raw: Vec<CombinedRawAssembly>, // make better struct
    pub c_values: RawCValues,
    pub chrom_nums: RawChromosomeNumbers,
    pub genome_sizes: RawGSs,
    pub haploid: RawHaploidNumbers,
}
