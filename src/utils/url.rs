use lazy_static::lazy_static;

const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
const GOAT_API_VERSION: &str = "v0.0.1/";

lazy_static! {
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    pub static ref TAXONOMY: String = "ncbi".to_string();
}

// make the GoaT API URLs
// function here to make the ranks URL string
// &ranks=subspecies%2Cspecies%2Cgenus%2Cfamily%2Corder%2Cclass%2Cphylum%2Ckingdom%2Csuperkingdom

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

// default fields are:
// &fields=assembly_level
// %2Cassembly_span
// %2CBUSCO%20completeness
// %2Cchromosome_number
// %2Chaploid_number
// %2Cc_value
// %2Cgenome_size

// To add:
// %2Cmitochondrion_assembly_span
// %2Cmitochondrion_gc_percent

// a struct to gather the options

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
}

impl FieldBuilder {
    // private fn used below in build_fields_string
    fn as_array(&self) -> [bool; 8] {
        [
            self.all,
            self.assembly,
            self.busco,
            self.cvalues,
            self.gs,
            self.karyotype,
            self.mitochondrion,
            self.plastid,
        ]
    }

    //
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

        let field_array = self.as_array();
        let mut field_string = String::new();

        field_string += base;
        if self.assembly || self.all {
            field_string += assembly_level_field;
            field_string += delimiter;
        }
        if self.assembly || self.all {
            field_string += assembly_span_field;
            field_string += delimiter;
        }
        if self.busco || self.all {
            field_string += busco_completeness_field;
            field_string += delimiter;
        }
        if self.karyotype || self.all {
            field_string += chromosome_number_field;
            field_string += delimiter;
            field_string += haploid_number_field;
            field_string += delimiter;
        }
        if self.gs || self.all {
            field_string += genome_size_field;
            field_string += delimiter;
        }
        if self.cvalues || self.all {
            field_string += c_value_field;
            field_string += delimiter;
        }
        if self.mitochondrion || self.all {
            field_string += mitochondrial_assembly_span_field;
            field_string += delimiter;
            field_string += mitochondrial_gc_percent_field;
            field_string += delimiter;
        }
        // add plastid
        if self.plastid || self.all {
            field_string += plastid_assembly_span_field;
            field_string += delimiter;
            field_string += plastid_gc_percent_field;
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

pub fn make_goat_search_urls(
    taxids: Vec<String>,
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

    for el in taxids {
        let url = format!(
        "{}search?query=tax_{}%28{}%29&includeEstimates={}&includeRawValues={}&summaryValues={}&result={}&taxonomy={}&size={}{}{}",
        goat_url, tax_tree, el, include_estimates, include_raw_values, summarise_values_by, result, taxonomy, size, rank_string, fields_string
    );
        res.push(url);
    }
    res
}
