use goat_cli::utils::url::FieldBuilder;

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

// ── all-false baseline ────────────────────────────────────────────────────────

#[test]
fn test_all_false_builds_empty_string() {
    assert!(empty_fields().build_fields_string().is_empty());
}

// ── individual field groups ───────────────────────────────────────────────────

#[test]
fn test_genome_size_group_produces_three_fields() {
    let mut f = empty_fields();
    f.taxon_gs = true;
    let s = f.build_fields_string();
    assert!(s.starts_with("&fields="), "should start with &fields=");
    assert!(s.contains("genome_size"), "missing genome_size");
    assert!(s.contains("genome_size_kmer"), "missing genome_size_kmer");
    assert!(s.contains("genome_size_draft"), "missing genome_size_draft");
    assert!(!s.ends_with("%2C"), "trailing delimiter present");
}

#[test]
fn test_karyotype_group_produces_two_fields() {
    let mut f = empty_fields();
    f.taxon_karyotype = true;
    let s = f.build_fields_string();
    assert!(s.contains("chromosome_number"));
    assert!(s.contains("haploid_number"));
    assert!(!s.ends_with("%2C"));
}

#[test]
fn test_assembly_group_produces_level_and_span() {
    let mut f = empty_fields();
    f.taxon_assembly = true;
    let s = f.build_fields_string();
    assert!(s.contains("assembly_level"));
    assert!(s.contains("assembly_span"));
}

#[test]
fn test_assembly_index_assembly_group() {
    let mut f = empty_fields();
    f.assembly_assembly = true;
    let s = f.build_fields_string();
    assert!(s.starts_with("&fields="));
    assert!(s.contains("assembly_level"));
    assert!(s.contains("assembly_span"));
}

#[test]
fn test_assembly_contig_group() {
    let mut f = empty_fields();
    f.assembly_contig = true;
    let s = f.build_fields_string();
    assert!(s.contains("contig_count"));
    assert!(s.contains("contig_l50"));
    assert!(s.contains("contig_n50"));
}

#[test]
fn test_bioproject_group() {
    let mut f = empty_fields();
    f.taxon_bioproject = true;
    let s = f.build_fields_string();
    assert!(s.contains("bioproject"));
    assert!(s.contains("biosample"));
}

// ── combining groups ─────────────────────────────────────────────────────────

#[test]
fn test_two_groups_both_appear() {
    let mut f = empty_fields();
    f.taxon_gs = true;
    f.taxon_karyotype = true;
    let s = f.build_fields_string();
    assert!(s.contains("genome_size"));
    assert!(s.contains("chromosome_number"));
    assert!(!s.ends_with("%2C"));
}

#[test]
fn test_three_groups_no_trailing_delimiter() {
    let mut f = empty_fields();
    f.taxon_gs = true;
    f.taxon_karyotype = true;
    f.taxon_n50 = true;
    let s = f.build_fields_string();
    assert!(s.contains("genome_size"));
    assert!(s.contains("chromosome_number"));
    assert!(s.contains("scaffold_n50"));
    assert!(!s.ends_with("%2C"));
}

// ── toggle_direct ─────────────────────────────────────────────────────────────

#[test]
fn test_toggle_direct_appends_direct_ancestor_descendant() {
    let mut f = empty_fields();
    f.taxon_gs = true;
    f.taxon_toggle_direct = true;
    let s = f.build_fields_string();
    // Each field gets :direct, :ancestor, :descendant appended (URL-encoded colon = %3A)
    assert!(s.contains("genome_size%3Adirect"));
    assert!(s.contains("genome_size%3Aancestor"));
    assert!(s.contains("genome_size%3Adescendant"));
    assert!(!s.ends_with("%2C"));
}

#[test]
fn test_toggle_direct_false_no_extra_columns() {
    let mut f = empty_fields();
    f.taxon_gs = true;
    f.taxon_toggle_direct = false;
    let s = f.build_fields_string();
    assert!(!s.contains("%3Adirect"));
    assert!(!s.contains("%3Aancestor"));
    assert!(!s.contains("%3Adescendant"));
}
