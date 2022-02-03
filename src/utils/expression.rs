// parse expressions on the command line
// example expressions will be:
// plastid_gc_percent < 33 AND gc_percent > 55

use crate::error::error::ExpressionParseError;
use crate::utils::utils::switch_string_to_url_encoding;
use anyhow::{bail, ensure, Result};
use lazy_static::lazy_static;
use regex::{CaptureMatches, Captures, Regex};
use std::collections::BTreeMap;
use std::fmt;
use tabled::{Header, MaxWidth, Modify, Row, Table, Tabled};

// https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}

#[derive(Tabled)]
pub enum TypeOf<'a> {
    Long,
    Short,
    OneDP,
    TwoDP,
    Integer,
    Date,
    HalfFloat,
    Keyword(Vec<&'a str>),
}

impl<'a> fmt::Display for TypeOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            TypeOf::Long => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::Short => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::OneDP => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::TwoDP => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::Integer => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::Date => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::HalfFloat => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::Keyword(k) => write!(f, "== {}", k.join(", ")),
        }
    }
}

// kind of an option alias here.
pub enum Function<'a> {
    None,
    Some(Vec<&'a str>),
}

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Function::None => write!(f, ""),
            Function::Some(fun) => write!(f, "{}", fun.join(", ")),
        }
    }
}

#[derive(Tabled)]
pub struct Variable<'a> {
    #[header("Display Name")]
    pub display_name: &'a str,
    #[header("Operators/Keywords")]
    pub type_of: TypeOf<'a>,
    #[header("Function(s)")]
    pub functions: Function<'a>,
}

lazy_static! {
    pub static ref GOAT_VARIABLE_DATA: BTreeMap<&'static str, Variable<'static>> = collection!(
        "mitochondrion_assembly_span" => Variable { display_name: "mitochondrion span", type_of: TypeOf::Long, functions: Function::None },
        "mitochondrion_gc_percent" => Variable { display_name: "mitochondrion GC%", type_of: TypeOf::TwoDP, functions: Function::None },
        "plastid_assembly_span" => Variable { display_name: "plastid span", type_of: TypeOf::Long, functions: Function::None },
        "plastid_gc_percent" => Variable { display_name: "plastid GC%", type_of: TypeOf::TwoDP, functions: Function::None },
        "isb_wildlife_act_1976" => Variable { display_name: "Irish Statute Book Wildlife Act, 1976", type_of: TypeOf::Keyword(vec!["iwa-nsch3", "iwa-sch5"]), functions: Function::None },
        "marhabreg-2017" => Variable { display_name: "Conservation of Offshore Marine Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["marhabreg-sch1"]), functions: Function::None },
        "habreg_2017" => Variable { display_name: "Conservation of Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["habreg-sch2", "habreg-sch5"]), functions: Function::None },
        "waca_1981" => Variable { display_name: "Wildlife and Countryside Act 1981", type_of: TypeOf::Keyword(vec!["waca-sch1", "waca-sch5"]), functions: Function::None },
        "protection_of_badgers_act_1992" => Variable { display_name: "Protection of Badgers Act 1992", type_of: TypeOf::Keyword(vec!["badgers92"]), functions: Function::None },
        "country_list" => Variable { display_name: "Country list", type_of: TypeOf::Keyword(vec!["gb"]), functions: Function::None },
        "echabs92" => Variable { display_name: "EC Habitats Directive 1992", type_of: TypeOf::Keyword(vec!["echabs92_annex_iib", "echabs92_annex_ivb", "echabs92_annex_iva"]), functions: Function::None },
        "assembly_level" => Variable { display_name: "Assembly level", type_of: TypeOf::Keyword(vec!["complete genome", "chromosome", "scaffold", "contig"]), functions: Function::None },
        "assembly_span" => Variable { display_name: "Assembly span", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "contig_n50" => Variable { display_name: "Contig N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "assembly_date" => Variable { display_name: "Last updated", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
        "scaffold_n50" => Variable { display_name: "Scaffold N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "gene_count" => Variable { display_name: "Gene count", type_of: TypeOf::Integer, functions: Function::Some(vec!["min", "max"]) },
        "ebp_metric_date" => Variable { display_name: "EBP metric date", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
        "chromosome_number" => Variable { display_name: "Chromosome number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "haploid_number" => Variable { display_name: "Haploid number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "genome_size" => Variable { display_name: "Genome size", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "c_value" => Variable { display_name: "C value", type_of: TypeOf::HalfFloat, functions: Function::Some(vec!["min", "max"]) },
        "genome_size_kmer" => Variable { display_name: "Genome size kmer", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "genome_size_draft" => Variable { display_name: "Genome size draft", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "ploidy" => Variable { display_name: "Ploidy", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "c_value_method" => Variable { display_name: "C value method", type_of: TypeOf::Keyword(vec!["biochemical analysis", "bulk fluorometric assay", "complete genome sequencing", "feulgen densitometry", "feulgen image analysis densitometry", "flow cytometry", "flow karyotyping", "fluorescence fading analysis", "gallocyanin chrom alum densitometry", "methyl green densitometry", "not specified", "static cell fluorometry", "ultraviolet microscopy", "unknown"]), functions: Function::None },
        "c_value_cell_type" => Variable { display_name: "C value cell type", type_of: TypeOf::Keyword(vec!["antennae", "antennal gland", "blood cells", "brain", "buccal epithelium", "coelomocytes", "corneal epithelium", "digestive gland", "dorsal fin clip", "egg", "embyro", "epidermis", "exopodite", "fibroblasts", "fin clips", "germarium", "gills", "haemocytes", "heart cells", "individual chromosomes", "intestine", "kidney cells", "legs", "leukocytes", "liver", "lung (culture)", "mantle", "midgut", "muscle cells", "ne", "not specified", "oocytes", "ovaries", "pancreas", "pharynx", "polypide cells in suspension", "red blood cells", "retinal cells", "salivary gland", "somatic cells", "sperm", "spleen", "tentacles", "testes", "thymus", "tissue culture", "various", "ventral hypodermal chord", "whole body", "whole body squash"]), functions: Function::None },
        "busco_completeness" => Variable { display_name: "BUSCO completeness", type_of: TypeOf::OneDP, functions: Function::None },
        "gc_percent" => Variable { display_name: "GC percent", type_of: TypeOf::OneDP, functions: Function::None },
        "nohit" => Variable { display_name: "No hit", type_of: TypeOf::OneDP, functions: Function::None },
        "target" => Variable { display_name: "Target", type_of: TypeOf::OneDP, functions: Function::None },
    );
}

#[derive(Tabled)]
struct ColHeader(#[header("Expression Name")] &'static str);

pub fn print_variable_data() {
    // for some space
    println!("");
    // map the header to a tuple combination
    // see https://github.com/zhiburt/tabled/blob/master/README.md
    let table_data = &*GOAT_VARIABLE_DATA
        .iter()
        .map(|(e, f)| (ColHeader(e), f))
        .collect::<Vec<(ColHeader, &Variable)>>();

    let table_string = Table::new(table_data)
        .with(Header(
            "Variable names in GoaT, with functional operator annotation.",
        ))
        // wrap the text!
        .with(Modify::new(Row(1..)).with(MaxWidth::wrapping(40)))
        .to_string();

    println!("{}", table_string);
}

pub struct CLIexpression<'a> {
    pub inner: &'a str,
    pub length: usize, // these queries can't be crazy long.
    pub expression: Vec<&'a str>,
}

impl<'a> CLIexpression<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            inner: string,
            length: string.len(),
            expression: Vec::new(),
        }
    }

    fn split(&self) -> Self {
        let mut res_vec = Vec::new();
        // commands only accept AND? Rich!
        let re = Regex::new("AND").unwrap();
        let splitter = SplitCaptures::new(&re, &self.inner);
        for state in splitter {
            let el = match state {
                SplitState::Unmatched(s) => s,
                SplitState::Captured(s) => s.get(0).map_or("", |m| m.as_str()),
            };
            res_vec.push(el);
        }
        Self {
            inner: &self.inner,
            length: self.length,
            expression: res_vec,
        }
    }

    pub fn parse(&mut self) -> Result<String> {
        if self.length > 100 {
            bail!(ExpressionParseError::QueryTooLong)
        }
        if self.inner.contains("&&") {
            bail!(ExpressionParseError::KeywordAndError)
        }
        if self.inner.contains(" contains") {
            bail!(ExpressionParseError::KeywordContainsError)
        }
        if self.inner.contains("||") || self.inner.contains("OR") {
            bail!(ExpressionParseError::KeywordOrError)
        }
        if self.inner.contains("tax_name") || self.inner.contains("tax_tree") {
            bail!(ExpressionParseError::KeywordTaxError)
        }
        let split_vec = &self.split();
        let exp_vec = &split_vec.expression;

        // split the expression vector into parts
        let mut index = 0;
        let exp_vec_len = exp_vec.len();
        let mut expression = String::new();
        // regular expression splitter
        // precedence here matters
        let re = Regex::new(r"!=|<=|<|==|=|>=|>").unwrap();
        if !re.is_match(self.inner) {
            bail!(ExpressionParseError::NoOperatorError)
        }

        // must always start with a space and AND
        expression += "%20AND";
        // vector of variables to check against
        let var_vec_check = &*GOAT_VARIABLE_DATA
            .iter()
            .map(|(e, _)| *e)
            .collect::<Vec<&str>>();
        // we can also create another vector of variables
        // with the appropriate max/min attached.
        // TODO: this seems like a crazy way of doing this - any better ideas?
        let var_vec_min_max_check = {
            let mut collector = Vec::new();
            for (goat_var, el) in &*GOAT_VARIABLE_DATA {
                match &el.functions {
                    Function::None => (),
                    Function::Some(f) => {
                        for pos in f {
                            let format_pos = format!("{}({})", pos, goat_var);
                            collector.push(format_pos);
                        }
                    }
                }
            }
            collector
        };

        // loop over the expression vector
        // splitting into further vectors
        // to evaluate each argument.
        loop {
            if index == exp_vec_len {
                break;
            }
            // expected to be in format
            // variable <operator> number/enum
            let curr_el = exp_vec[index];

            let mut curr_el_vec = Vec::new();
            // split this on the operator
            // do we need to check whether this operator actually exists?
            // I can imagine that this will break down otherwise...
            let splitter = SplitCaptures::new(&re, curr_el);

            for state in splitter {
                match state {
                    SplitState::Unmatched(s) => {
                        curr_el_vec.push(s);
                    }
                    SplitState::Captured(s) => {
                        curr_el_vec.push(s.get(0).map_or("", |m| m.as_str()));
                    }
                };
            }

            // check this vector is length 3 or 1
            ensure!(
                    curr_el_vec.len() == 3 || curr_el_vec.len() == 1,
                    "[-]\tSplit vector on single expression is invalid - length = {}. Are the input variables or operands correct?",
                    curr_el_vec.len()
                );
            match curr_el_vec.len() {
                3 => {
                    // trim strings
                    let variable = curr_el_vec[0].trim();
                    let operator = switch_string_to_url_encoding(curr_el_vec[1]).trim();
                    // let operator = curr_el_vec[1];
                    let value = curr_el_vec[2].trim();

                    if !var_vec_check.contains(&variable)
                        && !var_vec_min_max_check.contains(&variable.to_string())
                    {
                        // might be able to check max/min/length here.
                        // e.g. max(gc_content) > 0.3
                        bail!(ExpressionParseError::SplitVectorError)
                    }

                    // this panicks with min/max.
                    // if min/max present, extract within the parentheses.
                    let keyword_enums = match var_vec_min_max_check.contains(&variable.to_string())
                    {
                        true => {
                            // this means we have min/max
                            let re = Regex::new(r"\((.*?)\)").unwrap();
                            // we guarantee getting here with a variable, so unwrap is fine
                            // the second unwrap is always guaranteed too?
                            let extract_var =
                                re.captures(variable).unwrap().get(1).unwrap().as_str();
                            &GOAT_VARIABLE_DATA.get(extract_var).unwrap().type_of
                        }
                        false => &GOAT_VARIABLE_DATA.get(variable).unwrap().type_of,
                    };

                    // if there are parentheses - i.e. in min()/max() functions
                    let url_encoded_variable = variable.replace("(", "%28");
                    let url_encoded_variable = url_encoded_variable.replace(")", "%29");

                    // if there are keywords, make sure they are a match
                    match keyword_enums {
                        TypeOf::Keyword(k) => {
                            if !k.to_vec().contains(&value) {
                                // commas for or statements to be done here.
                                bail!(ExpressionParseError::KeywordEnumError)
                            }
                            // build expression
                            expression += "%20";
                            expression += &url_encoded_variable;
                            // do operators need to be translated?
                            expression += operator;
                            expression += value;
                            expression += "%20";
                            // end of sub expression
                            // assume there is another expression to follow
                            expression += "AND%20"
                        }
                        _ => {
                            // build expression
                            expression += "%20";
                            expression += &url_encoded_variable;
                            // do operators need to be translated?
                            expression += operator;
                            expression += value;
                            expression += "%20";
                            // end of sub expression
                            // assume there is another expression to follow
                            expression += "AND%20"
                        }
                    }
                }
                1 => (),
                _ => unreachable!(),
            }

            index += 1;
        }
        // remove trailing AND%20
        match expression.len() - 6 > 0 {
            true => {
                expression.drain(expression.len() - 6..);
                Ok(expression)
            }
            false => {
                bail!(ExpressionParseError::FormatExpressionError)
            }
        }
    }
}

// thanks https://github.com/rust-lang/regex/issues/330
// for splitting a string and keeping the delimiter.

#[derive(Debug)]
struct SplitCaptures<'r, 't> {
    finder: CaptureMatches<'r, 't>,
    text: &'t str,
    last: usize,
    caps: Option<Captures<'t>>,
}

impl<'r, 't> SplitCaptures<'r, 't> {
    pub fn new(re: &'r Regex, text: &'t str) -> SplitCaptures<'r, 't> {
        SplitCaptures {
            finder: re.captures_iter(text),
            text: text,
            last: 0,
            caps: None,
        }
    }
}

#[derive(Debug)]
enum SplitState<'t> {
    Unmatched(&'t str),
    Captured(Captures<'t>),
}

impl<'r, 't> Iterator for SplitCaptures<'r, 't> {
    type Item = SplitState<'t>;

    fn next(&mut self) -> Option<SplitState<'t>> {
        if let Some(caps) = self.caps.take() {
            return Some(SplitState::Captured(caps));
        }
        match self.finder.next() {
            None => {
                if self.last >= self.text.len() {
                    None
                } else {
                    let s = &self.text[self.last..];
                    self.last = self.text.len();
                    Some(SplitState::Unmatched(s))
                }
            }
            Some(caps) => {
                let m = caps.get(0).unwrap();
                let unmatched = &self.text[self.last..m.start()];
                self.last = m.end();
                self.caps = Some(caps);
                Some(SplitState::Unmatched(unmatched))
            }
        }
    }
}
