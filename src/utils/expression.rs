use crate::utils::tax_ranks::TaxRanks;
use crate::utils::utils::did_you_mean;
use crate::utils::variable_data::GOAT_VARIABLE_SYNONYMS;

use crate::error::{Error, ErrorKind, Result};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::sync::LazyLock;
use tabled::{object::Rows, Modify, Panel, Table, Tabled, Width};

/// Summary functions that can wrap a field in an expression, e.g.
/// `max(genome_size)`. Mirrors the API's own list (`functions/summaries.js`),
/// minus its internal `metadata` and `hexbin*` summaries.
pub const VALID_EXPRESSION_FUNCTIONS: &[&str] =
    &["min", "max", "count", "length", "sp_count", "range", "value"];

/// Functions which return a count, whatever the type of the field.
const COUNT_FUNCTIONS: &[&str] = &["count", "length", "sp_count"];

/// Aggregation subset specifiers that may follow a field name with a colon.
const VALID_SUBSETS: &[&str] = &["direct", "ancestor", "descendant", "estimate"];

/// The type checked against the value of a count function.
static COUNT_TYPE: TypeOf<'static> = TypeOf::Long;

/// The API splits on `or`, then `and`, case-insensitively and only where
/// surrounded by whitespace (so values such as `island` are safe).
static OR_SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\s+or\s+").unwrap());
static AND_SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\s+and\s+").unwrap());
/// `<lhs> <operator> <rhs>`. Two character operators must come first.
static CLAUSE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<lhs>[^!<>=]+?)\s*(?P<op>!=|<=|>=|==|<|>|=)\s*(?P<rhs>.*)$").unwrap()
});
/// A summary function wrapping a field, e.g. `max(genome_size:direct)`.
static FUNCTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<func>[a-z_]+)\(\s*(?P<field>[^()]*?)\s*\)$").unwrap());
/// A number with a size suffix, e.g. `1G`, which the API does not accept.
static NUMBER_WITH_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^-?\d+(\.\d+)?\s*[kmgtp]b?$").unwrap());

fn expression_error(message: String) -> Error {
    Error::new(ErrorKind::Expression(message))
}

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
    /// A keyword. If the list of values is not empty, the API rejects
    /// any other value.
    Keyword(Vec<&'a str>),
    /// A keyword with a list of known values, which the API does not
    /// enforce; other values are allowed but may match nothing.
    KeywordSuggest(Vec<&'a str>),
    /// Not usable in expressions (e.g. `geo_point`), so not checked.
    None,
}

impl<'a> TypeOf<'a> {
    /// Check the right hand side of a clause, so `goat-cli` displays
    /// meaningful help before the query is sent.
    ///
    /// `rhs` may be a comma separated list, each value may be negated with a
    /// leading `!`, and `null` is allowed for any type.
    fn check(&self, rhs: &str, variable: &str) -> Result<()> {
        for value in rhs.split(',') {
            let value = value.trim().trim_start_matches('!').trim();
            if value.is_empty() {
                return Err(expression_error(format!(
                    "missing value for \"{variable}\" in \"{rhs}\"."
                )));
            }
            if value.eq_ignore_ascii_case("null") {
                continue;
            }
            match self {
                TypeOf::Long
                | TypeOf::Short
                | TypeOf::OneDP
                | TypeOf::TwoDP
                | TypeOf::Integer
                | TypeOf::HalfFloat => check_number(value, variable)?,
                TypeOf::Date => check_date(value, variable)?,
                TypeOf::Keyword(allowed) if !allowed.is_empty() => {
                    if let Some(suggestion) = unknown_keyword(value, allowed) {
                        return Err(expression_error(format!(
                            "\"{value}\" is not a valid value for \"{variable}\" - did you mean \"{suggestion}\"?"
                        )));
                    }
                }
                TypeOf::KeywordSuggest(known) => {
                    if let Some(suggestion) = unknown_keyword(value, known) {
                        eprintln!(
                            "warning: \"{value}\" is not a known value for \"{variable}\" (did you mean \"{suggestion}\"?), so may match nothing."
                        );
                    }
                }
                TypeOf::Keyword(_) | TypeOf::None => (),
            }
        }
        Ok(())
    }
}

/// The API accepts anything JavaScript parses as a number, including
/// scientific notation (`1e9`) and negative numbers.
fn check_number(value: &str, variable: &str) -> Result<()> {
    if value
        .replace('−', "-")
        .parse::<f64>()
        .map_or(false, f64::is_finite)
    {
        return Ok(());
    }
    let hint = if NUMBER_WITH_SUFFIX.is_match(value) {
        " Size suffixes are not supported; use scientific notation instead, e.g. 1e9."
    } else {
        ""
    };
    Err(expression_error(format!(
        "for variable \"{variable}\", \"{value}\" is not a number.{hint}"
    )))
}

/// Dates may be given as `yyyy`, `yyyy-mm` or `yyyy-mm-dd`.
fn check_date(value: &str, variable: &str) -> Result<()> {
    let digits = |s: &str, n: usize| s.len() == n && s.chars().all(|c| c.is_ascii_digit());
    let in_range = |s: &str, max: u32| s.parse::<u32>().map_or(false, |v| (1..=max).contains(&v));
    let valid = match value.split('-').collect::<Vec<_>>().as_slice() {
        [y] => digits(y, 4),
        [y, m] => digits(y, 4) && digits(m, 2) && in_range(m, 12),
        [y, m, d] => {
            digits(y, 4) && digits(m, 2) && in_range(m, 12) && digits(d, 2) && in_range(d, 31)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(expression_error(format!(
            "for variable \"{variable}\", \"{value}\" is not a date. Use yyyy, yyyy-mm or yyyy-mm-dd."
        )))
    }
}

/// If `value` is not (case-insensitively) one of `allowed`, return the
/// closest match.
fn unknown_keyword(value: &str, allowed: &[&str]) -> Option<String> {
    if allowed.iter().any(|a| a.eq_ignore_ascii_case(value)) {
        return None;
    }
    let possibilities = allowed.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    Some(did_you_mean(&possibilities, value).unwrap_or_default())
}

impl<'a> fmt::Display for TypeOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // do nothing with None at the moment.
            TypeOf::None => write!(f, "Please don't use yet! This variable needs fixing."),
            TypeOf::Long
            | TypeOf::Short
            | TypeOf::OneDP
            | TypeOf::TwoDP
            | TypeOf::Integer
            | TypeOf::Date
            | TypeOf::HalfFloat => write!(f, "!=, <, <=, =, ==, >, >="),
            TypeOf::Keyword(k) | TypeOf::KeywordSuggest(k) if k.is_empty() => write!(f, ""),
            TypeOf::Keyword(k) | TypeOf::KeywordSuggest(k) => write!(f, "== {}", k.join(", ")),
        }
    }
}

/// Resolve a field name as the API does: case-insensitively, accepting
/// synonyms (e.g. `ebp_metric_date`) and `-` in place of `_`.
pub fn canonical_field(
    name: &str,
    reference_data: &BTreeMap<&'static str, Variable<'static>>,
) -> Option<&'static str> {
    let lookup = |n: &str| {
        reference_data
            .keys()
            .find(|k| k.eq_ignore_ascii_case(n))
            .copied()
    };
    lookup(name)
        .or_else(|| lookup(&name.replace('-', "_")))
        .or_else(|| {
            GOAT_VARIABLE_SYNONYMS
                .get(name.to_lowercase().as_str())
                .and_then(|canonical| lookup(canonical))
        })
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
pub fn print_variable_data(data: &BTreeMap<&'static str, Variable<'static>>) -> Result<()> {
    // for some space
    crate::outln!()?;
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

    crate::outln!("{}", table_string)?;
    Ok(())
}

/// The CLI expression which needs to be parsed.
///
/// See `EXPRESSIONS.md` for the syntax.
#[derive(Debug)]
pub struct CLIexpression<'a> {
    pub inner: &'a str,
}

impl<'a> CLIexpression<'a> {
    /// Constructor for [`CLIexpression`].
    pub fn new(string: &'a str) -> Self {
        Self { inner: string }
    }

    /// Validate an expression and return it in canonical form, as
    /// `" AND <clause> AND <clause> OR <clause> ..."`, ready to follow a
    /// taxon term.
    ///
    /// As in the API, `OR` binds more loosely than `AND`, so the result is a
    /// list of `AND` groups joined by `" OR "`. The taxon and rank must apply
    /// to every group; [`crate::utils::url::make_goat_urls`] does that.
    pub fn parse(
        &mut self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
        extra_fields: Option<&HashSet<String>>,
    ) -> Result<String> {
        let input = self.inner.trim();
        if input.is_empty() {
            return Err(expression_error("the expression is empty.".to_string()));
        }
        if input.contains("&&") {
            return Err(expression_error(
                "use the AND keyword, not &&, between clauses.".to_string(),
            ));
        }
        if input.contains("||") {
            return Err(expression_error(
                "use the OR keyword, not ||, between clauses.".to_string(),
            ));
        }

        let mut groups = Vec::new();
        for group in OR_SPLIT.split(input) {
            let clauses = AND_SPLIT
                .split(strip_outer_parens(group.trim()))
                .map(|clause| parse_clause(clause, reference_data, extra_fields))
                .collect::<Result<Vec<_>>>()?;
            groups.push(clauses.join(" AND "));
        }
        Ok(format!(" AND {}", groups.join(" OR ")))
    }

    /// Whether any clause names a field that `reference_data` does not know,
    /// which is when the live field registry is worth loading. Unlike
    /// [`CLIexpression::parse`], this prints no warnings.
    pub fn has_unknown_field(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
    ) -> bool {
        OR_SPLIT
            .split(self.inner.trim())
            .flat_map(|group| AND_SPLIT.split(strip_outer_parens(group.trim())))
            .any(|clause| {
                let clause = clause.replace(['"', '\''], "");
                let clause = clause.trim();
                let lhs = CLAUSE
                    .captures(clause)
                    .map_or(clause, |caps| caps.name("lhs").unwrap().as_str());
                !lhs.trim().to_lowercase().starts_with("tax_")
                    && resolve_lhs(lhs, reference_data, None).is_err()
            })
    }
}

/// Remove parentheses wrapping a whole `OR` branch, e.g. `(a AND b)`.
fn strip_outer_parens(group: &str) -> &str {
    let Some(inner) = group.strip_prefix('(').and_then(|g| g.strip_suffix(')')) else {
        return group;
    };
    // make sure the first `(` closes at the very end, not e.g. `(a) AND (b)`
    let mut depth = 0i32;
    for c in inner.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => (),
        }
        if depth < 0 {
            return group;
        }
    }
    inner.trim()
}

/// Validate a single clause, returning it in canonical form.
fn parse_clause(
    clause: &str,
    reference_data: &BTreeMap<&'static str, Variable<'static>>,
    extra_fields: Option<&HashSet<String>>,
) -> Result<String> {
    // quotes are not needed by the API, even for values containing spaces
    let clause = clause.replace(['"', '\''], "");
    let clause = clause.trim();
    if clause.is_empty() {
        return Err(expression_error(
            "found an empty clause - check for a doubled or trailing AND/OR.".to_string(),
        ));
    }
    if clause.matches('(').count() != clause.matches(')').count() {
        return Err(expression_error(format!(
            "unbalanced parentheses in \"{clause}\". Parentheses may only wrap a whole OR branch, as GoaT does not support nesting: write \"a AND (b OR c)\" as \"a AND b OR a AND c\"."
        )));
    }
    let lower = clause.to_lowercase();
    if lower.starts_with("tax_rank") {
        return Err(expression_error(
            "set tax_rank through --tax-rank <taxon_rank>.".to_string(),
        ));
    }
    if lower.starts_with("tax_") {
        return Err(expression_error("set tax_name through -t <taxon_name>, tax_tree by -d flag, and tax_lineage by -l flag.".to_string()));
    }

    let Some(caps) = CLAUSE.captures(clause) else {
        // a bare field, which means "has a value"
        if clause.contains(char::is_whitespace) {
            return Err(expression_error(format!(
                "\"{clause}\" is not a valid clause. Expected <field> <operator> <value>, e.g. \"long_list = dtol\", or a bare field name."
            )));
        }
        let (lhs, _) = resolve_lhs(clause, reference_data, extra_fields)?;
        return Ok(lhs);
    };

    let (lhs, type_of) = resolve_lhs(&caps["lhs"], reference_data, extra_fields)?;
    let rhs = caps["rhs"].trim();
    if rhs.is_empty() {
        return Err(expression_error(format!("missing value in \"{clause}\".")));
    }
    if let Some(type_of) = type_of {
        type_of.check(rhs, &lhs)?;
    }
    let values = rhs.split(',').map(str::trim).collect::<Vec<_>>().join(",");
    Ok(format!("{} {} {}", lhs, &caps["op"], values))
}

/// Validate the left hand side of a clause: a field, optionally with a
/// `:subset` and/or wrapped in a summary function. Returns the canonical
/// form, and the type to check values against (`None` for fields only known
/// from the live registry, and identifiers such as `taxon_id`).
fn resolve_lhs<'d>(
    lhs: &str,
    reference_data: &'d BTreeMap<&'static str, Variable<'static>>,
    extra_fields: Option<&HashSet<String>>,
) -> Result<(String, Option<&'d TypeOf<'static>>)> {
    let lhs = lhs.trim().to_lowercase();
    let (function, field) = match FUNCTION.captures(&lhs) {
        Some(caps) => (Some(caps["func"].to_string()), caps["field"].to_string()),
        None => (None, lhs.clone()),
    };
    if let Some(f) = &function {
        if !VALID_EXPRESSION_FUNCTIONS.contains(&f.as_str()) {
            return Err(expression_error(format!(
                "unknown function \"{f}\" in \"{lhs}\" - valid functions are: {}.",
                VALID_EXPRESSION_FUNCTIONS.join(", ")
            )));
        }
    }

    let (name, subset) = match field.split_once(':') {
        Some((name, subset)) => (name.trim(), Some(subset.trim())),
        None => (field.as_str(), None),
    };
    if let Some(subset) = subset {
        if !VALID_SUBSETS.contains(&subset) {
            return Err(expression_error(format!(
                "unknown field subset \":{}\" - valid subsets are: {}",
                subset,
                VALID_SUBSETS.join(", ")
            )));
        }
    }
    if name.is_empty() || name.contains(|c: char| c.is_whitespace() || c == '(' || c == ')') {
        return Err(expression_error(format!(
            "\"{lhs}\" is not a valid field name."
        )));
    }

    let is_dynamic = extra_fields.map_or(false, |fields| fields.contains(name));
    let (name, type_of) = match canonical_field(name, reference_data) {
        Some(canonical) => (canonical.to_string(), Some(&reference_data[canonical].type_of)),
        // identifiers are matched by the API, e.g. taxon_id = 9606
        None if is_dynamic || name.ends_with("_id") => (name.to_string(), None),
        None => {
            let possibilities = reference_data
                .keys()
                .map(|k| k.to_string())
                .chain(GOAT_VARIABLE_SYNONYMS.keys().map(|k| k.to_string()))
                .collect::<Vec<_>>();
            let hint = did_you_mean(&possibilities, name)
                .map(|s| format!(" - did you mean \"{s}\"?"))
                .unwrap_or_default();
            return Err(expression_error(format!(
                "unknown variable \"{name}\"{hint}"
            )));
        }
    };

    let type_of = match function.as_deref() {
        Some(f) if COUNT_FUNCTIONS.contains(&f) => Some(&COUNT_TYPE),
        _ => type_of,
    };
    let field = match subset {
        Some(subset) => format!("{name}:{subset}"),
        None => name,
    };
    let lhs = match function {
        Some(f) => format!("{f}({field})"),
        None => field,
    };
    Ok((lhs, type_of))
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
        let expression =
            "bioproject=!PRJEB40665 AND long_list=dtol AND ebp_metric_date AND tax_rank(species)";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);

        assert!(result.is_err())
    }

    #[test]
    fn test_1_1() {
        // =! is not a valid operator; `!` is a value-negation prefix.
        // bioproject=!PRJEB40665 → operator `=`, value `!PRJEB40665`.
        let expression =
            "bioproject=!PRJEB40665 AND long_list=dtol AND ebp_metric_date AND genome_size > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);

        // ebp_metric_date is a synonym of ebp_standard_date
        assert_eq!(
            result.unwrap(),
            " AND bioproject = !PRJEB40665 AND long_list = dtol AND ebp_standard_date AND genome_size > 1000"
        );
    }

    #[test]
    fn test_2() {
        let expression = "long_list=dtol AND length(long_list)>1";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(
            result.unwrap(),
            " AND long_list = dtol AND length(long_list) > 1"
        );
    }

    #[test]
    fn test_3() {
        let expression = "long_list=dtol AND sequencing_status";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND long_list = dtol AND sequencing_status");
    }

    #[test]
    fn test_4() {
        let expression = "genome_size > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND genome_size > 1000");
    }

    #[test]
    fn test_4_1() {
        // we always pad spaces around operators
        let expression = "genome_size<1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND genome_size < 1000");
    }

    #[test]
    fn test_5() {
        let expression = "sequencing_status_dtol == published";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND sequencing_status_dtol == published");
    }

    #[test]
    fn test_6() {
        let expression = "genome_size > 1000 AND sequencing_status_dtol == published";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(
            result.unwrap(),
            " AND genome_size > 1000 AND sequencing_status_dtol == published"
        );
    }

    #[test]
    fn test_field_subset_direct() {
        let expression = "genome_size:direct > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND genome_size:direct > 1000");
    }

    #[test]
    fn test_field_subset_invalid_rejected() {
        let expression = "genome_size:bogus > 1000";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bogus"));
    }

    #[test]
    fn test_null_value_accepted() {
        let expression = "genome_size = null";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND genome_size = null");
    }

    #[test]
    fn test_not_null_value_accepted() {
        let expression = "genome_size != !null";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert_eq!(result.unwrap(), " AND genome_size != !null");
    }

    #[test]
    fn test_extra_fields_dynamic_field_accepted() {
        let mut extra = HashSet::new();
        extra.insert("some_new_goat_field".to_string());
        let expression = "some_new_goat_field > 5";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, Some(&extra));
        assert_eq!(result.unwrap(), " AND some_new_goat_field > 5");
    }

    #[test]
    fn test_extra_fields_unknown_still_rejected() {
        let expression = "truly_unknown_field > 5";
        let mut cli_exp = CLIexpression::new(expression);
        let result = cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_operators_without_spaces() {
        for op in ["!=", "<", "<=", "=", "==", ">", ">="] {
            let expression = format!("genome_size{op}1000");
            let mut cli_exp = CLIexpression::new(&expression);
            assert_eq!(
                cli_exp.parse(&GOAT_TAXON_VARIABLE_DATA, None).unwrap(),
                format!(" AND genome_size {op} 1000")
            );
        }
    }

    #[test]
    fn test_date_check_accepts_year() {
        let t = TypeOf::Date;
        assert!(t.check("2024", "assembly_date").is_ok());
    }

    #[test]
    fn test_date_check_accepts_full_date() {
        let t = TypeOf::Date;
        assert!(t.check("2024-03-10", "assembly_date").is_ok());
    }

    #[test]
    fn test_date_check_accepts_year_month() {
        let t = TypeOf::Date;
        assert!(t.check("2024-03", "assembly_date").is_ok());
    }

    #[test]
    fn test_date_check_rejects_bad_token_count() {
        let t = TypeOf::Date;
        assert!(t.check("2024-03-10-01", "assembly_date").is_err());
    }

    #[test]
    fn test_date_check_rejects_bad_month() {
        let t = TypeOf::Date;
        assert!(t.check("2024-13", "assembly_date").is_err());
    }

    #[test]
    fn test_date_check_rejects_non_numeric_year() {
        let t = TypeOf::Date;
        assert!(t.check("abcd", "assembly_date").is_err());
    }

    #[test]
    fn test_date_check_rejects_bad_full_date_shape() {
        let t = TypeOf::Date;
        assert!(t.check("2024-3-10", "assembly_date").is_err());
    }

    fn parse(expression: &str) -> Result<String> {
        CLIexpression::new(expression).parse(&GOAT_TAXON_VARIABLE_DATA, None)
    }

    #[test]
    fn test_lowercase_and() {
        assert_eq!(
            parse("genome_size > 1000 and c_value < 5").unwrap(),
            " AND genome_size > 1000 AND c_value < 5"
        );
    }

    #[test]
    fn test_and_inside_a_value_does_not_split() {
        // "island" contains "and", but not surrounded by whitespace
        assert!(parse("country_list = island").is_ok());
    }

    #[test]
    fn test_or_groups() {
        assert_eq!(
            parse("genome_size > 1e9 AND c_value > 1 OR assembly_level = chromosome").unwrap(),
            " AND genome_size > 1e9 AND c_value > 1 OR assembly_level = chromosome"
        );
    }

    #[test]
    fn test_or_lowercase_with_parentheses() {
        assert_eq!(
            parse("(genome_size > 1e9 AND c_value > 1) or (assembly_level = chromosome)").unwrap(),
            " AND genome_size > 1e9 AND c_value > 1 OR assembly_level = chromosome"
        );
    }

    #[test]
    fn test_nested_parentheses_rejected() {
        let err = parse("genome_size > 1 AND (c_value > 1 OR c_value < 0)").unwrap_err();
        assert!(err.to_string().contains("nesting"), "{}", err);
    }

    #[test]
    fn test_symbolic_boolean_operators_rejected() {
        assert!(parse("genome_size > 1 && c_value > 1").is_err());
        assert!(parse("genome_size > 1 || c_value > 1").is_err());
    }

    #[test]
    fn test_scientific_notation_and_negative_numbers() {
        assert!(parse("genome_size > 1e9").is_ok());
        assert!(parse("genome_size > 1.5E9").is_ok());
        assert!(parse("c_value > -1").is_ok());
    }

    #[test]
    fn test_size_suffix_rejected_with_hint() {
        let err = parse("genome_size > 1G").unwrap_err();
        assert!(err.to_string().contains("scientific notation"), "{}", err);
    }

    #[test]
    fn test_enforced_keyword_is_case_insensitive() {
        assert_eq!(
            parse("assembly_level = Chromosome").unwrap(),
            " AND assembly_level = Chromosome"
        );
    }

    #[test]
    fn test_enforced_keyword_rejects_unknown_value() {
        let err = parse("assembly_level = chromosom").unwrap_err();
        assert!(err.to_string().contains("did you mean \"chromosome\""), "{}", err);
    }

    #[test]
    fn test_unenforced_keyword_accepts_unknown_value() {
        // long_list has known values, but the API accepts any value
        assert!(parse("long_list = DTOL").is_ok());
        assert!(parse("long_list = some_new_project").is_ok());
    }

    #[test]
    fn test_free_text_keyword_accepts_any_value() {
        assert!(parse("bioproject = PRJNA533106").is_ok());
    }

    #[test]
    fn test_multi_word_values_and_quotes() {
        assert_eq!(
            parse("assembly_level = chromosome, \"complete genome\"").unwrap(),
            " AND assembly_level = chromosome,complete genome"
        );
    }

    #[test]
    fn test_negated_list_values() {
        assert_eq!(
            parse("assembly_level = chromosome,!scaffold").unwrap(),
            " AND assembly_level = chromosome,!scaffold"
        );
    }

    #[test]
    fn test_keyword_range_operator() {
        assert_eq!(
            parse("assembly_level >= scaffold").unwrap(),
            " AND assembly_level >= scaffold"
        );
    }

    #[test]
    fn test_field_synonym_and_hyphen_and_case() {
        assert_eq!(parse("ebp_metric_date >= 2023").unwrap(), " AND ebp_standard_date >= 2023");
        assert_eq!(parse("genome-size > 1").unwrap(), " AND genome_size > 1");
        assert_eq!(parse("Genome_Size > 1").unwrap(), " AND genome_size > 1");
    }

    #[test]
    fn test_hyphenated_field_name() {
        assert!(parse("marhabreg-2017 = yes").is_ok());
    }

    #[test]
    fn test_identifier_terms() {
        assert_eq!(parse("taxon_id = 9606,9598").unwrap(), " AND taxon_id = 9606,9598");
    }

    #[test]
    fn test_bare_field_is_validated() {
        let err = parse("genome_sizee").unwrap_err();
        assert!(err.to_string().contains("did you mean \"genome_size\""), "{}", err);
    }

    #[test]
    fn test_contains_rejected_with_hint() {
        let err = parse("long_list contains dtol").unwrap_err();
        assert!(err.to_string().contains("long_list = dtol"), "{}", err);
    }

    #[test]
    fn test_count_function_takes_integer() {
        assert!(parse("count(assembly_level) > 1").is_ok());
        assert!(parse("length(long_list) > x").is_err());
    }

    #[test]
    fn test_unknown_function_rejected() {
        let err = parse("mean(genome_size) > 1").unwrap_err();
        assert!(err.to_string().contains("unknown function"), "{}", err);
    }

    #[test]
    fn test_long_expressions_allowed() {
        let expression = vec!["genome_size > 1"; 20].join(" AND ");
        assert!(parse(&expression).is_ok());
    }

    #[test]
    fn test_trailing_and_rejected() {
        assert!(parse("genome_size > 1 AND ").is_err());
    }
}
