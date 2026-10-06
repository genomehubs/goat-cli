//! Exercise the argument handling of every subcommand, from parsing through
//! to the URLs that would be requested. No network requests are made.

use clap::Parser;
use goat_cli::cli::{
    build_cli, write_completions, AssemblyCommand, Cli, Index, SearchRequest, TaxonCommand,
};
use goat_cli::report::report::{Report, ReportOptions, ReportType};
use goat_cli::utils::cli_matches::{process_cli_args, CliAction};
use goat_cli::utils::utils::{generate_unique_ids, taxa_from_input};

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(std::iter::once("goat-cli").chain(args.iter().copied()))
        .unwrap_or_else(|e| panic!("failed to parse {:?}: {}", args, e))
}

/// The `search`/`count` request for `args`.
fn search_request(args: &[&str]) -> SearchRequest {
    match parse(args).index {
        Index::Taxon { command: TaxonCommand::Search(a) | TaxonCommand::Count(a) } => {
            SearchRequest::from(&a)
        }
        Index::Assembly { command: AssemblyCommand::Search(a) | AssemblyCommand::Count(a) } => {
            SearchRequest::from(&a)
        }
        other => panic!("not a search or count: {:?}", other),
    }
}

/// The report options and type for a report subcommand.
fn report_options(args: &[&str]) -> (ReportOptions, ReportType) {
    match parse(args).index {
        Index::Taxon { command } => match command {
            TaxonCommand::Sources(a) => (ReportOptions::from(&a), ReportType::Sources),
            TaxonCommand::Newick(a) => (ReportOptions::from(&a), ReportType::Newick),
            TaxonCommand::Hist(a) => (ReportOptions::from(&a), ReportType::Histogram),
            TaxonCommand::Scatter(a) => (ReportOptions::from(&a), ReportType::Scatterplot),
            TaxonCommand::Arc(a) => (ReportOptions::from(&a), ReportType::Arc),
            other => panic!("not a report: {:?}", other),
        },
        other => panic!("not a report: {:?}", other),
    }
}

fn search_urls(args: &[&str]) -> Vec<String> {
    let request = search_request(args);
    let query = &request.query;
    let n = taxa_from_input(query.taxon.as_deref(), query.file.as_deref(), query.expression.is_some())
        .unwrap()
        .len();
    match process_cli_args(&request, "search", generate_unique_ids(n)).unwrap() {
        CliAction::Continue { urls, .. } => urls,
        CliAction::PrintedAndExit => panic!("unexpected PrintedAndExit"),
    }
}

/// The decoded `query` parameter of a URL.
fn query_param(url: &str) -> String {
    let query = url.split("query=").nth(1).unwrap().split('&').next().unwrap();
    query.replace("%20", " ").replace("%28", "(").replace("%29", ")").replace("%2C", ",")
}

#[test]
fn test_cli_debug_assert() {
    build_cli().debug_assert();
}

#[test]
fn test_search_and_count_arg_handling() {
    for index in ["taxon", "assembly"] {
        for (api, extra) in [("search", "-i"), ("count", "-d")] {
            let urls = search_urls(&[index, api, "-t", "Mammalia", extra]);
            assert_eq!(urls.len(), 1);
            assert!(urls[0].contains(&format!("result={}", index)), "{}", urls[0]);
        }
    }
}

#[test]
fn test_field_flags_fill_field_builder() {
    let request = search_request(&["taxon", "search", "-t", "Mammalia", "-G", "-r"]);
    assert!(request.fields.taxon_gs);
    assert!(request.include_raw_values);
    // raw values are always tidy
    assert!(request.fields.taxon_tidy);
    assert!(!request.fields.assembly_assembly);

    let request = search_request(&["assembly", "search", "-t", "Mammalia", "-a", "--btk"]);
    assert!(request.fields.assembly_assembly && request.fields.assembly_btk);
    assert!(!request.fields.taxon_assembly);
}

#[test]
fn test_lineage_flag_builds_tax_lineage_query() {
    for index in ["taxon", "assembly"] {
        let urls = search_urls(&[index, "search", "-t", "Mammalia", "-l"]);
        assert!(urls[0].contains("tax_lineage"), "{}", urls[0]);
    }
}

#[test]
fn test_lineage_conflicts_with_descendents() {
    for index in ["taxon", "assembly"] {
        let result = Cli::try_parse_from(["goat-cli", index, "search", "-t", "Mammalia", "-l", "-d"]);
        assert!(result.is_err());
    }
}

#[test]
fn test_taxon_include_estimates_conflicts_with_raw() {
    let result = Cli::try_parse_from([
        "goat-cli", "taxon", "search", "-t", "Mammalia", "-i", "--raw",
    ]);
    assert!(result.is_err());
}

#[test]
fn test_report_arg_handling() {
    let cases: &[&[&str]] = &[
        &["taxon", "newick", "-t", "Mammalia"],
        &["taxon", "sources", "-t", "Mammalia"],
        &["taxon", "hist", "-t", "Mammalia", "-x", "genome_size", "-c", "genus", "-s", "5"],
        &["taxon", "scatter", "-t", "Mammalia", "-x", "genome_size", "-y", "c_value"],
        &["taxon", "arc", "-x", "assembly_span"],
        &["taxon", "arc", "-t", "Mammalia", "-x", "assembly_span"],
    ];
    for args in cases {
        let (options, report_type) = report_options(args);
        let report = Report::new(&options, report_type).unwrap();
        let url = report.make_url(vec!["test_id".into()]).unwrap();
        assert!(url.contains("report="), "{:?} -> {}", args, url);
    }
}

#[test]
fn test_report_reads_taxa_from_file() {
    let path = std::env::temp_dir().join(format!("goat_cli_report_taxa_{}.txt", std::process::id()));
    std::fs::write(&path, "Primates\nCetacea\n").unwrap();
    let (options, report_type) =
        report_options(&["taxon", "hist", "-f", path.to_str().unwrap(), "-x", "genome_size"]);
    let url = Report::new(&options, report_type)
        .unwrap()
        .make_url(vec!["id".into()])
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(query_x(&url).contains("tax_tree(Primates,Cetacea)"), "{}", url);
}

/// The decoded `x` parameter of a report URL.
fn query_x(url: &str) -> String {
    let x = url.split("&x=").nth(1).unwrap().split('&').next().unwrap();
    x.replace("%20", " ").replace("%28", "(").replace("%29", ")").replace("%2C", ",")
}

#[test]
fn test_newick_threshold_accepts_minus_one_only() {
    let (options, _) = report_options(&["taxon", "newick", "-t", "Mammalia", "--threshold", "-1"]);
    assert_eq!(options.threshold, -1);
    assert!(Cli::try_parse_from(["goat-cli", "taxon", "newick", "-t", "X", "--threshold", "-5"]).is_err());
}

#[test]
fn test_expression_without_taxon_searches_all_taxa() {
    let urls = search_urls(&["taxon", "search", "-e", "genome_size > 1e11"]);
    assert_eq!(urls.len(), 1);
    assert_eq!(query_param(&urls[0]), "genome_size %3E 1e11");
}

#[test]
fn test_neither_taxon_nor_expression_is_rejected_by_clap() {
    let result = Cli::try_parse_from(["goat-cli", "taxon", "search", "-v", "genome_size"]);
    assert!(result.is_err());
}

#[test]
fn test_print_expression_needs_no_taxon() {
    let request = search_request(&["assembly", "count", "--print-expression"]);
    assert!(request.output.print_expression);
}

#[test]
fn test_or_expression_applies_taxon_to_each_branch() {
    let urls = search_urls(&[
        "assembly", "search", "-t", "Hominidae", "-d", "-e",
        "assembly_level = chromosome OR assembly_span > 3e9",
    ]);
    assert_eq!(query_param(&urls[0]).matches("tax_tree(Hominidae)").count(), 2, "{}", urls[0]);
}

#[test]
fn test_empty_taxon_list_is_an_error() {
    let err = taxa_from_input(Some(" , "), None, false).unwrap_err();
    assert!(err.to_string().contains("no taxa found"), "{}", err);
}

#[test]
fn test_completions_for_every_shell() {
    use clap::ValueEnum;
    for shell in clap_complete::Shell::value_variants() {
        let mut script = Vec::new();
        write_completions(*shell, &mut script).unwrap();
        let script = String::from_utf8(script).unwrap();
        // completes nested subcommands and their flags
        for word in ["taxon", "assembly", "search", "include-estimates", "x-filter"] {
            assert!(script.contains(word), "{:?} completions lack {}", shell, word);
        }
    }
}

#[test]
fn test_completions_subcommand_parses() {
    match parse(&["completions", "zsh"]).index {
        Index::Completions { shell } => assert_eq!(shell, clap_complete::Shell::Zsh),
        other => panic!("{:?}", other),
    }
    assert!(Cli::try_parse_from(["goat-cli", "completions", "tcsh"]).is_err());
}
