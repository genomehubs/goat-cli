use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::error::error::FileError;
use anyhow::{bail, Result};
use serde_json::Value;

// read taxids or binomials from file.
pub fn lines_from_file(filename: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = File::open(filename).map_err(|_| FileError::FileOpenError)?;
    let buf = BufReader::new(file);
    Ok(buf
        .lines()
        .map(|l| l.map_err(|_| FileError::FileOpenError).unwrap())
        .collect())
}

// taxids should be comma separated
// remove whitespace from beginning and end of each element of the vec.
// TODO: check structure of each element in vec.
pub fn parse_multiple_taxids(taxids: &str) -> Vec<String> {
    let res: Vec<&str> = taxids.split(",").map(|s| s).collect();

    let mut res2 = Vec::new();
    for mut str in res {
        // sort the rights
        while str.ends_with(" ") {
            let len = str.len();
            let new_len = len.saturating_sub(" ".len());
            str = &str[..new_len];
        }
        // sort the lefts
        let mut index = 0;
        while str.starts_with(" ") {
            index += 1;
            str = &str[index..];
        }
        res2.push(str.to_string());
    }
    res2.sort_unstable();
    res2.dedup();
    res2
}

// needed so that the output headers for ranks can be formatted properly
// similar to the function `format_rank` but return the vector
// of ranks the user has chosen

pub fn get_rank_vector(r: &str) -> Vec<String> {
    let ranks = vec![
        "subspecies".to_string(),
        "species".to_string(),
        "genus".to_string(),
        "family".to_string(),
        "order".to_string(),
        "class".to_string(),
        "phylum".to_string(),
        "kingdom".to_string(),
        "superkingdom".to_string(),
    ];
    let position_selected = ranks.iter().position(|e| e == &r);
    let updated_ranks = match position_selected {
        Some(p) => ranks[p..].to_vec(),
        None => vec!["".to_string()],
    };

    updated_ranks
}

// make the GoaT API URLs

// function here to make the ranks URL string
// &ranks=subspecies%2Cspecies%2Cgenus%2Cfamily%2Corder%2Cclass%2Cphylum%2Ckingdom%2Csuperkingdom

fn format_rank(r: &str) -> String {
    // fixed vector of ranks.
    // "none" by default will return an empty string here.
    let ranks = vec![
        "subspecies",
        "species",
        "genus",
        "family",
        "order",
        "class",
        "phylum",
        "kingdom",
        "superkingdom",
    ];
    let position_selected = ranks.iter().position(|e| e == &r);
    let updated_ranks = match position_selected {
        Some(p) => &ranks[p..],
        None => return "".to_string(),
    };
    let mut rank_string = String::new();
    rank_string += "&ranks=";
    let ranks_to_add = updated_ranks.join("%2C");
    rank_string += &ranks_to_add;

    rank_string
}

pub fn make_goat_search_urls(
    taxids: Vec<String>,
    goat_url: &str,
    tax_tree: &str,
    include_estimates: bool,
    include_raw_values: bool,
    summarise_values_by: &str,
    result: &str,
    taxonomy: &str,
    size: &str,
    ranks: &str,
) -> Vec<String> {
    let mut res = Vec::new();

    let rank_string = format_rank(ranks);

    for el in taxids {
        let url = format!(
        "{}search?query=tax_{}%28{}%29&includeEstimates={}&includeRawValues={}&summaryValues={}&result={}&taxonomy={}&size={}{}",
        goat_url, tax_tree, el, include_estimates, include_raw_values, summarise_values_by, result, taxonomy, size, rank_string
    );
        res.push(url);
    }
    res
}

// check if number of hits > size of query

pub fn check_number_hits(v: &Value, size: &str) -> Result<()> {
    // parse size to i32
    let size_int: u64 = size.parse()?;
    // get value from JSON response
    let hits = v["status"]["hits"].as_u64();

    match hits {
        Some(hits) => {
            if size_int < hits {
                eprintln!(
                    "[-]\tOnly {} results are displayed, but there are {} hits from GoaT.",
                    size_int, hits
                );
            }
        }
        None => {
            bail!("[-]\tThere were no hits.")
        }
    }

    Ok(())
}
