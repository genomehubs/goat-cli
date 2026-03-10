use goat_cli::utils::url::{format_expression, make_goat_urls, FieldBuilder};
use goat_cli::IndexType;

#[test]
fn test_basic_taxon_search_url_generation() {
    let expression =
        format_expression("genome_size > 1000", IndexType::Taxon).expect("expression parsed");

    let fields = FieldBuilder {
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
    };

    let urls = make_goat_urls(
        "search",
        &[String::from("Mammalia")],
        "https://goat.genomehubs.org/api/v2/",
        "name",
        false,   // include_estimates
        false,   // include_raw_values
        false,   // exclude
        "count", // summarise_values_by
        "taxon", // result
        "ncbi",  // taxonomy
        50,      // size
        "none",  // ranks
        fields,
        None,        // variables
        &expression, // expression
        "",          // tax_rank
        vec![String::from("abc123")],
        IndexType::Taxon,
    )
    .expect("URL generation failed");

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
