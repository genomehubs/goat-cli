//! Exercise the argument handling of every subcommand, so that a mismatch
//! between `cli.rs` and the code reading the matches is caught by the test
//! suite rather than by a panic at runtime. No network requests are made.

use goat_cli::cli::build_cli;
use goat_cli::report::report::{Report, ReportType};
use goat_cli::utils::cli_matches::{process_cli_args, CliAction};
use goat_cli::utils::utils::{generate_unique_strings, taxa_from_matches, UniqueIdAction};
use goat_cli::IndexType;

/// Parse `args` and return the matches of the innermost subcommand.
fn leaf_matches(args: &[&str]) -> clap::ArgMatches {
    let mut matches = build_cli()
        .try_get_matches_from(std::iter::once("goat-cli").chain(args.iter().copied()))
        .unwrap_or_else(|e| panic!("failed to parse {:?}: {}", args, e));
    while let Some((_, sub)) = matches.subcommand() {
        matches = sub.clone();
    }
    matches
}

fn unique_ids(matches: &clap::ArgMatches, index_type: IndexType) -> Vec<String> {
    match generate_unique_strings(matches, index_type).unwrap() {
        UniqueIdAction::Continue(ids) => ids,
        UniqueIdAction::PrintedAndExit => panic!("unexpected PrintedAndExit"),
    }
}

#[test]
fn test_cli_debug_assert() {
    build_cli().debug_assert();
}

#[test]
fn test_search_and_count_arg_handling() {
    for (index, index_type) in [("taxon", IndexType::Taxon), ("assembly", IndexType::Assembly)] {
        for (api, extra) in [("search", "-i"), ("count", "-d")] {
            let matches = leaf_matches(&[index, api, "-t", "Mammalia", extra]);
            let ids = unique_ids(&matches, index_type);
            match process_cli_args(&matches, api, ids, index_type).unwrap() {
                CliAction::Continue { urls, .. } => assert_eq!(urls.len(), 1),
                CliAction::PrintedAndExit => panic!("unexpected PrintedAndExit"),
            }
        }
    }
}

#[test]
fn test_lineage_flag_builds_tax_lineage_query() {
    for (index, index_type) in [("taxon", IndexType::Taxon), ("assembly", IndexType::Assembly)] {
        let matches = leaf_matches(&[index, "search", "-t", "Mammalia", "-l"]);
        let ids = unique_ids(&matches, index_type);
        match process_cli_args(&matches, "search", ids, index_type).unwrap() {
            CliAction::Continue { urls, .. } => assert!(urls[0].contains("tax_lineage")),
            CliAction::PrintedAndExit => panic!("unexpected PrintedAndExit"),
        }
    }
}

#[test]
fn test_taxon_include_estimates_conflicts_with_raw() {
    let result = build_cli().try_get_matches_from([
        "goat-cli", "taxon", "search", "-t", "Mammalia", "-i", "--raw",
    ]);
    assert!(result.is_err());
}

#[test]
fn test_report_arg_handling() {
    let cases: &[(&[&str], ReportType)] = &[
        (&["taxon", "newick", "-t", "Mammalia"], ReportType::Newick),
        (&["taxon", "sources", "-t", "Mammalia"], ReportType::Sources),
        (
            &["taxon", "hist", "-t", "Mammalia", "-x", "genome_size", "-c", "genus", "-s", "5"],
            ReportType::Histogram,
        ),
        (
            &["taxon", "scatter", "-t", "Mammalia", "-x", "genome_size", "-y", "c_value"],
            ReportType::Scatterplot,
        ),
        (&["taxon", "arc", "-x", "assembly_span"], ReportType::Arc),
        (&["taxon", "arc", "-t", "Mammalia", "-x", "assembly_span"], ReportType::Arc),
    ];
    for (args, report_type) in cases {
        let matches = leaf_matches(args);
        let report = Report::new(&matches, *report_type).unwrap();
        let url = report.make_url(vec!["test_id".into()]).unwrap();
        assert!(url.contains("report="), "{:?} -> {}", args, url);
    }
}

#[test]
fn test_unique_ids_for_report_subcommands() {
    for args in [
        &["taxon", "newick", "-t", "Mammalia,Aves"][..],
        &["taxon", "sources", "-t", "Mammalia,Aves"][..],
        &["taxon", "hist", "-t", "Mammalia,Aves", "-x", "genome_size"][..],
    ] {
        let matches = leaf_matches(args);
        assert_eq!(unique_ids(&matches, IndexType::Taxon).len(), 2);
    }
}

fn search_urls(args: &[&str], index_type: IndexType) -> Vec<String> {
    let matches = leaf_matches(args);
    let ids = unique_ids(&matches, index_type);
    match process_cli_args(&matches, "search", ids, index_type).unwrap() {
        CliAction::Continue { urls, .. } => urls,
        CliAction::PrintedAndExit => panic!("unexpected PrintedAndExit"),
    }
}

#[test]
fn test_expression_without_taxon_searches_all_taxa() {
    let urls = search_urls(&["taxon", "search", "-e", "genome_size > 1e11"], IndexType::Taxon);
    assert_eq!(urls.len(), 1);
    assert!(urls[0].contains("query=genome_size%20%3E%201e11&"), "{}", urls[0]);
}

#[test]
fn test_neither_taxon_nor_expression_is_rejected_by_clap() {
    let result = build_cli().try_get_matches_from(["goat-cli", "taxon", "search", "-v", "genome_size"]);
    assert!(result.is_err());
}

#[test]
fn test_or_expression_applies_taxon_to_each_branch() {
    let urls = search_urls(
        &["assembly", "search", "-t", "Hominidae", "-d", "-e", "assembly_level = chromosome OR assembly_span > 3e9"],
        IndexType::Assembly,
    );
    let query = urls[0].split("query=").nth(1).unwrap().split('&').next().unwrap();
    assert_eq!(query.matches("tax_tree%28Hominidae%29").count(), 2, "{}", query);
}

#[test]
fn test_empty_taxon_list_is_an_error() {
    let matches = leaf_matches(&["taxon", "search", "-t", " , "]);
    let err = taxa_from_matches(&matches).unwrap_err();
    assert!(err.to_string().contains("no taxa found"), "{}", err);
}
