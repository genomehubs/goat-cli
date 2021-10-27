use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::error::error::FileError;
use anyhow::Result;

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

// make the GoaT API URLs
// potentially add ranks
// ranks=species%2Cgenus%2Cfamily%2Corder%2Cclass%2Cphylum%2Ckingdom%2Csuperkingdom
// or the mitochondrial/plastid genome attributes
// fields=assembly_level%2Cassembly_span%2CBUSCO%20completeness%2Cmitochondrion_assembly_span%2Cmitochondrion_gc_percent%2Cplastid_assembly_span%2Cplastid_gc_percent%2Cchromosome_number%2Chaploid_number%2Cc_value%2Cgenome_size
//

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
) -> Vec<String> {
    let mut res = Vec::new();
    for el in taxids {
        let url = format!(
        "{}search?query=tax_{}%28{}%29&includeEstimates={}&includeRawValues={}&summaryValues={}&result={}&taxonomy={}&size={}",
        goat_url, tax_tree, el, include_estimates, include_raw_values, summarise_values_by, result, taxonomy, size
    );
        res.push(url);
    }
    res
}

// parse a file with tax-id or names
