use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::error::{Error, ErrorKind, Result};
use crate::utils::args::ArgMatchesExt;
use crate::{
    utils::expression,
    utils::variable_data::{GOAT_ASSEMBLY_VARIABLE_DATA, GOAT_TAXON_VARIABLE_DATA},
    IndexType, UPPER_CLI_FILE_LIMIT,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub enum UniqueIdAction {
    Continue(Vec<String>),
    PrintedAndExit,
}

/// Determine from the CLI matches how many URLs
/// are needing to be generated, and return a
/// vector of random character strings to use as
/// unique identifiers.
pub fn generate_unique_strings(
    matches: &clap::ArgMatches,
    index_type: IndexType,
) -> Result<UniqueIdAction> {
    // print expression table
    // got to include this here, otherwise we error.
    // reports don't include this.
    let print_expression = matches.opt_one::<bool>("print-expression");

    if let Some(p) = print_expression {
        if *p {
            match index_type {
                IndexType::Taxon => expression::print_variable_data(&GOAT_TAXON_VARIABLE_DATA)?,
                IndexType::Assembly => {
                    expression::print_variable_data(&GOAT_ASSEMBLY_VARIABLE_DATA)?
                }
            }
            return Ok(UniqueIdAction::PrintedAndExit);
        }
    }

    let url_vector_len = taxa_from_matches(matches)?.len();

    let mut chars_vec = vec![];
    for _ in 0..url_vector_len {
        let mut rng = thread_rng();
        let chars: String = (0..15).map(|_| rng.sample(Alphanumeric) as char).collect();
        chars_vec.push(chars.clone());
    }

    Ok(UniqueIdAction::Continue(chars_vec))
}

/// The taxa to query, from `-t` (comma separated) or `-f` (one per line).
///
/// If neither is given but an expression (`-e`) is, returns a single empty
/// string, meaning a query across all taxa.
pub fn taxa_from_matches(matches: &clap::ArgMatches) -> Result<Vec<String>> {
    let taxa = if let Some(taxon) = matches.opt_one::<String>("taxon") {
        let taxa = parse_comma_separated(taxon);
        if taxa.is_empty() {
            return Err(Error::new(ErrorKind::GenericCli(
                "no taxa found in -t (--taxon), please specify a taxon.".to_string(),
            )));
        }
        taxa
    } else if let Some(file) = matches.opt_one::<PathBuf>("file") {
        let taxa = lines_from_file(file)?;
        if taxa.is_empty() {
            return Err(Error::new(ErrorKind::GenericCli(format!(
                "no taxa found in {}.",
                file.display()
            ))));
        }
        taxa
    } else if matches.opt_one::<String>("expression").is_some() {
        return Ok(vec![String::new()]);
    } else {
        return Err(Error::new(ErrorKind::GenericCli(
            "one of -f (--file) or -t (--taxon) should be specified.".to_string(),
        )));
    };

    if taxa.len() > *UPPER_CLI_FILE_LIMIT {
        return Err(Error::new(ErrorKind::GenericCli(format!(
            "number of taxa specified cannot exceed {}.",
            pretty_print_usize(*UPPER_CLI_FILE_LIMIT)
        ))));
    }
    Ok(taxa)
}

/// Generate a single random query ID, for use when no taxon input is needed.
pub fn generate_one_unique_id() -> String {
    let mut rng = thread_rng();
    (0..15).map(|_| rng.sample(Alphanumeric) as char).collect()
}

/// Read NCBI taxon ID's or binomial names of species,
/// or higher order taxa from a file.
///
/// Lines are trimmed, and blank lines and `#` comments are skipped.
pub fn lines_from_file(filename: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = File::open(&filename)?;
    let mut lines = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            lines.push(line.to_string());
        }
    }
    Ok(lines)
}

// taxids should be comma separated
// remove whitespace from beginning and end of each element of the vec.
// TODO: check structure of each element in vec.

pub fn parse_comma_separated(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim())
        .map(|s| s.replace(['\"', '\''], ""))
        .filter(|s| !s.is_empty())
        .collect()
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
    // return the first failed request's error as is
    let tsvs = awaited_fetches.into_iter().collect::<Result<Vec<String>>>()?;
    let headers = tsvs.iter().map(|tsv| tsv.split('\n').next()).collect::<Vec<_>>();

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

    let mut out = BufWriter::new(std::io::stdout().lock());

    match header {
        Some(h) => writeln!(out, "{}", h)?,
        None => {
            return Err(Error::new(ErrorKind::FormatTSV(
                "no header found (please report if you get this error!)".to_string(),
            )))
        }
    }

    for tsv in &tsvs {
        let tsv_iter = tsv.split('\n');
        for row in tsv_iter.skip(1) {
            writeln!(out, "{}", row)?;
        }
    }

    out.flush()?;
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
    let i_str = i.to_string();
    let mut chars: Vec<char> = Vec::with_capacity(i_str.len() + i_str.len() / 3);
    for (idx, val) in i_str.chars().rev().enumerate() {
        if idx != 0 && idx % 3 == 0 {
            chars.push(',');
        }
        chars.push(val);
    }
    chars.iter().rev().collect()
}

/// A function to replace certain combinations of characters
/// as their URL encoded variations. Not entirely sure if this is
/// necessary.
pub fn switch_string_to_url_encoding(string: &str) -> Result<&str> {
    let res = match string {
        "=!" => "%3D%21",
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
                "should not reach here.".to_string(),
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

#[cfg(test)]
mod tests {
    use super::{lines_from_file, parse_comma_separated, pretty_print_usize};
    #[test]
    fn test_parse_comma_separated_trims_and_preserves_order() {
        let parsed = parse_comma_separated(" Mammalia, Aves ,Reptilia ");
        assert_eq!(parsed, vec!["Mammalia", "Aves", "Reptilia"]);
    }

    #[test]
    fn test_parse_comma_separated_removes_quotes() {
        let parsed = parse_comma_separated("'assembly_level',\"busco_complete\"");
        assert_eq!(parsed, vec!["assembly_level", "busco_complete"]);
    }

    #[test]
    fn test_parse_comma_separated_does_not_sort_or_dedup() {
        let parsed = parse_comma_separated("zeta,alpha,zeta");
        assert_eq!(parsed, vec!["zeta", "alpha", "zeta"]);
    }

    #[test]
    fn test_parse_comma_separated_drops_empty_entries() {
        let parsed = parse_comma_separated("Mammalia,,Aves,   ,Reptilia");
        assert_eq!(parsed, vec!["Mammalia", "Aves", "Reptilia"]);
    }

    #[test]
    fn test_lines_from_file_trims_and_skips_blanks_and_comments() {
        let path = std::env::temp_dir().join(format!("goat_cli_taxa_{}.txt", std::process::id()));
        std::fs::write(&path, "# my taxa\nMammalia\n\n  Aves  \r\n   \nReptilia\n").unwrap();
        let lines = lines_from_file(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(lines, vec!["Mammalia", "Aves", "Reptilia"]);
    }

    #[test]
    fn test_pretty_print_usize_zero() {
        assert_eq!(pretty_print_usize(0), "0");
    }

    #[test]
    fn test_pretty_print_usize_below_threshold() {
        assert_eq!(pretty_print_usize(999), "999");
    }

    #[test]
    fn test_pretty_print_usize_thousands() {
        assert_eq!(pretty_print_usize(1000), "1,000");
    }

    #[test]
    fn test_pretty_print_usize_millions() {
        assert_eq!(pretty_print_usize(1_000_000), "1,000,000");
    }

    #[test]
    fn test_pretty_print_usize_mid_range() {
        assert_eq!(pretty_print_usize(12345), "12,345");
    }
}
