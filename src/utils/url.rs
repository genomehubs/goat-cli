use crate::utils::expression::CLIexpression;
use crate::utils::variables::Variables;

use anyhow::Result;

// format the ranks for the URL.

/// Function to format the rank into a GoaT URL segment.
fn format_rank(r: &str) -> String {
    // fixed vector of ranks.
    // "none" by default will return an empty string here.
    let ranks = vec![
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
    let position_selected = ranks.iter().position(|e| e == &r);
    let updated_ranks = match position_selected {
        Some(p) => &ranks[p..],
        None => return "".to_string(),
    };
    let mut rank_string = String::new();
    rank_string += "&ranks=";
    let ranks_to_add = updated_ranks.join("%2C");
    rank_string += &ranks_to_add;

    rank_string
}

/// If names appears in [`FieldBuilder`], then we add the
/// GoaT URL segment for that.
fn format_names(flag: bool) -> String {
    match flag {
        true => "&names=synonym%2Ctol_id%2Ccommon_name".to_string(),
        false => "".to_string(),
    }
}

/// Format an expression put into the `-e` flag on the CLI.
pub fn format_expression(exp: &str) -> Result<String> {
    let mut new_exp = CLIexpression::new(exp);
    let parsed_string = new_exp.parse()?;
    Ok(parsed_string)
}

/// Boolean struct containing all of the CLI flag information
/// passed from the user. This struct has been expanded to include
/// both `taxon` and `assembly` indexes.
#[derive(Copy, Clone)]
pub struct FieldBuilder {
    /// Add only assembly level/span GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_assembly: bool,
    /// Add bioproject GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_bioproject: bool,
    /// Add BUSCO completeness.
    ///
    /// A taxon index flag.
    pub taxon_busco: bool,
    /// Add country list GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_country_list: bool,
    /// Add C-value information GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_cvalues: bool,
    /// Add assembly & EBP metric date GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_date: bool,
    /// Add GC percent GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_gc_percent: bool,
    /// Add gene count GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_gene_count: bool,
    /// Add genome size GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_gs: bool,
    /// Add karyotype GoaT fields; chromosome number and
    /// haploid number.
    ///
    /// A taxon index flag.
    pub taxon_karyotype: bool,
    /// Add return information for `isb_wildlife_act_1976`,
    /// `habreg_2017`, `marhabreg-2017`, `waca_1981`,
    /// `protection_of_badgers_act_1992`, `echabs92`
    ///
    /// A taxon index flag.
    pub taxon_legislation: bool,
    /// Add mitochondrial assembly span and gc percent
    /// GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_mitochondrion: bool,
    /// Add contig and scaffold n50 GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_n50: bool,
    /// Add synonym, tolID, and common name GoaT fields.
    ///
    /// Not implemented in [`FieldBuilder`] below.
    ///
    /// A taxon index flag.
    pub taxon_names: bool,
    /// Add plastid assembly span and gc percent GoaT
    /// fields.
    ///
    /// A taxon index flag.
    pub taxon_plastid: bool,
    /// Add ploidy GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_ploidy: bool,
    /// Add sex determination GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_sex_determination: bool,
    /// Add sample tracking information GoaT field.
    ///
    /// A taxon index flag.
    pub taxon_status: bool,
    /// Add `long_list`, `other_priority`, and `family_representative`
    /// GoaT fields.
    ///
    /// A taxon index flag.
    pub taxon_target_lists: bool,
    /// Render output in tidy format?
    ///
    /// Not implemented in [`FieldBuilder`] below.
    ///
    /// A taxon index flag.
    pub taxon_tidy: bool,
    /// Assembly span and level.
    ///
    /// An assembly index flag.
    pub assembly_assembly: bool,
    /// Only chromosome count.
    ///
    /// An assembly index flag.
    pub assembly_karyotype: bool,
    /// All the contig information.
    ///
    /// An assembly index flag.
    pub assembly_contig: bool,
    /// All scaffold information.
    ///
    /// An assembly index flag.
    pub assembly_scaffold: bool,
    /// GC content.
    ///
    /// An assembly index flag.
    pub assembly_gc: bool,
    /// Gene and non-coding gene count.
    ///
    /// An assembly index flag.
    pub assembly_gene: bool,
    /// BUSCO completeness, lineage and string.
    ///
    /// An assembly index flag.
    pub assembly_busco: bool,
    /// BlobToolKit stats(?). No hit/target.
    ///
    /// An assembly index flag.
    pub assembly_btk: bool,
}

impl FieldBuilder {
    /// A function to turn all of the fields into a small data structure.
    ///
    /// This is hardcoded, but could be modified to be read in from
    /// the goat standard variables JSON in the future.
    ///
    /// It's a [`Vec`] of a tuple of:
    /// - [`bool`] which shows whether the user chose this flag or not
    /// - [`Vec<&str>`] which enumerates the variable strings (as GoaT
    /// would recognise) that correspond to this field.
    ///
    /// It's a bit of a judgement call on my part but happy to change if
    /// there is a compelling argument.
    fn to_vec_tuples(&self) -> Vec<(bool, Vec<&str>)> {
        vec![
            // Add all of the taxon_* fields
            (self.taxon_assembly, vec!["assembly_level", "assembly_span"]),
            (self.taxon_bioproject, vec!["bioproject", "biosample"]),
            // testing all these busco fields.
            (
                self.taxon_busco,
                vec![
                    "busco_completeness",
                    "odb10_lineage",
                    "busco_lineage",
                    "busco_string",
                ],
            ),
            (self.taxon_country_list, vec!["country_list"]),
            (self.taxon_cvalues, vec!["c_value"]),
            (self.taxon_date, vec!["assembly_date", "ebp_metric_date"]),
            (self.taxon_gc_percent, vec!["gc_percent"]),
            (self.taxon_gene_count, vec!["gene_count"]),
            (
                self.taxon_gs,
                vec!["genome_size", "genome_size_kmer", "genome_size_draft"],
            ),
            (
                self.taxon_karyotype,
                vec!["chromosome_number", "haploid_number"],
            ),
            (
                self.taxon_legislation,
                vec![
                    "isb_wildlife_act_1976",
                    "HabReg_2017",
                    "MarHabReg-2017",
                    "waca_1981",
                    "Protection_of_Badgers_Act_1992",
                    "ECHabs92",
                ],
            ),
            (
                self.taxon_mitochondrion,
                vec!["mitochondrion_assembly_span", "mitochondrion_gc_percent"],
            ),
            (self.taxon_n50, vec!["scaffold_n50", "contig_n50"]),
            (
                self.taxon_plastid,
                vec!["plastid_assembly_span", "plastid_gc_percent"],
            ),
            (self.taxon_ploidy, vec!["ploidy"]),
            (self.taxon_sex_determination, vec!["sex_determination"]),
            // there's now a bunch of sequencing status_asg/b10k/cbp... etc
            // don't know if these should go here.
            (
                self.taxon_status,
                vec![
                    "sequencing_status",
                    "sample_collected",
                    "sample_acquired",
                    "in_progress",
                    "insdc_submitted",
                    "insdc_open",
                    "published",
                    "sample_collected_by",
                ],
            ),
            (
                self.taxon_target_lists,
                vec!["long_list", "other_priority", "family_representative"],
            ),
            // Add all of the assembly_* fields
            (
                self.assembly_assembly,
                vec!["assembly_level", "assembly_span"],
            ),
            (self.assembly_btk, vec!["nohit", "target"]),
            (
                self.assembly_busco,
                vec!["busco_completeness", "busco_lineage", "busco_string"],
            ),
            (
                self.assembly_contig,
                vec!["contig_count", "contig_l50", "contig_n50"],
            ),
            (self.assembly_gc, vec!["gc_percent"]),
            (
                self.assembly_gene,
                vec!["gene_count", "noncoding_gene_count"],
            ),
            (self.assembly_karyotype, vec!["chromosome_count"]),
            (
                self.assembly_scaffold,
                vec!["scaffold_count", "scaffold_l50", "scaffold_n50"],
            ),
        ]
    }

    /// A function which formats all of the GoaT fields
    /// together into a URL segment.
    pub fn build_fields_string(&self) -> String {
        const BASE: &str = "&fields=";
        const DELIMITER: &str = "%2C";

        // build the little data base
        let data = self.to_vec_tuples();

        // and now build the string
        let mut field_string = String::new();
        // add the base
        field_string += BASE;
        for (field_present, field_vec) in data.iter() {
            match field_present {
                true => {
                    field_string += &field_vec.join(DELIMITER);
                    field_string += DELIMITER;
                }
                false => continue,
            }
        }

        // remove the last three chars == '&2C'
        field_string.drain(field_string.len() - 3..);
        // check for blanks
        let any_true = data.iter().map(|e| e.0).any(|e| e == true);
        if !any_true {
            // remove everything
            field_string.drain(..);
        }

        field_string
    }
}

/// The function which creats the GoaT API URLs
/// which are then used as GET requests.
pub fn make_goat_urls(
    api: &str,
    taxids: &Vec<String>,
    goat_url: &str,
    tax_tree: &str,
    include_estimates: bool,
    include_raw_values: bool,
    summarise_values_by: &str,
    result: &str,
    taxonomy: &str,
    size: &str,
    ranks: &str,
    fields: FieldBuilder,
    variables: Option<&str>,
    expression: &str,
    tax_rank: &str,
    unique_ids: Vec<String>,
) -> Result<Vec<String>> {
    let mut res = Vec::new();

    // make the rank string
    let rank_string = format_rank(ranks);
    // make the fields string
    // either from hand coded variables by the user
    // or from flag switches
    let fields_string = match variables {
        Some(v) => Variables::new(v).parse()?,
        None => fields.build_fields_string(),
    };
    let names = format_names(fields.taxon_names);

    let tidy_data: &str;

    match fields.taxon_tidy {
        true => tidy_data = "&tidyData=true",
        false => tidy_data = "",
    }

    // enumeration of the taxa will be 0 -> n,
    // corresponding to alphabetical order of taxa
    for (el, chars) in taxids.iter().zip(unique_ids.iter()) {
        let query_id = format!("&queryId=goat_cli_{}", chars);
        let url = format!(
        // hardcode tidy data for now.
        "{goat_url}{api}?query=tax_{tax_tree}%28{taxon}%29{tax_rank}{expression}&includeEstimates={include_estimates}&includeRawValues={include_raw_values}&summaryValues={summarise_values_by}&result={result}&taxonomy={taxonomy}&size={size}{rank_string}{fields_string}{tidy_data}{names}{query_id}",
        goat_url = goat_url, api = api, tax_tree = tax_tree, taxon = el, tax_rank = tax_rank, expression = expression, include_estimates = include_estimates, include_raw_values = include_raw_values, summarise_values_by = summarise_values_by, result = result, taxonomy = taxonomy, size = size, rank_string = rank_string, fields_string = fields_string, tidy_data = tidy_data, names = names, query_id = query_id
    );
        res.push(url);
    }
    Ok(res)
}
