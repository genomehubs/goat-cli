use crate::utils::expression::CLIexpression;
use crate::utils::variables::Variables;

use anyhow::Result;
use lazy_static::lazy_static;

/// The base URL for GoaT.
const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
/// The current version of the GoaT API.
const GOAT_API_VERSION: &str = "v0.0.1/";

lazy_static! {
    /// The current GoaT URL.
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    /// The taxonomy that `goat-cli` uses.
    pub static ref TAXONOMY: String = "ncbi".to_string();
}

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
/// passed from the user.
#[derive(Copy, Clone)]
pub struct FieldBuilder {
    /// Add all of the current GoaT fields.
    pub all: bool,
    /// Add only assembly level/span GoaT fields.
    pub assembly: bool,
    /// Add bioproject GoaT field.
    pub bioproject: bool,
    /// Add BUSCO completeness.
    pub busco: bool,
    /// Add country list GoaT field.
    pub country_list: bool,
    /// Add C-value information GoaT field.
    pub cvalues: bool,
    /// Add assembly & EBP metric date GoaT fields.
    pub date: bool,
    /// Add GC percent GoaT field.
    pub gc_percent: bool,
    /// Add gene count GoaT field.
    pub gene_count: bool,
    /// Add genome size GoaT fields.
    pub gs: bool,
    /// Add karyotype GoaT fields; chromosome number and
    /// haploid number.
    pub karyotype: bool,
    /// Add return information for `isb_wildlife_act_1976`,
    /// `habreg_2017`, `marhabreg-2017`, `waca_1981`,
    /// `protection_of_badgers_act_1992`, `echabs92`
    pub legislation: bool,
    /// Add mitochondrial assembly span and gc percent
    /// GoaT fields.
    pub mitochondrion: bool,
    /// Add contig and scaffold n50 GoaT fields.
    pub n50: bool,
    /// Add synonym, tolID, and common name GoaT fields.
    pub names: bool,
    /// Add plastid assembly span and gc percent GoaT
    /// fields.
    pub plastid: bool,
    /// Add ploidy GoaT field.
    pub ploidy: bool,
    /// Add sex determination GoaT field.
    pub sex_determination: bool,
    /// Add sample tracking information GoaT field.
    pub status: bool,
    /// Add `long_list`, `other_priority`, and `family_representative`
    /// GoaT fields.
    pub target_lists: bool,
    /// Render output in tidy format?
    pub tidy: bool,
}

impl FieldBuilder {
    // Used for checking absence of any fields.
    fn as_array(&self) -> [bool; 21] {
        [
            self.all,
            self.assembly,
            self.bioproject,
            self.busco,
            self.country_list,
            self.cvalues,
            self.date,
            self.gc_percent,
            self.gene_count,
            self.gs,
            self.karyotype,
            self.legislation,
            self.mitochondrion,
            self.names,
            self.n50,
            self.plastid,
            self.ploidy,
            self.sex_determination,
            self.status,
            self.target_lists,
            self.tidy,
        ]
    }

    /// A function which formats all of the GoaT fields
    /// together into a URL segment.
    pub fn build_fields_string(&self) -> String {
        let base = "&fields=";
        let delimiter = "%2C";
        // these are display level 1
        let assembly_level_field = "assembly_level";
        let assembly_span_field = "assembly_span";
        let busco_completeness_field = "busco_completeness";
        let chromosome_number_field = "chromosome_number";
        let haploid_number_field = "haploid_number";
        let gc_percent_field = "gc_percent";
        // we also have genome size kmer & draft
        let genome_size_field = "genome_size";
        // c value method & cell type
        let c_value_field = "c_value";
        // these are display level 2
        let mitochondrial_assembly_span_field = "mitochondrion_assembly_span";
        let mitochondrial_gc_percent_field = "mitochondrion_gc_percent";
        let plastid_assembly_span_field = "plastid_assembly_span";
        let plastid_gc_percent_field = "plastid_gc_percent";
        let ploidy = "ploidy";
        let sex_determination = "sex_determination";
        // all legislation data
        let isb_wildlife_act_1976 = "isb_wildlife_act_1976";
        let hab_reg_2017 = "HabReg_2017";
        let mar_hab_reg_2017 = "MarHabReg-2017";
        let waca_1981 = "waca_1981";
        let protection_of_badgers_act_1992 = "Protection_of_Badgers_Act_1992";
        let e_c_habs92 = "ECHabs92";
        // all target lists
        let long_list = "long_list";
        let other_priority = "other_priority";
        let family_representative = "family_representative";
        // add n50
        let contig_n50 = "contig_n50";
        let scaffold_n50 = "scaffold_n50";
        // add bioproject & biosample
        let bioproject = "bioproject";
        let biosample = "biosample";
        // gene count
        let gene_count = "gene_count";
        // dates
        let assembly_date = "assembly_date";
        let ebp_metric_date = "ebp_metric_date";
        // country list
        let country_list = "country_list";
        // sequencing status of the taxon
        // lump all these together at the moment.
        let sequencing_status = "sequencing_status";
        let sample_collected = "sample_collected";
        let sample_acquired = "sample_acquired";
        let in_progress = "in_progress";
        let insdc_submitted = "insdc_submitted";
        let insdc_open = "insdc_open";
        let published = "published";
        let sample_collected_by = "sample_collected_by";

        let field_array = self.as_array();
        let mut field_string = String::new();

        field_string += base;
        // default assembly stats
        if self.assembly || self.all {
            field_string += assembly_level_field;
            field_string += delimiter;
            field_string += assembly_span_field;
            field_string += delimiter;
        }
        // busco stats
        if self.busco || self.all {
            field_string += busco_completeness_field;
            field_string += delimiter;
        }
        // gc_percent
        if self.gc_percent || self.all {
            field_string += gc_percent_field;
            field_string += delimiter;
        }
        // default karyotype stats
        if self.karyotype || self.all {
            field_string += chromosome_number_field;
            field_string += delimiter;
            field_string += haploid_number_field;
            field_string += delimiter;
        }
        // additional karyotype
        if self.ploidy || self.all {
            field_string += ploidy;
            field_string += delimiter;
        }
        // additional karyotype
        if self.sex_determination || self.all {
            field_string += sex_determination;
            field_string += delimiter;
        }
        // genome size and c-value split here
        // even though they are both default in GoaT.
        if self.gs || self.all {
            field_string += genome_size_field;
            field_string += delimiter;
        }
        if self.cvalues || self.all {
            field_string += c_value_field;
            field_string += delimiter;
        }
        // add mito data
        if self.mitochondrion || self.all {
            field_string += mitochondrial_assembly_span_field;
            field_string += delimiter;
            field_string += mitochondrial_gc_percent_field;
            field_string += delimiter;
        }
        // add plastid data
        if self.plastid || self.all {
            field_string += plastid_assembly_span_field;
            field_string += delimiter;
            field_string += plastid_gc_percent_field;
            field_string += delimiter;
        }
        // add all legislation data
        if self.legislation || self.all {
            field_string += isb_wildlife_act_1976;
            field_string += delimiter;
            field_string += hab_reg_2017;
            field_string += delimiter;
            field_string += mar_hab_reg_2017;
            field_string += delimiter;
            field_string += waca_1981;
            field_string += delimiter;
            field_string += protection_of_badgers_act_1992;
            field_string += delimiter;
            field_string += e_c_habs92;
            field_string += delimiter;
        }
        if self.target_lists || self.all {
            field_string += long_list;
            field_string += delimiter;
            field_string += other_priority;
            field_string += delimiter;
            field_string += family_representative;
            field_string += delimiter;
        }
        if self.n50 || self.all {
            field_string += contig_n50;
            field_string += delimiter;
            field_string += scaffold_n50;
            field_string += delimiter;
        }
        if self.bioproject || self.all {
            field_string += bioproject;
            field_string += delimiter;
            field_string += biosample;
            field_string += delimiter;
        }
        if self.gene_count || self.all {
            field_string += gene_count;
            field_string += delimiter;
        }
        if self.date || self.all {
            field_string += assembly_date;
            field_string += delimiter;
            field_string += ebp_metric_date;
            field_string += delimiter;
        }
        if self.country_list || self.all {
            field_string += country_list;
            field_string += delimiter;
        }
        if self.status || self.all {
            field_string += sequencing_status;
            field_string += delimiter;
            field_string += sample_collected;
            field_string += delimiter;
            field_string += sample_acquired;
            field_string += delimiter;
            field_string += in_progress;
            field_string += delimiter;
            field_string += insdc_submitted;
            field_string += delimiter;
            field_string += insdc_open;
            field_string += delimiter;
            field_string += published;
            field_string += delimiter;
            field_string += sample_collected_by;
            field_string += delimiter;
        }

        // remove the last three chars == '&2C'
        field_string.drain(field_string.len() - 3..);
        // check for blanks
        let any_true = field_array.iter().any(|&e| e == true);
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
    let names = format_names(fields.names);

    let tidy_data: &str;

    match fields.tidy {
        true => tidy_data = "&tidyData=true",
        false => tidy_data = "",
    }

    // enumeration of the taxa will be 0 -> n,
    // corresponding to alphabetical order of taxa
    for (index, el) in taxids.iter().enumerate() {
        let query_id = format!("&queryId=goat_cli_{}", index);
        let url = format!(
        // hardcode tidy data for now.
        "{goat_url}{api}?query=tax_{tax_tree}%28{taxon}%29{tax_rank}{expression}&includeEstimates={include_estimates}&includeRawValues={include_raw_values}&summaryValues={summarise_values_by}&result={result}&taxonomy={taxonomy}&size={size}{rank_string}{fields_string}{tidy_data}{names}{query_id}",
        goat_url = goat_url, api = api, tax_tree = tax_tree, taxon = el, tax_rank = tax_rank, expression = expression, include_estimates = include_estimates, include_raw_values = include_raw_values, summarise_values_by = summarise_values_by, result = result, taxonomy = taxonomy, size = size, rank_string = rank_string, fields_string = fields_string, tidy_data = tidy_data, names = names, query_id = query_id
    );
        res.push(url);
    }
    Ok(res)
}
