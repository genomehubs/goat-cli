use std::fmt::{Display, Error, Formatter};

use serde_json::Value;

// get the ranks from a GoaT API ping from 'search'
// I think the user is going to have to specify the ranks
// that they want. It's easier for me. Less thinking.

pub fn get_ranks(v: &Value, index: usize, ref_ranks: &Vec<String>) -> Option<Vec<String>> {
    // take borrowed Value, and subset
    let map_of_ranks_op = v["results"][index]["result"]["ranks"].as_object();

    let mut rank_vec = Vec::new();

    match map_of_ranks_op {
        Some(map) => {
            for (_key, value) in map {
                // key is the rank, value will give us the name.
                let rank_op = value["scientific_name"].as_str();
                let rank_rank_op = value["taxon_rank"].as_str();

                // not sure if this is the right thing to do
                // for rank_rank.
                let rank_rank = match rank_rank_op {
                    Some(r) => r,
                    None => "",
                };
                let rank = match rank_op {
                    Some(r) => r,
                    None => "",
                };
                // push a tuple so we can sort on rank,
                // for some reason, when I get the ranks out
                // they are in strange order
                rank_vec.push((rank_rank.to_string(), rank.to_string()));
            }
        }
        None => {}
    }

    // updated vec
    let mut updated_rank_vec = Vec::new();

    for el in ref_ranks {
        let mut is_in_vec = false;
        for el2 in &rank_vec {
            if *el == el2.0 {
                updated_rank_vec.push(el2.1.clone());
                is_in_vec = true;
            }
        }
        // a guard to not mess up the column number
        // in the TSV. Sometimes there are higher orders
        // presented from a search than what was specified, 
        // in which case we pad with "None" values.
        if !is_in_vec && !updated_rank_vec.is_empty() {
            updated_rank_vec.push("None".to_string());
        }
    }
    updated_rank_vec.reverse();
    Some(updated_rank_vec)
}

// for displaying tab separated vectors of strings
// specifically here, the taxonomic ranks

// I wrote this in a flurry. 
// TODO: needs some QC.

#[derive(Clone)]
pub struct Ranks(pub Option<Vec<String>>);

impl Display for Ranks {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut tab_separated = String::new();

        let result = match &self.0 {
            Some(r) => r.clone(),
            None => ["".to_string()].to_vec(),
        };

        let mut cloned_result = result.clone();
        if cloned_result.is_empty() {
            cloned_result.push("".to_string());
        }

        for rank in &cloned_result[0..cloned_result.len() - 1] {
            tab_separated.push_str(&rank.to_string());
            if rank != &"".to_string() {
                tab_separated.push_str("\t");
            }
        }

        tab_separated.push_str(&cloned_result[cloned_result.len() - 1]);
        if cloned_result.len() > 1 {
            write!(f, "{}\t", tab_separated)
        } else {
            write!(f, "{}", tab_separated)
        }
    }
}

pub struct RankHeaders(pub Vec<String>);

impl Display for RankHeaders {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut tab_separated = String::new();

        let length = self.0.len();

        // not sure why they are in reverse order, but they are.
        // reallocate here, because Rust.
        let mut reversed = self.0.clone();
        reversed.reverse();

        for num in &reversed[0..length - 1] {
            tab_separated.push_str(&num.to_string());
            tab_separated.push_str("\t");
        }

        tab_separated.push_str(&reversed[length - 1].to_string());
        if reversed.len() > 1 {
            write!(f, "{}\t", tab_separated)
        } else {
            write!(f, "{}", tab_separated)
        }
    }
}
