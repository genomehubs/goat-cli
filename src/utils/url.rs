use lazy_static::lazy_static;

const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
const GOAT_API_VERSION: &str = "v0.0.1/";

lazy_static! {
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    pub static ref TAXONOMY: String = "ncbi".to_string();
}

// format the ranks for the URL.

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

// for now, let's put all names in.
fn format_names(flag: bool) -> String {
    match flag {
        true => "&names=synonym%2Ctol_id%2Ccommon_name".to_string(),
        false => "".to_string(),
    }
}

// Parse the fields from search.
#[derive(Copy, Clone)]
pub struct FieldBuilder {
    pub all: bool,
    pub assembly: bool,
    pub busco: bool,
    pub cvalues: bool,
    pub gs: bool,
    pub karyotype: bool,
    pub mitochondrion: bool,
    pub plastid: bool,
    pub ploidy: bool,
    pub sex_determination: bool,
    pub legislation: bool,
    pub names: bool,
}

impl FieldBuilder {
    // private fn used below in build_fields_string
    fn as_array(&self) -> [bool; 12] {
        [
            self.all,
            self.assembly,
            self.busco,
            self.cvalues,
            self.gs,
            self.karyotype,
            self.mitochondrion,
            self.plastid,
            self.ploidy,
            self.sex_determination,
            self.legislation,
            self.names,
        ]
    }

    pub fn build_fields_string(&self) -> String {
        let base = "&fields=";
        let delimiter = "%2C";
        // these are display level 1
        let assembly_level_field = "assembly_level";
        let assembly_span_field = "assembly_span";
        let busco_completeness_field = "busco_completeness";
        let chromosome_number_field = "chromosome_number";
        let haploid_number_field = "haploid_number";
        let genome_size_field = "genome_size";
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
) -> Vec<String> {
    let mut res = Vec::new();

    // make the rank string
    let rank_string = format_rank(ranks);
    // make the fields string
    let fields_string = fields.build_fields_string();
    let names = format_names(fields.names);

    for el in taxids {
        let url = format!(
        // hardcode tidy data for now.
        "{goat_url}{api}?query=tax_{tax_tree}%28{taxon}%29&includeEstimates={include_estimates}&includeRawValues={include_raw_values}&summaryValues={summarise_values_by}&result={result}&taxonomy={taxonomy}&size={size}{rank_string}{fields_string}&tidyData=true{names}",
        goat_url = goat_url, api = api, tax_tree = tax_tree, taxon = el, include_estimates = include_estimates, include_raw_values = include_raw_values, summarise_values_by = summarise_values_by, result = result, taxonomy = taxonomy, size = size, rank_string = rank_string, fields_string = fields_string, names = names
    );
        res.push(url);
    }
    res
}
