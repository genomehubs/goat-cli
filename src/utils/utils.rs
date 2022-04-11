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

// levenshtein suggestions
// taken from nushell
// https://github.com/nushell/nushell/blob/690ec9abfa994e6cf8b85ec38173ee5f0c91011c/crates/nu-protocol/src/shell_error.rs

pub fn did_you_mean(possibilities: &[String], tried: &str) -> Option<String> {
    let mut possible_matches: Vec<_> = possibilities
        .iter()
        .map(|word| {
            let edit_distance = levenshtein_distance(&word.to_lowercase(), &tried.to_lowercase());
            (edit_distance, word.to_owned())
        })
        .collect();

    possible_matches.sort();

    if let Some((_, first)) = possible_matches.into_iter().next() {
        Some(first)
    } else {
        None
    }
}

// Borrowed from here https://github.com/wooorm/levenshtein-rs
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let mut result = 0;

    /* Shortcut optimizations / degenerate cases. */
    if a == b {
        return result;
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 {
        return length_b;
    }

    if length_b == 0 {
        return length_a;
    }

    /* Initialize the vector.
     *
     * This is why it’s fast, normally a matrix is used,
     * here we use a single vector. */
    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result
}
