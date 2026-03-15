use goat_cli::utils::url::{format_expression, make_goat_urls, FieldBuilder};
use goat_cli::IndexType;

// ── helpers ──────────────────────────────────────────────────────────────────

fn empty_fields() -> FieldBuilder {
    FieldBuilder {
        taxon_assembly: false,
        taxon_bioproject: false,
        taxon_busco: false,
        taxon_country_list: false,
        taxon_cvalues: false,
        taxon_date: false,
        taxon_gc_percent: false,
        taxon_gene_count: false,
        taxon_gs: false,
        taxon_karyotype: false,
        taxon_legislation: false,
        taxon_mitochondrion: false,
        taxon_names: false,
        taxon_n50: false,
        taxon_plastid: false,
        taxon_ploidy: false,
        taxon_sex_determination: false,
        taxon_status: false,
        taxon_target_lists: false,
        taxon_tidy: false,
        taxon_toggle_direct: false,
        assembly_assembly: false,
        assembly_karyotype: false,
        assembly_contig: false,
        assembly_scaffold: false,
        assembly_gc: false,
        assembly_gene: false,
        assembly_busco: false,
        assembly_btk: false,
    }
}

fn make_taxon_urls(
    taxids: &[String],
    fields: FieldBuilder,
    expression: &str,
    unique_ids: Vec<String>,
) -> Vec<String> {
    make_goat_urls(
        "search",
        taxids,
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "none",
        fields,
        None,
        expression,
        "",
        unique_ids,
        IndexType::Taxon,
    )
    .expect("URL generation should not fail")
}

// ── format_expression ────────────────────────────────────────────────────────

#[test]
fn test_format_expression_taxon_encodes_operator() {
    let exp = format_expression("genome_size > 1000", IndexType::Taxon).expect("expression parsed");
    assert!(exp.contains("genome_size"));
    assert!(exp.contains("%3E"));
    assert!(exp.contains("1000"));
}

#[test]
fn test_format_expression_unknown_variable_returns_err() {
    let result = format_expression("not_a_real_var > 0", IndexType::Taxon);
    assert!(result.is_err());
}

#[test]
fn test_format_expression_rejects_tax_rank_in_expression() {
    let result = format_expression("tax_rank(species)", IndexType::Taxon);
    assert!(result.is_err());
}

// ── make_goat_urls: core shape ────────────────────────────────────────────────

#[test]
fn test_basic_taxon_search_url_generation() {
    let expression =
        format_expression("genome_size > 1000", IndexType::Taxon).expect("expression parsed");
    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        empty_fields(),
        &expression,
        vec![String::from("abc123")],
    );

    assert_eq!(urls.len(), 1);
    let url = &urls[0];
    assert!(url.contains("search?query=tax_name%28Mammalia%29"));
    assert!(url.contains("genome_size"));
    assert!(url.contains("%3E"));
    assert!(url.contains("1000"));
    assert!(url.contains("result=taxon"));
    assert!(url.contains("taxonomy=ncbi"));
    assert!(url.contains("size=50"));
    assert!(url.contains("queryId=goat_cli_abc123"));
}

#[test]
fn test_multiple_taxa_generate_one_url_each() {
    let urls = make_taxon_urls(
        &[
            String::from("Mammalia"),
            String::from("Reptilia"),
            String::from("Aves"),
        ],
        empty_fields(),
        "",
        vec![
            String::from("id1"),
            String::from("id2"),
            String::from("id3"),
        ],
    );

    assert_eq!(urls.len(), 3);
    assert!(urls[0].contains("Mammalia"));
    assert!(urls[1].contains("Reptilia"));
    assert!(urls[2].contains("Aves"));
}

#[test]
fn test_query_ids_appear_in_respective_urls() {
    let urls = make_taxon_urls(
        &[String::from("Mammalia"), String::from("Aves")],
        empty_fields(),
        "",
        vec![String::from("aaa"), String::from("bbb")],
    );

    assert!(urls[0].contains("queryId=goat_cli_aaa"));
    assert!(urls[1].contains("queryId=goat_cli_bbb"));
}

// ── make_goat_urls: parameter flags ──────────────────────────────────────────

#[test]
fn test_include_estimates_true_appears_in_url() {
    let urls = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        true, // include_estimates
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "none",
        empty_fields(),
        None,
        "",
        "",
        vec![String::from("id1")],
        IndexType::Taxon,
    )
    .expect("URL generation should not fail");

    assert!(urls[0].contains("includeEstimates=true"));
}

#[test]
fn test_include_estimates_false_appears_in_url() {
    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        empty_fields(),
        "",
        vec![String::from("id1")],
    );
    assert!(urls[0].contains("includeEstimates=false"));
}

#[test]
fn test_ranks_parameter_species_appears_in_url() {
    let urls = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "species", // ranks
        empty_fields(),
        None,
        "",
        "",
        vec![String::from("id1")],
        IndexType::Taxon,
    )
    .expect("URL generation should not fail");

    assert!(urls[0].contains("ranks=species"));
}

#[test]
fn test_ranks_none_does_not_add_ranks_segment() {
    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        empty_fields(),
        "",
        vec![String::from("id1")],
    );
    assert!(!urls[0].contains("ranks="));
}

#[test]
fn test_assembly_index_type_result_field() {
    let urls = make_goat_urls(
        "search",
        &[String::from("GCA_000001405")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,
        false,
        false,
        "count",
        "assembly", // result
        "ncbi",
        10,
        "none",
        empty_fields(),
        None,
        "",
        "",
        vec![String::from("id1")],
        IndexType::Assembly,
    )
    .expect("URL generation should not fail");

    assert!(urls[0].contains("result=assembly"));
}

#[test]
fn test_tax_tree_query_type() {
    let urls = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "tree", // tax_tree
        false,
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "none",
        empty_fields(),
        None,
        "",
        "",
        vec![String::from("id1")],
        IndexType::Taxon,
    )
    .expect("URL generation should not fail");

    assert!(urls[0].contains("tax_tree%28Mammalia%29"));
}

#[test]
fn test_taxon_name_query_type() {
    let urls = make_taxon_urls(
        &[String::from("Homo sapiens")],
        empty_fields(),
        "",
        vec![String::from("id1")],
    );
    assert!(urls[0].contains("tax_name%28Homo sapiens%29"));
}

// ── make_goat_urls: field builder flags ──────────────────────────────────────

#[test]
fn test_genome_size_fields_appear_when_flag_set() {
    let mut fields = empty_fields();
    fields.taxon_gs = true;

    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        fields,
        "",
        vec![String::from("id1")],
    );

    let url = &urls[0];
    assert!(url.contains("genome_size"));
    assert!(url.contains("&fields="));
}

#[test]
fn test_no_fields_segment_when_all_flags_false() {
    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        empty_fields(),
        "",
        vec![String::from("id1")],
    );
    // When all flags are false, build_fields_string() returns "" so no &fields= in URL
    assert!(!urls[0].contains("&fields="));
}

#[test]
fn test_karyotype_fields_appear_when_flag_set() {
    let mut fields = empty_fields();
    fields.taxon_karyotype = true;

    let urls = make_taxon_urls(
        &[String::from("Mammalia")],
        fields,
        "",
        vec![String::from("id1")],
    );

    let url = &urls[0];
    assert!(url.contains("chromosome_number"));
    assert!(url.contains("haploid_number"));
}

// ── make_goat_urls: variables parameter ──────────────────────────────────────

#[test]
fn test_variables_parameter_appears_in_fields() {
    let urls = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "none",
        empty_fields(),
        Some("genome_size"), // variables
        "",
        "",
        vec![String::from("id1")],
        IndexType::Taxon,
    )
    .expect("URL generation should not fail");

    assert!(urls[0].contains("genome_size"));
    assert!(urls[0].contains("&fields="));
}

#[test]
fn test_invalid_variable_returns_err() {
    let result = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,
        false,
        false,
        "count",
        "taxon",
        "ncbi",
        50,
        "none",
        empty_fields(),
        Some("not_a_real_variable_xyz"),
        "",
        "",
        vec![String::from("id1")],
        IndexType::Taxon,
    );
    assert!(result.is_err());
}
