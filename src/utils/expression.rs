// parse expressions on the command line
// example expressions will be:
// plastid_gc_percent < 33 AND gc_percent > 55
// TODO: ADD TAX_RANKS?

use crate::error::error::ExpressionParseError;
use crate::utils::tax_ranks::TaxRanks;
use crate::utils::utils::switch_string_to_url_encoding;
use crate::utils::variable_data::GOAT_VARIABLE_DATA;

use anyhow::{bail, ensure, Result};
use regex::{CaptureMatches, Captures, Regex};
use std::fmt;
use tabled::{Footer, Header, MaxWidth, Modify, Row, Table, Tabled};

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
            TypeOf::Keyword(k) => match k[0] {
                "" => write!(f, ""),
                _ => write!(f, "== {}", k.join(", ")),
            },
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
    // add taxon ranks at end...
    let footer_data = TaxRanks::init();

    let table_string = Table::new(table_data)
        .with(Header(
            "Variable names in GoaT, with functional operator annotation.",
        ))
        // wrap the text!
        .with(Footer(format!("NCBI taxon ranks:\n\n{}", footer_data)))
        .with(Modify::new(Row(1..table_data.len() - 1)).with(MaxWidth::wrapping(30).keep_words()))
        // 4 rows
        .with(Modify::new(Row(table_data.len()..)).with(MaxWidth::wrapping(30 * 4).keep_words()))
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
                    let operator = switch_string_to_url_encoding(curr_el_vec[1])?.trim();
                    // let operator = curr_el_vec[1];
                    let value = curr_el_vec[2].trim();

                    if !var_vec_check.contains(&variable)
                        && !var_vec_min_max_check.contains(&variable.to_string())
                    {
                        // might be able to check max/min/length here.
                        // e.g. max(gc_content) > 0.3
                        bail!(ExpressionParseError::SplitVectorError)
                    }

                    // this panics with min/max.
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
                            // split on commas here
                            // and trim
                            let value_split_commas = value
                                .split(",")
                                .map(|e| {
                                    let trimmed = e.trim();
                                    let remove_bools = trimmed.replace("!", "");
                                    remove_bools
                                })
                                .collect::<Vec<String>>();

                            if !value_split_commas.iter().all(|e| k.contains(&&e[..])) {
                                // BUG: if there are parentheses in the enum variant
                                // but I think GoaT central can fix this.
                                // then the URL encoding fails.
                                bail!(ExpressionParseError::KeywordEnumError)
                            }
                            // now modify value_split_commas to parse parentheses
                            let parsed_value_split_commas = value
                                .split(",")
                                .map(|e| {
                                    // trim again but keep bool flags
                                    let f = e.trim();
                                    // janky but will do for now.
                                    let f = f.replace("(", "%28");
                                    let f = f.replace(")", "%29");
                                    let f = f.replace(" ", "%20");
                                    let f = f.replace("!", "%21");
                                    f
                                })
                                .collect::<Vec<String>>();
                            // build expression
                            expression += "%20";
                            expression += &url_encoded_variable;
                            // do operators need to be translated?
                            expression += operator;
                            expression += &parsed_value_split_commas.join("%2C");
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
