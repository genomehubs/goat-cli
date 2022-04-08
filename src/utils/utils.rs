use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{bail, Context, Result};

// read taxids or binomials from file.
pub fn lines_from_file(filename: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = File::open(&filename)
        .with_context(|| format!("Could not open {:?}", filename.as_ref().as_os_str()))?;
    let buf = BufReader::new(file);
    let buf_res: Result<Vec<String>> = buf
        .lines()
        .map(|l| {
            let x = l.with_context(|| {
                format!(
                    "Error in mapping buf_lines from {:?}",
                    filename.as_ref().as_os_str()
                )
            });
            x
        })
        .collect();
    Ok(buf_res?)
}

// taxids should be comma separated
// remove whitespace from beginning and end of each element of the vec.
// TODO: check structure of each element in vec.
pub fn parse_comma_separated(taxids: &str) -> Vec<String> {
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
        // in addition, remove any quotes
        // so we can parse things like:
        // `-v"assembly_level"`, where there is
        // no space between the `-v` and `assembly_level`
        let replaced = str.replace("\"", "").replace("'", "");

        res2.push(replaced);
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
    // is there a way to get all the headers, and compare them...
    let mut headers = Vec::new();
    for el in &awaited_fetches {
        let tsv = match el {
            Ok(ref e) => e,
            Err(e) => bail!("{}", e),
        };
        headers.push(tsv.split("\n").next());
    }

    // mainly a guard - but Rich I think fixed this so shouldn't need to be done.
    let header = headers.iter().fold(headers[0], |acc, &item| {
        let acc = acc?;
        let item = item?;
        if item.len() > acc.len() {
            Some(item)
        } else {
            Some(acc)
        }
    });

    match header {
        Some(h) => println!("{}", h),
        None => bail!("No header found."),
    }

    for el in awaited_fetches {
        let tsv = match el {
            Ok(ref e) => e,
            Err(e) => bail!("{}", e),
        };

        let tsv_iter = tsv.split("\n");
        for row in tsv_iter.skip(1) {
            println!("{}", row)
        }
    }

    Ok(())
}

// https://stackoverflow.com/questions/38406793/why-is-capitalizing-the-first-letter-of-a-string-so-convoluted-in-rust
pub fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// https://stackoverflow.com/questions/26998485/is-it-possible-to-print-a-number-formatted-with-thousand-separator-in-rust
// for error messages above cli query limit
pub fn pretty_print_usize(i: usize) -> String {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    format!("{}", s)
}

// not sure this needs to be done... Rich?
pub fn switch_string_to_url_encoding(string: &str) -> Result<&str> {
    let res = match string {
        // "!=" => "%21%3D",
        "!=" => "!%3D",
        // "<" => "%3C",
        "<" => "<",
        // "<=" => "%3C%3D",
        "<=" => "<%3D",
        "=" => "%3D",
        "==" => "%3D%3D",
        // ">" => "%3E",
        ">" => ">",
        // ">=" => "%3E%3D",
        ">=" => ">%3D",
        _ => bail!("Should not reach here."),
    };
    Ok(res)
}
