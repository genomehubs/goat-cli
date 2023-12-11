use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::error::{Error, ErrorKind, Result};
use crate::{
    utils::expression,
    utils::variable_data::{GOAT_ASSEMBLY_VARIABLE_DATA, GOAT_TAXON_VARIABLE_DATA},
    IndexType, UPPER_CLI_FILE_LIMIT,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Determine from the CLI matches how many URLs
/// are needing to be generated, and return a
/// vector of random character strings to use as
/// unique identifiers.
pub fn generate_unique_strings(
    matches: &clap::ArgMatches,
    index_type: IndexType,
) -> Result<Vec<String>> {
    let tax_name_op = matches.get_one::<String>("taxon");
    let filename_op = matches.get_one::<PathBuf>("file");
    // print expression table
    // got to include this here, otherwise we error.
    // reports don't include this.
    let print_expression = matches.get_one::<bool>("print-expression");

    if let Some(p) = print_expression {
        if *p {
            match index_type {
                IndexType::Taxon => expression::print_variable_data(&GOAT_TAXON_VARIABLE_DATA),
                IndexType::Assembly => {
                    expression::print_variable_data(&GOAT_ASSEMBLY_VARIABLE_DATA)
                }
            }
            std::process::exit(0);
        }
    }

    let url_vector: Vec<String>;
    // if -t use this
    match tax_name_op {
        Some(s) => {
            // catch empty string hanging here.
            if s.is_empty() {
                return Err(Error::new(ErrorKind::GenericCli(
                    "Empty string found, please specify a taxon.".to_string(),
                )));
            }
            url_vector = parse_comma_separated(s);
        }
        None => match filename_op {
            Some(s) => {
                url_vector = lines_from_file(s)?;
                // check length of vector and bail if > 1000
                if url_vector.len() > *UPPER_CLI_FILE_LIMIT {
                    let limit_string = pretty_print_usize(*UPPER_CLI_FILE_LIMIT);
                    return Err(Error::new(ErrorKind::GenericCli(format!(
                        "Number of taxa specified cannot exceed {}.",
                        limit_string
                    ))));
                }
            }
            None => {
                return Err(Error::new(ErrorKind::GenericCli(
                    "One of -f (--file) or -t (--taxon) should be specified.".to_string(),
                )))
            }
        },
    }

    let url_vector_len = url_vector.len();

    let mut chars_vec = vec![];
    for _ in 0..url_vector_len {
        let mut rng = thread_rng();
        let chars: String = (0..15).map(|_| rng.sample(Alphanumeric) as char).collect();
        chars_vec.push(chars.clone());
    }

    Ok(chars_vec)
}

/// Read NCBI taxon ID's or binomial names of species,
/// or higher order taxa from a file.
pub fn lines_from_file(filename: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = File::open(&filename)?;
    let buf = BufReader::new(file);
    let buf_res = buf.lines().collect::<std::result::Result<Vec<_>, _>>();
    buf_res.map_err(|e| Error::new(ErrorKind::IO(e)))
}

// taxids should be comma separated
// remove whitespace from beginning and end of each element of the vec.
// TODO: check structure of each element in vec.

/// Parse a comma separated string and return each of the elements
/// stripped of whitespace in a vector.
pub fn parse_comma_separated(taxids: &str) -> Vec<String> {
    let res: Vec<&str> = taxids.split(',').collect();

    let mut res2 = Vec::new();
    for mut str in res {
        // sort the rights
        while str.ends_with(' ') {
            let len = str.len();
            let new_len = len.saturating_sub(" ".len());
            str = &str[..new_len];
        }
        // sort the lefts
        let mut index = 0;
        while str.starts_with(' ') {
            index += 1;
            str = &str[index..];
        }
        // in addition, remove any quotes
        // so we can parse things like:
        // `-v"assembly_level"`, where there is
        // no space between the `-v` and `assembly_level`
        let replaced = str.replace(['\"', '\''], "");

        res2.push(replaced);
    }
    res2.sort_unstable();
    res2.dedup();
    res2
}

/// Creates a vector of taxon ranks which will eventually form the
/// headers of the taxon ranks in the returned TSV file.
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
    let position_selected = ranks.iter().position(|e| e == r);
    match position_selected {
        Some(p) => ranks[p..].to_vec(),
        None => vec!["".to_string()],
    }
}

/// If multiple taxa are queried at once, headers will return for every new taxon.
/// We can suppress this by storing the whole return as a string.
pub fn format_tsv_output(awaited_fetches: Vec<Result<String>>) -> Result<()> {
    // if there is a single element, return this.
    // is there a way to get all the headers, and compare them...
    let mut headers = Vec::new();
    for el in &awaited_fetches {
        let tsv = match el {
            Ok(ref e) => e,
            Err(e) => return Err(Error::new(ErrorKind::FormatTSV(e.to_string()))),
        };
        headers.push(tsv.split('\n').next());
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
        None => {
            return Err(Error::new(ErrorKind::FormatTSV(
                "no header found (please report if you get this error!)".to_string(),
            )))
        }
    }

    for el in awaited_fetches {
        let tsv = match el {
            Ok(ref e) => e,
            Err(e) => return Err(e),
        };

        let tsv_iter = tsv.split('\n');
        for row in tsv_iter.skip(1) {
            println!("{}", row)
        }
    }

    Ok(())
}

/// Thanks to [this](https://stackoverflow.com/questions/38406793/why-is-capitalizing-the-first-letter-of-a-string-so-convoluted-in-rust)
/// post on stack overflow. Make a string uppercase on the first character.
pub fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Thanks to  [`this`](https://stackoverflow.com/questions/26998485/is-it-possible-to-print-a-number-formatted-with-thousand-separator-in-rust)
/// post on stack overflow. For error messages above cli query limit, print
/// the [`usize`] prettily.
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
    s.to_string()
}

/// A function to replace certain combinations of characters
/// as their URL encoded variations. Not entirely sure if this is
/// necessary.
pub fn switch_string_to_url_encoding(string: &str) -> Result<&str> {
    let res = match string {
        // "!=" => "%21%3D",
        "!=" => "!%3D",
        // "<" => "%3C",
        "<" => "%3C",
        // "<=" => "%3C%3D",
        "<=" => "<%3D",
        "=" => "%3D",
        "==" => "%3D%3D",
        // ">" => "%3E",
        ">" => "%3E",
        // ">=" => "%3E%3D",
        ">=" => ">%3D",
        _ => {
            // FIXME: probably should have its own error return type
            return Err(Error::new(ErrorKind::GenericCli(
                "Should not reach here.".to_string(),
            )));
        }
    };
    Ok(res)
}

/// Shamelessly poached from the [Nushell core code](https://github.com/nushell/nushell/blob/690ec9abfa994e6cf8b85ec38173ee5f0c91011c/crates/nu-protocol/src/shell_error.rs).
/// Suggest the closest match to a string.
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

/// Compute the Levenshtein distance between two strings.
/// Borrowed from [here](https://github.com/wooorm/levenshtein-rs).
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
