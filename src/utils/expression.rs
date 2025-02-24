use crate::utils::tax_ranks::TaxRanks;
use crate::utils::utils::{did_you_mean, switch_string_to_url_encoding};

use crate::error::{Error, ErrorKind, Result};
use regex::{CaptureMatches, Captures, Regex};
use std::{collections::BTreeMap, fmt};
use tabled::{object::Rows, Modify, Panel, Table, Tabled, Width};

/// Serialize GoaT variables into their types.
///
/// See [here](https://www.elastic.co/guide/en/elasticsearch/reference/current/number.html)
/// for more details.
#[derive(Tabled)]
pub enum TypeOf<'a> {
    /// Signed 64 bit int.
    Long,
    /// Signed 16 bit int.
    Short,
    /// Float with one decimal place.
    OneDP,
    /// Float with two decimal places.
    TwoDP,
    /// Signed 32 bit int.
    Integer,
    /// A date.
    Date,
    /// Half precision 16 bit float.
    HalfFloat,
    /// A variable which itself is an enumeration.
    Keyword(Vec<&'a str>),
    /// None to catch parsing errors
    None,
}

impl<'a> TypeOf<'a> {
    /// Check the values input by a user, so `goat-cli` displays meaningful help.
    fn check(&self, other: &str, variable: &str) -> Result<()> {
        // we will have to parse the `other` conditionally on what the
        // `TypeOf` is.
        match self {
            TypeOf::Long => match other.parse::<i64>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass an integer as a value.") ))),
            },
            TypeOf::Short => match other.parse::<i16>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass an integer as a value.")))),
            },
            TypeOf::OneDP => match other.parse::<f32>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass a float as a value.")))),
            },
            TypeOf::TwoDP => match other.parse::<f32>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass a float as a value.")))),
            },
            TypeOf::Integer => match other.parse::<i32>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass an integer as a value.")))),
            },
            // dates should be in a specified format
            // yyyy-mm-dd
            TypeOf::Date => {
                let tokens = other.split('-').collect::<Vec<_>>();
                if tokens.len() == 1 || tokens.len() == 3 {
                   return Err(Error::new(ErrorKind::Expression("improperly formatted date. Please make sure date is in the format yyyy-mm-dd, or yyyy.".to_string())))
                    
                }
                            }
            TypeOf::HalfFloat => match other.parse::<f32>() {
                Ok(_) => (),
                Err(_) => return Err(Error::new(ErrorKind::Expression(format!("for variable \"{variable}\", an input error was found. Pass a float as a value.")))),
            },
            // keywords handled elsewhere
            TypeOf::Keyword(_) => (),
            // None to catch errors.
            TypeOf::None => (),
        };
        Ok(())
    }
}

impl<'a> fmt::Display for TypeOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // do nothing with None at the moment.
            TypeOf::None => write!(f, "Please don't use yet! This variable needs fixing."),
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

/// Kind of an option alias. Does a
/// particular variable have a function
/// associated with it? Usually min/max.
pub enum Function<'a> {
    None,
    Some(Vec<&'a str>),
}

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::None => write!(f, ""),
            Function::Some(fun) => write!(f, "{}", fun.join(", ")),
        }
    }
}

/// The GoaT variable of interest.
#[derive(Tabled)]
pub struct Variable<'a> {
    #[tabled(rename = "Display Name")]
    pub display_name: &'a str,
    #[tabled(rename = "Operators/Keywords")]
    pub type_of: TypeOf<'a>,
    #[tabled(rename = "Function(s)")]
    pub functions: Function<'a>,
}

/// The column headers for `goat-cli search --print-expression`.
#[derive(Tabled)]
struct ColHeader(#[tabled(rename = "Expression Name")] &'static str);

/// Print the table of GoaT variable data.
pub fn print_variable_data(data: &BTreeMap<&'static str, Variable<'static>>) {
    // for some space
    println!();
    // map the header to a tuple combination
    // see https://github.com/zhiburt/tabled/blob/master/README.md
    let table_data = data
        .iter()
        .map(|(e, f)| (ColHeader(e), f))
        .collect::<Vec<(ColHeader, &Variable)>>();
    // add taxon ranks at end...
    let footer_data = TaxRanks::init();

    let table_string = Table::new(&table_data)
        .with(Panel::footer(format!(
            "NCBI taxon ranks:\n\n{}",
            footer_data
        )))
        .with(Modify::new(Rows::new(1..table_data.len() - 1)).with(Width::wrap(30).keep_words()))
        // 4 rows
        .with(Modify::new(Rows::new(table_data.len()..)).with(Width::wrap(30 * 4).keep_words()))
        .to_string();

    println!("{}", table_string);
}

/// The CLI expression which needs to be parsed.
#[derive(Debug)]
pub struct CLIexpression<'a> {
    pub inner: &'a str,
    pub length: usize, // these queries can't be crazy long.
    pub expression: Vec<&'a str>,
}

impl<'a> CLIexpression<'a> {
    /// Constructor for [`CLIexpression`].
    pub fn new(string: &'a str) -> Self {
        Self {
            inner: string,
            length: string.len(),
            expression: Vec::new(),
        }
    }

    /// The initial split on the keyword `AND`.
    fn split(&self) -> Self {
        let mut res_vec = Vec::new();
        let re = Regex::new("AND").unwrap();
        let splitter = SplitCaptures::new(&re, self.inner);
        for state in splitter {
            let el = match state {
                SplitState::Unmatched(s) => s,
                SplitState::Captured(s) => s.get(0).map_or("", |m| m.as_str()),
            };
            res_vec.push(el);
        }
        Self {
            inner: self.inner,
            length: self.length,
            expression: res_vec,
        }
    }

    /// The main function which parses a [`CLIexpression`]. A bit of a
    /// monster of a function. Might need cleaning up at some point.
    // FIXME: this function can also be used in reports for the -x
    // so let's make it more general.
    pub fn parse(
        &mut self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
    ) -> Result<String> {
        // TODO: what's an acceptable limit here?
        let expression_length_limit = 100;
        if self.length > expression_length_limit {
            return Err(Error::new(ErrorKind::Expression(format!(
                "the query provided is greater than {} chars.",
                expression_length_limit
            ))))
        }
        // we don't use &&
        if self.inner.contains("&&") {
            return Err(Error::new(ErrorKind::Expression("use AND keyword, not && for queries.".to_string())))
        }
        // we don't use 'contains'
        if self.inner.contains(" contains") {
            return Err(Error::new(ErrorKind::Expression("using the \"contains\" keyword is not yet supported.".to_string())))
        }
        // we don't allow OR
        if self.inner.contains("||") || self.inner.contains("OR") {
            return Err(Error::new(ErrorKind::Expression("OR (or ||) keyword is not supported. Commas between categories operate like the OR keyword.".to_string())))
        }
        // for the more general expression case, we want to include this
        if self.inner.contains("tax_name")
            || self.inner.contains("tax_tree") || self.inner.contains("tax_lineage") {
            return Err(Error::new(ErrorKind::Expression("set tax_name through -t <taxon_name>, tax_tree by -d flag, and tax_lineage by -l flag.".to_string())))
        }
        // tax_rank is dealt with elsewhere
        if self.inner.contains("tax_rank") {
            return Err(Error::new(ErrorKind::Expression("set tax_rank through --tax-rank <taxon_rank>.".to_string())))
        }
        
        // essentially splitting on AND
        let split_vec = &self.split();
        eprintln!("split_vec: {:?}", split_vec);
        let exp_vec = &split_vec.expression;

        // split the expression vector into parts
        let mut index = 0;
        let exp_vec_len = exp_vec.len();
        let mut expression = String::new();
        // regular expression splitter
        // precedence here matters
        let re = Regex::new(r"=!|!=|<=|<|==|=|>=|>").unwrap();
        if !re.is_match(self.inner) {
            return Err(Error::new(ErrorKind::Expression("no operators were found in the expression.".to_string())))
        }

        // start with a space and AND
        expression += "%20AND";
        // vector of variables to check against
        let var_vec_check = &reference_data
            .iter()
            .map(|(e, _)| *e)
            .collect::<Vec<&str>>();
        // we can also create another vector of variables
        // with the appropriate max/min attached.
        let var_vec_functions_check = {
            let mut collector = Vec::new();
            for (goat_var, el) in reference_data {
                match &el.functions {
                    Function::None => {
                        // FIXME: this is a horrible hack. Not sure how expressions like this
                        // fit into the engine at the moment
                        collector.push(format!("length(long_list)"));
                    },
                    Function::Some(f) => {
                        // FIXME: some functions like 'length' are not yet in the reference data.
                        // so we can just add them here
                        for pos in f.iter().chain(["length"].iter()) {
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
            // OR a plain variable
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

            match curr_el_vec.len() {
                3 => {
                    // trim strings
                    // replace rogue quotes (not sure why this is happening now, but was not before...)
                    // manually escape these...
                    let variable = &curr_el_vec[0].trim().replace('\"', "").replace('\'', "")[..];
                    let operator = switch_string_to_url_encoding(curr_el_vec[1])?.trim();
                    let value = &curr_el_vec[2].trim().replace('\"', "").replace('\'', "")[..];

                    if !var_vec_check.contains(&variable)
                        && !var_vec_functions_check.contains(&variable.to_string())
                    {
                        // ew
                        // just combining the min/max and normal variable vectors
                        // into a single vector.
                        let combined_checks = var_vec_check
                            .iter()
                            .map(|e| String::from(*e))
                            .collect::<Vec<String>>()
                            .iter()
                            .chain(
                                var_vec_functions_check
                                    .iter()
                                    .map(String::from)
                                    .collect::<Vec<String>>()
                                    .iter(),
                            )
                            .map(String::from)
                            .collect::<Vec<String>>();

                        let var_vec_mean = did_you_mean(&combined_checks, variable);

                        if let Some(value) = var_vec_mean {
                            return Err(Error::new(ErrorKind::Expression(format!(
                                "in LHS you typed \"{}\" - did you mean \"{}\"?",
                                variable,
                                value
                            ))))
                        }
                    }

                    // this panics with min/max.
                    // if min/max present, extract within the parentheses.
                    let keyword_enums = match var_vec_functions_check.contains(&variable.to_string())
                    {
                        true => {
                            // this means we have min/max
                            let re = Regex::new(r"\((.*?)\)").unwrap();
                            // we guarantee getting here with a variable, so unwrap is fine
                            // the second unwrap is always guaranteed too?
                            let extract_var =
                                re.captures(variable).unwrap().get(1).unwrap().as_str();
                            &reference_data.get(extract_var).unwrap().type_of
                        }
                        false => &reference_data.get(variable).unwrap().type_of,
                    };

                    // if there are parentheses - i.e. in min()/max() functions
                    let url_encoded_variable = variable.replace('(', "%28");
                    let url_encoded_variable = url_encoded_variable.replace(')', "%29");

                    // if there are keywords, make sure they are a match
                    match keyword_enums {
                        TypeOf::Keyword(k) => {
                            // split on commas here
                            // and trim
                            let value_split_commas = value
                                .split(',')
                                .map(|e| {
                                    let trimmed = e.trim();
                                    trimmed.replace('!', "")
                                })
                                .collect::<Vec<String>>();

                            // now check our keyword enums
                            for val in &value_split_commas {
                                // if 'val' starts with PRJEB, continue
                                // as these are project identifiers and there are too many to enumerate.
                                // or if val is numeric
                                if val.starts_with("PRJEB") || val.parse::<i64>().is_ok() {
                                    continue;
                                }
                                
                                let possibilities =
                                    k.iter().map(|e| String::from(*e)).collect::<Vec<_>>();
                                let did_you_mean_str = did_you_mean(&possibilities, val);

                                if let Some(value) = did_you_mean_str {
                                    if value != *val {
                                        return Err(Error::new(ErrorKind::Expression(format!("in (RHS you typed \"{}\" - did you mean \"{}\"?", val, value))))
                                    }
                                }
                            }

                            // now modify value_split_commas to parse parentheses
                            let parsed_value_split_commas = value
                                .split(',')
                                .map(|e| {
                                    // trim again but keep bool flags
                                    let f = e.trim();
                                    // janky but will do for now.
                                    let f = f.replace('(', "%28");
                                    let f = f.replace(')', "%29");
                                    let f = f.replace(' ', "%20");
                                    f.replace('!', "%21")
                                })
                                .collect::<Vec<String>>();
                            // build expression
                            expression += "%20";
                            expression += &url_encoded_variable;
                            // do operators need to be translated?
                            expression += "%20";
                            expression += operator;
                            expression += "%20";
                            expression += &parsed_value_split_commas.join("%2C");
                            expression += "%20";
                            // end of sub expression
                            // assume there is another expression to follow
                            expression += "AND"
                        }
                        t => {
                            // here can we type check input
                            TypeOf::check(t, value, variable)?;

                            // build expression
                            expression += "%20";
                            expression += &url_encoded_variable;
                            // do operators need to be translated?
                            expression += "%20";
                            expression += operator;
                            expression += "%20";
                            expression += value;
                            expression += "%20";
                            // end of sub expression
                            // assume there is another expression to follow
                            expression += "AND"
                        }
                    }
                }
                1 => {
                    // if this is AND, don't do anything
                    if curr_el_vec[0].trim() != "AND" {
                        let variable = &curr_el_vec[0].trim().replace('\"', "").replace('\'', "")[..];
                        expression += "%20";
                        expression += &variable;
                        expression += "%20";
                        expression += "AND";
                    }
                },
                _ => return Err(Error::new(ErrorKind::Expression("are the input variables or operands correct?".to_string())))
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
                Err(Error::new(ErrorKind::Expression(format!("must be in the format: <variable> <operator> <value> AND ..."))))
            }
        }
    }
}

/// Split a string and keep the delimiter.
/// Thanks [`BurntSushi`](https://github.com/rust-lang/regex/issues/330)
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
            text,
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

#[cfg(test)]
mod tests {
    use crate::utils::variable_data::GOAT_TAXON_VARIABLE_DATA;

    use super::*;

    // The tests need to be able to parse:
    // bioproject=!PRJEB40665 AND long_list=dtol AND ebp_metric_date AND tax_rank(species)
    // bioproject%3D!PRJEB40665%20AND%20long_list%3Ddtol%20AND%20ebp_metric_date%20AND%20tax_rank%28species%29
    #[test]
    fn test_1() {
        let expression = "bioproject=!PRJEB40665 AND long_list=dtol AND ebp_metric_date AND tax_rank(species)";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);

        assert!(result.is_err())
    }

    #[test]
    fn test_1_1() {
        let expression = "bioproject=!PRJEB40665 AND long_list=dtol AND ebp_metric_date AND genome_size > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        
        assert_eq!(result.unwrap(), "%20AND%20bioproject%20%3D%21%20PRJEB40665%20AND%20long_list%20%3D%20dtol%20AND%20ebp_metric_date%20AND%20genome_size%20%3E%201000");
    }
    
    // long_list=dtol AND length(long_list)>1
    // long_list%3Ddtol%20AND%20length%28long_list%29>1
    #[test]
    fn test_2() {
        let expression = "long_list=dtol AND length(long_list)>1";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20long_list%20%3D%20dtol%20AND%20length%28long_list%29%20%3E%201");
    }

    // long_list=dtol AND sequencing_status
    // long_list%3Ddtol%20AND%20sequencing_status%20AND%20tax_rank(species)
    #[test]
    fn test_3() {
        let expression = "long_list=dtol AND sequencing_status";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20long_list%20%3D%20dtol%20AND%20sequencing_status");
    }

    // some basic expressions
    #[test]
    fn test_4() {
        let expression = "genome_size > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20genome_size%20%3E%201000");
    }

    #[test]
    fn test_4_1() {
        // we always pad spaces around operators
        let expression = "genome_size<1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20genome_size%20%3C%201000");
    }

    #[test]
    fn test_5() {
        let expression = "sequencing_status_dtol == published";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20sequencing_status_dtol%20%3D%3D%20published");
    }

    // and combining expressions
    #[test]
    fn test_6() {
        let expression = "genome_size > 1000 AND sequencing_status_dtol == published";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA);
        assert_eq!(result.unwrap(), "%20AND%20genome_size%20%3E%201000%20AND%20sequencing_status_dtol%20%3D%3D%20published");
    }
}
