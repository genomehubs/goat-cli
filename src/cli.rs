//! The command line interface, defined with clap's derive API.
//!
//! Each subcommand's arguments are a typed struct, so the code using them is
//! checked at compile time.
//!
//! Help text is given with explicit `help = ...` attributes rather than doc
//! comments, because clap derive turns doc comments on these types into
//! (long) help, which would change the `--help` output.

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};

use crate::output::Format;
use crate::report::report::ReportOptions;
use crate::utils::url::FieldBuilder;
use crate::utils::utils::pretty_print_usize;
use crate::{IndexType, UPPER_CLI_FILE_LIMIT, UPPER_CLI_SIZE_LIMIT};

const ABOUT: &str = "Genomes on a Tree. Query metadata across the tree of life.\n\nFor a tutorial on usage, visit: https://github.com/genomehubs/goat-cli/wiki\nVisit the GoaT website here: https://goat.genomehubs.org/";
const SEARCH_TAXON_HELP: &str = "The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.\nMay be omitted if an expression (-e) is given, to search across all taxa.";
const LOOKUP_TAXON_HELP: &str =
    "The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.";
const EXPRESSION_HELP: &str = "Use an expression to filter results server-side, e.g.\n  'genome_size > 1e9 AND assembly_level = chromosome,complete genome'\nClauses are <variable> <operator> <value>, joined by AND / OR (OR binds loosest).\nSee --print-expression for variables, and EXPRESSIONS.md for the full syntax.";
const EXCLUDE_HELP: &str = "Exclude all missing and ancestral values, so that a returned table may contain only direct measures (excluding missing/ancestral). If multiple variables are requested, a row is only returned if all the variables have a direct value. Will only take effect if one or more variables are specified.";
const LINEAGE_HELP: &str = "Displays lineage information. I.e. from this node in the tree go back and give all the nodes to the root. Conflicts with descendents.";
const PRINT_EXPRESSION_HELP: &str = "Print all variables in GoaT currently, with their associated variants.\nUseful for construction of expressions.";
const PROGRESS_BAR_HELP: &str = "Add a progress bar to large queries, to estimate time left.";
const NO_DESCENDENTS_HELP: &str =
    "Do not return values for descendents (i.e. a tax_name() call).";
const SEARCH_FORMAT_HELP: &str =
    "Output format: tsv, csv, or json (search: GoaT's full JSON response; count: the rows as objects).";
const TABLE_FORMAT_HELP: &str = "Output format: tsv, csv, or json (the rows as objects).";
const REPORT_FORMAT_HELP: &str =
    "Output format: a tsv or csv table, or json (GoaT's full report response).";
const OPTS_HELP: &str = "Options for the x-axis. Comma-separated: min,max,tickCount,scale,axisTitle.\nScales: linear, sqrt, log10, log2, log, proportion, ordinal.";

/// Ranks for `-R/--ranks` in search and count.
const RANKS: [&str; 10] = [
    "none",
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
/// Ranks for `-r/--rank` in most reports.
const REPORT_RANKS: [&str; 4] = ["species", "genus", "family", "order"];
/// Ranks for `-r/--rank` in the arc report.
const ARC_RANKS: [&str; 7] = [
    "species", "genus", "family", "order", "phylum", "class", "kingdom",
];

fn file_help() -> String {
    format!("A file of NCBI taxonomy ID's (tips) and/or binomial names.\nEach line should contain a single entry.\nFile size is limited to {} entries.", pretty_print_usize(*UPPER_CLI_FILE_LIMIT))
}

fn size_help() -> String {
    format!(
        "The number of results to return. Max {} currently.",
        pretty_print_usize(*UPPER_CLI_SIZE_LIMIT)
    )
}

/// The `clap::Command` for the CLI, e.g. for generating completions or
/// checking its definition in tests.
pub fn build_cli() -> clap::Command {
    Cli::command()
}

// `goat-cli`
#[derive(Parser, Debug)]
#[command(
    name = "goat-cli",
    bin_name = "goat-cli",
    version,
    propagate_version = true,
    arg_required_else_help = true,
    author = "Max Brown, Richard Challis, Sujai Kumar, Cibele Sotero-Caio <goat@genomehubs.org>",
    about = ABOUT
)]
pub struct Cli {
    #[command(subcommand)]
    pub index: Index,
}

// The index to query.
#[derive(Subcommand, Debug)]
pub enum Index {
    #[command(arg_required_else_help = true, about = "Query by taxon index.")]
    Taxon {
        #[command(subcommand)]
        command: TaxonCommand,
    },
    #[command(arg_required_else_help = true, about = "Query by assembly index.")]
    Assembly {
        #[command(subcommand)]
        command: AssemblyCommand,
    },
    #[command(about = "Print a shell completion script (see the README to install it).")]
    Completions {
        #[arg(value_enum, value_name = "shell", help = "The shell to generate completions for.")]
        shell: clap_complete::Shell,
    },
}

/// Write the completion script for `shell` to `out`.
pub fn write_completions(shell: clap_complete::Shell, out: &mut impl std::io::Write) -> std::io::Result<()> {
    // clap_complete panics on a write error (e.g. a closed pipe), so generate
    // into memory and write that.
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "goat-cli", &mut script);
    out.write_all(&script)
}

// `goat-cli taxon <command>`
#[derive(Subcommand, Debug)]
pub enum TaxonCommand {
    #[command(about = "Query metadata for any taxon across the tree of life by taxon index.")]
    Search(TaxonSearchArgs),
    #[command(
        about = "Return the count of results for any taxon across the tree of life by taxon index."
    )]
    Count(TaxonSearchArgs),
    #[command(about = "Return information relating to a taxon name, e.g. synonyms, authorities.")]
    Lookup(LookupArgs),
    #[command(
        about = "Show the full record for taxa: every field with its value and sources, or their names or lineage."
    )]
    Record(RecordArgs),
    #[command(about = "Get the sources of data for all taxa input.")]
    Sources(SourcesArgs),
    #[command(about = "Generate a newick tree from input taxa.")]
    Newick(NewickArgs),
    #[command(about = "Generate a histogram report from input taxa.")]
    Hist(HistArgs),
    #[command(about = "Generate a scatter (bivariate) report.")]
    Scatter(ScatterArgs),
    #[command(
        about = "Generate an arc report (proportion of taxa meeting a condition).\nOmit --taxon for a global query across all taxa."
    )]
    Arc(ArcArgs),
}

// `goat-cli assembly <command>`
#[derive(Subcommand, Debug)]
pub enum AssemblyCommand {
    #[command(about = "Query metadata for any taxon across the tree of life by assembly index.")]
    Search(AssemblySearchArgs),
    #[command(
        about = "Return the count of results for any taxon across the tree of life by assembly index."
    )]
    Count(AssemblySearchArgs),
    #[command(about = "Return information relating to a taxon name, e.g. synonyms, authorities.")]
    Lookup(LookupArgs),
    #[command(
        about = "Show the full record for assemblies: every field with its value and source, or their identifiers or lineage."
    )]
    Record(RecordArgs),
}

// ── search and count ─────────────────────────────────────────────────────────

// Arguments shared by `search` and `count` in both indexes.
#[derive(Args, Debug, Clone, Default)]
pub struct QueryArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present_any = ["file", "print_expression", "expression"],
        help = SEARCH_TAXON_HELP
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present_any = ["taxon", "print_expression", "expression"],
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short = 'v',
        long,
        value_name = "variables",
        help = "Variable parser. Input a comma separated string of variables."
    )]
    pub variables: Option<String>,
    #[arg(long, value_name = "size", default_value_t = 50, help = size_help())]
    pub size: u64,
    #[arg(
        short = 'R',
        long,
        value_name = "ranks",
        default_value = "none",
        value_parser = RANKS,
        help = "Choose a rank to display with the results. All ranks up to the given rank are displayed."
    )]
    pub ranks: String,
    #[arg(short = 'e', long, value_name = "expression", help = EXPRESSION_HELP)]
    pub expression: Option<String>,
    #[arg(
        long,
        value_name = "tax-rank",
        help = "The taxonomic rank to return the results at."
    )]
    pub tax_rank: Option<String>,
    #[arg(short = 'x', long, help = EXCLUDE_HELP)]
    pub exclude: bool,
    #[arg(
        short = 'd',
        long,
        help = "Get information for all descendents of a common ancestor."
    )]
    pub descendents: bool,
    #[arg(short = 'l', long, conflicts_with = "descendents", help = LINEAGE_HELP)]
    pub lineage: bool,
}

// Output options shared by `search` and `count` in both indexes.
#[derive(Args, Debug, Clone, Default)]
pub struct OutputArgs {
    #[arg(long, help = PRINT_EXPRESSION_HELP)]
    pub print_expression: bool,
    #[arg(long, help = PROGRESS_BAR_HELP)]
    pub progress_bar: bool,
    #[arg(
        short = 'u',
        long,
        help = "Print the underlying GoaT API URL(s). Useful for debugging."
    )]
    pub url: bool,
    #[arg(
        short = 'U',
        long,
        help = "Print the underlying GoaT UI URL(s). View on the browser!"
    )]
    pub goat_ui_url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = SEARCH_FORMAT_HELP
    )]
    pub format: Format,
}

// Field flags for `taxon search` and `taxon count`.
#[derive(Args, Debug, Clone, Default)]
pub struct TaxonFieldArgs {
    #[arg(short = 'a', long, help = "Print assembly data (assembly span, assembly level).")]
    pub assembly: bool,
    #[arg(short = 'b', long, help = "Print BUSCO estimates.")]
    pub busco: bool,
    #[arg(short = 'g', long, help = "Print GC%.")]
    pub gc_percent: bool,
    #[arg(
        short = 'k',
        long,
        help = "Print karyotype data (chromosome number & haploid number)."
    )]
    pub karyotype: bool,
    #[arg(short = 'G', long, help = "Print genome size data.")]
    pub genome_size: bool,
    #[arg(
        short = 'B',
        long,
        help = "Print the bioproject and biosample ID of records."
    )]
    pub bioproject: bool,
    #[arg(short = 'N', long, help = "Print the contig & scaffold n50 of assemblies.")]
    pub n50: bool,
    #[arg(short = 'D', long, help = "Print EBP & assembly dates.")]
    pub date: bool,
    #[arg(long, help = "Print gene count data.")]
    pub gene_count: bool,
    #[arg(
        short = 'm',
        long,
        help = "Print mitochondrial genome assembly size & GC%."
    )]
    pub mitochondria: bool,
    #[arg(short = 'p', long, help = "Print plastid genome assembly size & GC%.")]
    pub plastid: bool,
    #[arg(short = 'S', long, help = "Print sex determination data.")]
    pub sex_determination: bool,
    #[arg(short = 'P', long, help = "Print ploidy estimates.")]
    pub ploidy: bool,
    #[arg(short = 'c', long, help = "Print c-value data.")]
    pub c_values: bool,
    #[arg(long, help = "Print legislation data.")]
    pub legislation: bool,
    #[arg(long, help = "Print target list data associated with each taxon.")]
    pub target_lists: bool,
    #[arg(short = 'C', long, help = "Print list of countries where taxon is found.")]
    pub country_list: bool,
    #[arg(
        long,
        help = "Print all data associated with how far this taxon has progressed with genomic sequencing.\nThis includes sample collection, acquisition, progress in sequencing, and whether submitted to INSDC."
    )]
    pub status: bool,
    #[arg(
        short = 'n',
        long,
        help = "Print all associated name data (synonyms, Tree of Life ID, and common names)."
    )]
    pub names: bool,
    #[arg(short = 'r', long, help = "Print raw values (i.e. no aggregation/summary).")]
    pub raw: bool,
    #[arg(short = 'T', long, help = "Print data in tidy format.")]
    pub tidy: bool,
    #[arg(
        short = 'i',
        long,
        conflicts_with = "raw",
        help = "Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon. Cannot be used with --raw."
    )]
    pub include_estimates: bool,
    #[arg(
        long,
        help = "For each variable specified, return additional columns for direct measures, ancestral inferred, and descendent inferred."
    )]
    pub toggle_direct: bool,
}

// Field flags for `assembly search` and `assembly count`.
#[derive(Args, Debug, Clone, Default)]
pub struct AssemblyFieldArgs {
    #[arg(short = 'a', long, help = "Print assembly data (span & level)")]
    pub assembly: bool,
    #[arg(short = 'k', long, help = "Print karyotype data (chromosome number only).")]
    pub karyotype: bool,
    #[arg(short = 'c', long, help = "Print contig data (count, l50, n50).")]
    pub contig: bool,
    #[arg(short = 's', long, help = "Print scaffold data (count, l50, n50).")]
    pub scaffold: bool,
    #[arg(
        short = 'g',
        long,
        help = "Print gene count data (gene count, non-coding gene count)."
    )]
    pub gene_count: bool,
    #[arg(long, help = "Print GC percent data.")]
    pub gc_percent: bool,
    #[arg(
        short = 'b',
        long,
        help = "Print BUSCO data (BUSCO completeness, lineage, and string)."
    )]
    pub busco: bool,
    #[arg(long, help = "Print BlobToolKit data (no-hit, target).")]
    pub btk: bool,
    #[arg(
        short = 'i',
        long,
        help = "Include ancestral estimates. Omitting this flag includes only direct estimates from a taxon."
    )]
    pub include_estimates: bool,
}

// `goat-cli taxon search` and `goat-cli taxon count`.
#[derive(Args, Debug, Clone, Default)]
pub struct TaxonSearchArgs {
    #[command(flatten)]
    pub query: QueryArgs,
    #[command(flatten)]
    pub fields: TaxonFieldArgs,
    #[command(flatten)]
    pub output: OutputArgs,
}

// `goat-cli assembly search` and `goat-cli assembly count`.
#[derive(Args, Debug, Clone, Default)]
pub struct AssemblySearchArgs {
    #[command(flatten)]
    pub query: QueryArgs,
    #[command(flatten)]
    pub fields: AssemblyFieldArgs,
    #[command(flatten)]
    pub output: OutputArgs,
}

/// Everything `search` and `count` need, from either index's arguments.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// The index to search.
    pub index_type: IndexType,
    /// The taxa and filters.
    pub query: QueryArgs,
    /// What to print.
    pub output: OutputArgs,
    /// Include ancestral estimates.
    pub include_estimates: bool,
    /// Include raw values (taxon index only).
    pub include_raw_values: bool,
    /// The fields selected by flags.
    pub fields: FieldBuilder,
}

impl From<&TaxonSearchArgs> for SearchRequest {
    fn from(args: &TaxonSearchArgs) -> Self {
        let f = &args.fields;
        let fields = FieldBuilder {
            taxon_assembly: f.assembly,
            taxon_bioproject: f.bioproject,
            taxon_busco: f.busco,
            taxon_country_list: f.country_list,
            taxon_cvalues: f.c_values,
            taxon_date: f.date,
            taxon_gc_percent: f.gc_percent,
            taxon_gene_count: f.gene_count,
            taxon_gs: f.genome_size,
            taxon_karyotype: f.karyotype,
            taxon_legislation: f.legislation,
            taxon_mitochondrion: f.mitochondria,
            taxon_names: f.names,
            taxon_n50: f.n50,
            taxon_plastid: f.plastid,
            taxon_ploidy: f.ploidy,
            taxon_sex_determination: f.sex_determination,
            taxon_status: f.status,
            taxon_target_lists: f.target_lists,
            // raw values are always returned in tidy format
            taxon_tidy: f.tidy || f.raw,
            taxon_toggle_direct: f.toggle_direct,
            ..Default::default()
        };
        Self {
            index_type: IndexType::Taxon,
            query: args.query.clone(),
            output: args.output.clone(),
            include_estimates: f.include_estimates,
            include_raw_values: f.raw,
            fields,
        }
    }
}

impl From<&AssemblySearchArgs> for SearchRequest {
    fn from(args: &AssemblySearchArgs) -> Self {
        let f = &args.fields;
        let fields = FieldBuilder {
            assembly_assembly: f.assembly,
            assembly_karyotype: f.karyotype,
            assembly_contig: f.contig,
            assembly_scaffold: f.scaffold,
            assembly_gc: f.gc_percent,
            assembly_gene: f.gene_count,
            assembly_busco: f.busco,
            assembly_btk: f.btk,
            ..Default::default()
        };
        Self {
            index_type: IndexType::Assembly,
            query: args.query.clone(),
            output: args.output.clone(),
            include_estimates: f.include_estimates,
            include_raw_values: false,
            fields,
        }
    }
}

// ── lookup ───────────────────────────────────────────────────────────────────

// `goat-cli taxon lookup` and `goat-cli assembly lookup`.
#[derive(Args, Debug, Clone, Default)]
pub struct LookupArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present = "file",
        help = LOOKUP_TAXON_HELP
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present_any = ["taxon"],
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(short = 'u', long, help = "Print lookup URL.")]
    pub url: bool,
    #[arg(
        short = 's',
        long,
        value_name = "size",
        default_value_t = 10,
        help = "The number of results to return."
    )]
    pub size: u64,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = TABLE_FORMAT_HELP
    )]
    pub format: Format,
}

// ── record ───────────────────────────────────────────────────────────────────

// `goat-cli taxon record` and `goat-cli assembly record`
#[derive(Args, Debug, Clone, Default)]
pub struct RecordArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present = "file",
        help = "The records to show, comma separated: NCBI taxon IDs or names (taxon record), or assembly accessions (assembly record)."
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present = "taxon",
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short = 'n',
        long,
        conflicts_with = "lineage",
        help = "Show names (taxon record) or identifiers (assembly record) instead of fields."
    )]
    pub names: bool,
    #[arg(short = 'l', long, help = "Show the lineage instead of fields.")]
    pub lineage: bool,
    #[arg(short = 'u', long, help = "Print the record URL(s).")]
    pub url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = "Output format: a tsv or csv table, or json (the full records)."
    )]
    pub format: Format,
}

// ── reports ──────────────────────────────────────────────────────────────────

// `goat-cli taxon sources`
#[derive(Args, Debug, Clone)]
pub struct SourcesArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        help = "The taxon to return sources for. Multiple taxa will return the sources for all."
    )]
    pub taxon: String,
    #[arg(
        short = 'r',
        long,
        value_name = "rank",
        default_value = "species",
        value_parser = REPORT_RANKS,
        help = "The rank of the results to return."
    )]
    pub rank: String,
    #[arg(short = 'n', long, help = NO_DESCENDENTS_HELP)]
    pub no_descendents: bool,
    #[arg(short = 'u', long, help = "Print report URL.")]
    pub url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = REPORT_FORMAT_HELP
    )]
    pub format: Format,
}

// `goat-cli taxon newick`
#[derive(Args, Debug, Clone)]
pub struct NewickArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present = "file",
        help = "The taxon to return a newick of. Multiple taxa will return the joint tree."
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present = "taxon",
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(
        long,
        value_name = "threshold",
        default_value_t = 2000,
        value_parser = clap::value_parser!(i32).range(-1..),
        allow_negative_numbers = true,
        help = "Threshold for returned number of nodes. -1 disables the parameter."
    )]
    pub threshold: i32,
    #[arg(
        short = 'r',
        long,
        value_name = "rank",
        default_value = "species",
        value_parser = REPORT_RANKS,
        help = "The rank of the results to return."
    )]
    pub rank: String,
    #[arg(short = 'n', long, help = NO_DESCENDENTS_HELP)]
    pub no_descendents: bool,
    #[arg(short = 'u', long, help = "Print report URL.")]
    pub url: bool,
    #[arg(long, help = PROGRESS_BAR_HELP)]
    pub progress_bar: bool,
}

// `goat-cli taxon hist`
#[derive(Args, Debug, Clone)]
pub struct HistArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present = "file",
        help = "The taxon to return a histogram of. Multiple taxa will return the joint histogram."
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present = "taxon",
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short = 'x',
        long,
        value_name = "x-variable",
        help = "The variable to bin on the x-axis."
    )]
    pub x_variable: String,
    #[arg(
        short = 'r',
        long,
        value_name = "rank",
        default_value = "species",
        value_parser = REPORT_RANKS,
        help = "The taxonomic rank to aggregate results at."
    )]
    pub rank: String,
    #[arg(
        short = 'c',
        long,
        value_name = "category",
        help = "A variable or rank to use as a colour category (e.g. 'sex', 'genus')."
    )]
    pub category: Option<String>,
    #[arg(
        short = 's',
        long,
        value_name = "size",
        help = "The number of category levels to show."
    )]
    pub size: Option<usize>,
    #[arg(short = 'n', long, help = NO_DESCENDENTS_HELP)]
    pub no_descendents: bool,
    #[arg(
        short = 'o',
        long,
        value_name = "x-opts",
        help = format!("{}\nE.g. ',,20' = 20 bins; '1,10,5' = range 1–10 with 5 bins.", OPTS_HELP)
    )]
    pub x_opts: Option<String>,
    #[arg(short = 'u', long, help = "Print report URL.")]
    pub url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = REPORT_FORMAT_HELP
    )]
    pub format: Format,
}

// `goat-cli taxon scatter`
#[derive(Args, Debug, Clone)]
pub struct ScatterArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        required_unless_present = "file",
        help = "The taxon to return a scatter of. Multiple taxa will return the joint scatter."
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'f',
        long,
        value_name = "file",
        required_unless_present = "taxon",
        help = file_help()
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short = 'x',
        long,
        value_name = "x-variable",
        help = "The variable on the x-axis."
    )]
    pub x_variable: String,
    #[arg(
        short = 'y',
        long,
        value_name = "y-variable",
        help = "The variable on the y-axis."
    )]
    pub y_variable: String,
    #[arg(
        short = 'r',
        long,
        value_name = "rank",
        default_value = "species",
        value_parser = REPORT_RANKS,
        help = "The taxonomic rank to aggregate results at."
    )]
    pub rank: String,
    #[arg(
        short = 'c',
        long,
        value_name = "category",
        help = "A variable or rank to use as a colour category."
    )]
    pub category: Option<String>,
    #[arg(short = 'n', long, help = NO_DESCENDENTS_HELP)]
    pub no_descendents: bool,
    #[arg(long, value_name = "x-opts", help = OPTS_HELP)]
    pub x_opts: Option<String>,
    #[arg(
        long,
        value_name = "y-opts",
        help = "Options for the y-axis. Same format as --x-opts."
    )]
    pub y_opts: Option<String>,
    #[arg(short = 'u', long, help = "Print report URL.")]
    pub url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = REPORT_FORMAT_HELP
    )]
    pub format: Format,
}

// `goat-cli taxon arc`
#[derive(Args, Debug, Clone)]
pub struct ArcArgs {
    #[arg(
        short = 't',
        long,
        value_name = "taxon",
        help = "The taxon to scope the arc report to. Omit for a global query."
    )]
    pub taxon: Option<String>,
    #[arg(
        short = 'x',
        long,
        value_name = "x-filter",
        help = "Filter expression for the numerator.\nE.g. 'assembly_level >= scaffold', 'assembly_span', 'genome_size > 1e9'."
    )]
    pub x_filter: String,
    #[arg(
        short = 'y',
        long,
        value_name = "y-filter",
        help = "Filter expression for the denominator (reference population).\nDefaults to all taxa at the given rank."
    )]
    pub y_filter: Option<String>,
    #[arg(
        short = 'r',
        long,
        value_name = "rank",
        default_value = "species",
        value_parser = ARC_RANKS,
        help = "The taxonomic rank to aggregate results at."
    )]
    pub rank: String,
    #[arg(
        long,
        value_name = "exclude-missing",
        help = "Comma-separated fields to exclude if missing (e.g. 'assembly_span').\nReduces denominator to only taxa with a direct value for this field."
    )]
    pub exclude_missing: Option<String>,
    #[arg(
        long,
        value_name = "exclude-ancestral",
        help = "Comma-separated fields to exclude if ancestrally inferred (e.g. 'assembly_span')."
    )]
    pub exclude_ancestral: Option<String>,
    #[arg(
        short = 'n',
        long,
        help = "Do not include descendents (i.e. a tax_name() call)."
    )]
    pub no_descendents: bool,
    #[arg(short = 'u', long, help = "Print report URL.")]
    pub url: bool,
    #[arg(
        short = 'F',
        long,
        value_enum,
        value_name = "format",
        default_value = "tsv",
        help = REPORT_FORMAT_HELP
    )]
    pub format: Format,
}

impl From<&SourcesArgs> for ReportOptions {
    fn from(args: &SourcesArgs) -> Self {
        Self {
            taxon: Some(args.taxon.clone()),
            rank: args.rank.clone(),
            no_descendents: args.no_descendents,
            url: args.url,
            format: args.format,
            ..Default::default()
        }
    }
}

impl From<&NewickArgs> for ReportOptions {
    fn from(args: &NewickArgs) -> Self {
        Self {
            taxon: args.taxon.clone(),
            file: args.file.clone(),
            rank: args.rank.clone(),
            threshold: args.threshold,
            no_descendents: args.no_descendents,
            url: args.url,
            ..Default::default()
        }
    }
}

impl From<&HistArgs> for ReportOptions {
    fn from(args: &HistArgs) -> Self {
        Self {
            taxon: args.taxon.clone(),
            file: args.file.clone(),
            rank: args.rank.clone(),
            x_variable: Some(args.x_variable.clone()),
            category: args.category.clone(),
            size: args.size,
            no_descendents: args.no_descendents,
            x_opts: args.x_opts.clone(),
            url: args.url,
            format: args.format,
            ..Default::default()
        }
    }
}

impl From<&ScatterArgs> for ReportOptions {
    fn from(args: &ScatterArgs) -> Self {
        Self {
            taxon: args.taxon.clone(),
            file: args.file.clone(),
            rank: args.rank.clone(),
            x_variable: Some(args.x_variable.clone()),
            y_variable: Some(args.y_variable.clone()),
            category: args.category.clone(),
            no_descendents: args.no_descendents,
            x_opts: args.x_opts.clone(),
            y_opts: args.y_opts.clone(),
            url: args.url,
            format: args.format,
            ..Default::default()
        }
    }
}

impl From<&ArcArgs> for ReportOptions {
    fn from(args: &ArcArgs) -> Self {
        Self {
            taxon: args.taxon.clone(),
            rank: args.rank.clone(),
            x_filter: Some(args.x_filter.clone()),
            y_filter: args.y_filter.clone(),
            exclude_missing: args.exclude_missing.clone(),
            exclude_ancestral: args.exclude_ancestral.clone(),
            no_descendents: args.no_descendents,
            url: args.url,
            format: args.format,
            ..Default::default()
        }
    }
}
