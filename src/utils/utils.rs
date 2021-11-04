use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::error::error::FileError;
use anyhow::{bail, Result};

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

// if you query multiple taxa, headers pop up for every new taxon.

pub fn format_tsv_output(awaited_fetches: Vec<Result<String, anyhow::Error>>) -> Result<()> {
    // if there is a single element, return this.
    if awaited_fetches.len() == 1 {
        let first = awaited_fetches.get(0);
        match first {
            Some(s) => match s {
                Ok(s) => {
                    println!("{}", s);
                }
                Err(e) => bail!("{}", e),
            },
            None => bail!("There were no results."),
        }
    } else {
        let mut index = 0;
        for el in awaited_fetches {
            let tsv = match el {
                Ok(ref e) => e,
                Err(e) => bail!("{}", e),
            };
            if index == 0 {
                println!("{}", tsv);
            } else {
                let tsv_iter = tsv.split("\n");
                for row in tsv_iter.skip(1) {
                    println!("{}", row)
                }
            }
            index += 1;
        }
    }

    Ok(())
}
