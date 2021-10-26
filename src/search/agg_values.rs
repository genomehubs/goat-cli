// ignore assembly level
// aggregation method does not seem to be consistent?
// so record this.

// the request URL for this is:

//https://goat.genomehubs.org/api/v0.0.1/
// search?
// query=tax_name%28Arabidopsis%20thaliana%29 // can also be query=tax_tree%28Arabidopsis%29 (which I believe gives no raw values back.)
// &includeEstimates=false
// &includeRawValues=false
// &searchRawValues=false
// &summaryValues=median
// &result=taxon
// &taxonomy=ncbi
// &tidyData=false

// build a bunch of structs to hold this information.

use anyhow::Result;
use serde_json::Value;
// Again, these structs are all very copy-heavy.
// I think this is fine for the small (ish) data
// coming from GoaT.
#[derive(Debug, Clone)]
pub struct AssemblyLevel {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<u64>,                     // only in tax_tree
    pub max: Option<u64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: String,                        // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Debug, Clone)]
pub struct AssemblySpan {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<u64>,                     // only in tax_tree
    pub max: Option<u64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: u64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

// no example yet for tax_tree busco completeness
// assume they are the same as the assembly ones.
#[derive(Debug, Clone)]
pub struct BuscoCompleteness {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<f64>,                     // only in tax_tree
    pub max: Option<f64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: f64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Debug, Clone)]
pub struct CValue {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<f64>,                     // only in tax_tree
    pub max: Option<f64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: f64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Debug, Clone)]
pub struct ChromosomeNumber {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<u64>,                     // only in tax_tree
    pub max: Option<u64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: u64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Debug, Clone)]
pub struct GenomeSize {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<u64>,                     // only in tax_tree
    pub max: Option<u64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: u64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

#[derive(Debug, Clone)]
pub struct Haploid {
    pub taxon_name: String,
    pub taxon_id: String,
    pub aggregation_source: String,           // always present
    pub min: Option<u64>,                     // only in tax_tree
    pub max: Option<u64>,                     // only in tax_tree
    pub count: u64,                           // always present
    pub aggregation_taxon_id: Option<String>, // only in tax_tree
    pub value: u64,                           // always present
    pub aggregation_method: String,           // always present
    pub aggregation_rank: Option<String>,     // only in tax_tree
}

// essentially a vector of records
#[derive(Clone)]
pub struct Records {
    pub span: Vec<AssemblySpan>,
    pub level: Vec<AssemblyLevel>,
    pub busco_completeness: Vec<BuscoCompleteness>,
    pub c_value: Vec<CValue>,
    pub chromosome_number: Vec<ChromosomeNumber>,
    pub genome_size: Vec<GenomeSize>,
    pub haploid: Vec<Haploid>,
}

impl Records {
    pub fn new() -> Self {
        Records {
            level: Vec::new(),
            span: Vec::new(),
            busco_completeness: Vec::new(),
            c_value: Vec::new(),
            chromosome_number: Vec::new(),
            genome_size: Vec::new(),
            haploid: Vec::new(),
        }
    }
    // don't think I need to sort... yet
}

// mutate the Records struct
pub fn get_results(v: &Value, records: &mut Records) -> Result<()> {
    // there might be more than one hit in 'results'.
    // this panics on the unwrap - fix this.
    let results_len_op = v["results"].as_array();

    let results_len = match results_len_op {
        Some(r) => r.len(),
        None => 0,
    };

    for index in 0..results_len {
        // can I get the name and taxid here?
        // TODO: make the unwrap safer here.
        let taxon_name = v["results"][index]["result"]["scientific_name"]
            .as_str()
            .unwrap();
        let taxon_id = v["results"][index]["result"]["taxon_id"].as_str().unwrap();
        // get the length of the number of fields per record.
        let map_of_fields_op = v["results"][index]["result"]["fields"].as_object();

        match map_of_fields_op {
            Some(r) => {
                for (key, value) in r {
                    // match on the key here?
                    match &key[..] {
                        "assembly_level" => {
                            let get_values = AssemblyLevel {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_u64(),
                                max: value["max"].as_u64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_str().unwrap().to_string(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.level.push(get_values);
                        }
                        "assembly_span" => {
                            let get_values = AssemblySpan {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_u64(),
                                max: value["max"].as_u64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_u64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.span.push(get_values);
                        }
                        "busco completeness" => {
                            let get_values = BuscoCompleteness {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_f64(),
                                max: value["max"].as_f64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_f64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.busco_completeness.push(get_values);
                        }
                        "c_value" => {
                            let get_values = CValue {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_f64(),
                                max: value["max"].as_f64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_f64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.c_value.push(get_values);
                        }
                        "chromosome_number" => {
                            let get_values = ChromosomeNumber {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_u64(),
                                max: value["max"].as_u64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_u64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.chromosome_number.push(get_values);
                        }
                        "genome_size" => {
                            let get_values = GenomeSize {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_u64(),
                                max: value["max"].as_u64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_u64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.genome_size.push(get_values);
                        }
                        "haploid_number" => {
                            let get_values = Haploid {
                                taxon_name: taxon_name.to_string(),
                                taxon_id: taxon_id.to_string(),
                                aggregation_source: value["aggregation_source"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                min: value["min"].as_u64(),
                                max: value["max"].as_u64(),
                                count: value["count"].as_u64().unwrap(),
                                aggregation_taxon_id: Some(
                                    value["aggregation_taxon_id"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                ),
                                value: value["value"].as_u64().unwrap(),
                                aggregation_method: value["aggregation_method"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(), // always present
                                aggregation_rank: Some(
                                    value["aggregation_rank"].as_str().unwrap_or("").to_string(),
                                ),
                            };
                            records.haploid.push(get_values);
                        }
                        _ => {}
                    }
                }
            }
            None => {
                // do nothing
            }
        };
    }

    Ok(())
}
