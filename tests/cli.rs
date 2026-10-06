//! Exercise the argument handling of every subcommand, so that a mismatch
//! between `cli.rs` and the code reading the matches is caught by the test
//! suite rather than by a panic at runtime. No network requests are made.

use goat_cli::cli::build_cli;
use goat_cli::report::report::{Report, ReportType};
use goat_cli::utils::cli_matches::{process_cli_args, CliAction};
use goat_cli::utils::utils::{generate_unique_strings, UniqueIdAction};
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
