use crate::error::{Error, ErrorKind, Result};
use crate::utils::url::percent_encode_query_value;
use crate::utils::variable_data;
use crate::utils::{tax_ranks::TaxRanks, utils, variables::Variables};
use crate::{TaxType, GOAT_URL, TAXONOMY};
use std::fmt;
use url::Url;

// | Implemented | Report        | Required             | Optional
// --------------|---------------|-------------------------------------------
//               | Files         | x                    | checkedFiles
//               | Histogram     | x                    | cat, catToX, rank, xOpts
//               | Map           | x                    | cat, rank
//               | Oxford        | x                    | cat, xOpts, yOpts
// X             | Scatter       | x, y, rank           | cat, xOpts, yOpts, scatterThreshold
// X             | Table         | x, y                 | cat, rank, xOpts, yOpts, scatterThreshold
// X             | Sources       | -                    | -
// X (as newick) | Tree          | x                    | y, cat, xOpts, yOpts, collapseMonotypic, treeThreshold
// X             | arc           | x, rank              | y
//               | xPerRank      | x                    | ranks

// Search related parameters fields, includeEstimates, exclude* and queryId are optional for all reports (except sources where they have no effect).

/// The record type to return.
///
/// Should support all main report types, at least in their
/// basic forms.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    None,
    /// A Newick text string.
    #[default]
    Newick,
    /// A histogram, which is a single variable
    /// binned.
    Histogram,
    /// A scatterplot, requiring two variables.
    Scatterplot,
    /// Arc
    Arc,
    /// Sources
    Sources,
}

impl fmt::Display for ReportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportType::Newick => write!(f, "tree"),
            ReportType::Histogram => write!(f, "histogram"),
            ReportType::Scatterplot => write!(f, "scatter"),
            ReportType::Arc => write!(f, "arc"),
            ReportType::Sources => write!(f, "sources"),
            ReportType::None => write!(f, ""),
        }
    }
}

/// The x or y options for a returned table.
///
/// Argh these are going to be annoying to parse.
#[derive(Default, Debug)]
pub struct Opts {
    min: Option<i32>,
    max: Option<i32>,
    tick_count: Option<i32>,
    scale: Option<String>,
    axis_title: Option<String>,
}

impl Opts {
    /// The scale types that are possible in GoaT reports (I think).
    pub const SCALE_TYPES: [&'static str; 7] = [
        "linear",
        "sqrt",
        "log10",
        "log2",
        "log",
        "proportion",
        "ordinal",
    ];
    /// Try and parse a string of options into the CLI.
    ///
    /// "1,10":
    ///
    /// ```rust,ignore
    /// use goat_cli::report::report::Opts;
    /// let _ = Opts {
    ///     min: Some(1),
    ///     max: Some(10),
    ///     tick_count: None,
    ///     scale: None,
    ///     axis_title: None
    /// };
    /// ```
    ///
    /// ",,20":
    ///
    /// ```rust,ignore
    /// use goat_cli::report::report::Opts;
    /// let _ = Opts {
    ///     min: None,
    ///     max: None,
    ///     tick_count: Some(20),
    ///     scale: None,
    ///     axis_title: None
    /// };
    /// ```
    pub fn try_from_string(cli_opts: &str) -> Result<Self> {
        let mut tokens: Vec<Option<_>> = cli_opts
            .split(',')
            .map(|e| match e.trim() {
                "" => None,
                _ => Some(e),
            })
            .collect();
        // somehow remove trailing blanks
        tokens.reverse();

        // now skip while is None and collect back.
        let mut t: Vec<_> = tokens.iter().skip_while(|e| e.is_none()).collect();
        // turn them back!
        t.reverse();

        // create default struct
        let mut opts: Opts = Default::default();

        // parse the string to an integer, giving useful error if
        // we can't.
        fn parse_str_to_int(el: &Option<&str>) -> Result<Option<i32>> {
            // if we are calling this function, we can safely get
            // the value out
            let int_str = el;
            // bubble up the error here if parsing goes awry
            let int = match int_str {
                Some(e) => Some(e.trim().parse::<i32>().map_err(|e| {
                    let mut error = String::new();
                    error += "in parsing x or y options, ";
                    error += &e.to_string();
                    Error::new(ErrorKind::Report(error))
                })?),
                None => None,
            };

            Ok(int)
        }

        match t.len() {
            0 => return Err(Error::new(ErrorKind::Report("no options found".into()))),
            1 => {
                opts.min = parse_str_to_int(t[0])?;
            }
            2 => {
                opts.min = parse_str_to_int(t[0])?;
                opts.max = parse_str_to_int(t[1])?;
            }
            3 => {
                opts.min = parse_str_to_int(t[0])?;
                opts.max = parse_str_to_int(t[1])?;
                opts.tick_count = parse_str_to_int(t[2])?;
            }
            4 => {
                opts.min = parse_str_to_int(t[0])?;
                opts.max = parse_str_to_int(t[1])?;
                opts.tick_count = parse_str_to_int(t[2])?;
                opts.scale = t[3].map(|e| e.to_string());
                // bail if the scale isn't one we recognise
                if !Self::SCALE_TYPES
                    .iter()
                    .any(|e| **e == *opts.scale.as_deref().unwrap_or(""))
                {
                    return Err(Error::new(ErrorKind::Report(format!(
                        "Did not recognise scale type supplied. The options are: {}.",
                        Self::SCALE_TYPES.join(", ")
                    ))));
                }
            }
            5 => {
                opts.min = parse_str_to_int(t[0])?;
                opts.max = parse_str_to_int(t[1])?;
                opts.tick_count = parse_str_to_int(t[2])?;
                opts.scale = t[3].map(|e| e.to_string());
                // bail if the scale isn't one we recognise
                if !Self::SCALE_TYPES
                    .iter()
                    .any(|e| **e == *opts.scale.as_deref().unwrap_or(""))
                {
                    return Err(Error::new(ErrorKind::Report(format!(
                        "Did not recognise scale type supplied. The options are: {}.",
                        Self::SCALE_TYPES.join(", ")
                    ))));
                }
                opts.axis_title = t[4].map(|e| e.to_string());
            }
            _ => {
                return Err(Error::new(ErrorKind::Report(
                    "Too many tokens supplied to opts.".into(),
                )))
            }
        }

        Ok(opts)
    }
}

impl fmt::Display for Opts {
    /// Implement [`fmt::Display`] for [`Opts`] so we can
    /// use `.to_string()` method.
    ///
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = match self.min {
            Some(m) => format!("{}", m),
            None => "".into(),
        };
        let max = match self.max {
            Some(m) => format!("{}", m),
            None => "".into(),
        };
        let tick_count = match self.tick_count {
            Some(m) => format!("{}", m),
            None => "".into(),
        };
        let scale = match &self.scale {
            Some(m) => m.clone(),
            None => "".into(),
        };
        let axis_title = match &self.axis_title {
            Some(m) => m.clone(),
            None => "".into(),
        };

        write!(
            f,
            "{}",
            [min, max, tick_count, scale, axis_title].join(",")
        )
    }
}

/// The record struct to make URLs from.
#[derive(Default)]
pub struct Report {
    /// The type of the report; tree or table.
    pub report_type: ReportType,
    /// A vector of taxon ID's/names.
    pub search: Vec<String>,
    /// The rank of the return type.
    /// Default from CLI is species.
    pub rank: String,
    /// Taxon type: tax_tree or tax_name
    pub taxon_type: TaxType,
    /// The size of the result to return
    pub size: Option<usize>,
    // these below are optional extras, which are
    // needed for some report return types.
    /// The x value
    pub x: Option<String>,
    /// The y value. Required for Scatterplot.
    pub y: Option<String>,
    /// x options. Always optional.
    pub x_opts: Option<Opts>,
    /// The y options. Always optional.
    pub y_opts: Option<Opts>,
    /// The category. Required for CategoricalHistogram.
    pub category: Option<String>,
    /// The threshold. For Newick.
    pub threshold: i32,
    /// Fields to exclude if missing. For Arc.
    pub exclude_missing: Vec<String>,
    /// Fields to exclude if ancestral. For Arc.
    pub exclude_ancestral: Vec<String>,
}

impl Report {
    /// Constructor function for [`Report`].
    pub fn new(matches: &clap::ArgMatches, report_type: ReportType) -> Result<Self> {
        // create the default struct
        let mut report: Report = Report {
            report_type,
            ..Default::default()
        };

        // Taxon is optional for arc (global query), required for all other report types.
        if let Some(search) = matches.get_one::<String>("taxon") {
            report.search = utils::parse_comma_separated(search);
        }

        // safe to unwrap, as default is defined.
        report.rank = matches
            .get_one::<String>("rank")
            .expect("cli default = species")
            .to_string();
        // taxon type will be by default tax_tree(). change this here
        // for future reference. But will require a flag on the cli.

        report.threshold = matches
            .get_one::<i32>("threshold")
            .copied()
            .unwrap_or(2000);

        // Arc uses raw filter expressions; other reports use validated variable names.
        if report_type == ReportType::Arc {
            if let Some(xf) = matches.get_one::<String>("x-filter") {
                report.x = Some(xf.clone());
            }
            if let Some(yf) = matches.get_one::<String>("y-filter") {
                report.y = Some(yf.clone());
            }
            if let Some(em) = matches.get_one::<String>("exclude-missing") {
                report.exclude_missing = em.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            if let Some(ea) = matches.get_one::<String>("exclude-ancestral") {
                report.exclude_ancestral = ea.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
        } else {
            let x_variable = matches.get_one::<String>("x-variable");
            if let Some(xvar) = x_variable {
                let inner_x =
                    Variables::new(xvar).parse_one(&variable_data::GOAT_TAXON_VARIABLE_DATA)?;
                report.x = Some(inner_x);
            }

            let y_variable = matches.get_one::<String>("y-variable");
            if let Some(y_var) = y_variable {
                report.y = Some(y_var.to_string());
            }
        }

        // parse size
        let size = matches.get_one::<usize>("size");
        report.size = size.copied();

        // descendents (default) or not?
        let no_descendents = matches.get_one::<bool>("no-descendents");

        if let Some(desc) = no_descendents {
            if *desc {
                report.taxon_type = TaxType::Name;
            }
        }
        // x options
        let xopts = matches.get_one::<String>("x-opts");
        if let Some(x_opts) = xopts {
            report.x_opts = Some(Opts::try_from_string(x_opts)?);
        }
        // y options
        let yopts = matches.get_one::<String>("y-opts");
        if let Some(y_opts) = yopts {
            report.y_opts = Some(Opts::try_from_string(y_opts)?);
        }
        // category for histogram.
        let category = matches.get_one::<String>("category");
        if let Some(cat) = category {
            // FIXME: is this correct? Looks a bit wrong

            // check this variable against the various lists
            let parsed_taxon_rank = TaxRanks::parse(&TaxRanks::init(), cat, true).ok();
            let parsed_category = Variables::new(cat)
                .parse_one(&variable_data::GOAT_TAXON_VARIABLE_DATA)
                .ok();

            match parsed_taxon_rank {
                Some(tr) => {
                    report.category = Some(tr);
                    // kinda janky but works for now.
                    return Ok(report);
                }
                None => {
                    // propagate this error through
                    match parsed_category {
                        Some(pc) => {
                            report.category = Some(pc);
                            return Ok(report);
                        }
                        None => {
                            // TODO: this is quite a brutally large error, but can't think
                            // right now how to make it nicer.
                            return Err(Error::new(ErrorKind::Report(
                                "neither taxon rank or category specified".into(),
                            )));
                        }
                    }
                }
            }
        }

        Ok(report)
    }

    /// A function to construct the report URL for any kind of
    /// report.
    pub fn make_url(&self, unique_ids: Vec<String>) -> Result<String> {
        match self.report_type {
            ReportType::None => Err(Error::new(ErrorKind::Report(
                "No report type specified.".into(),
            ))),
            ReportType::Newick => {
                let base = format!("{}report", *GOAT_URL);
                let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
                let csqs = match self.search.len() {
                    1 => self.search[0].clone(),
                    _ => self.search.join(","),
                };
                let x_value =
                    format!("tax_rank({}) AND tax_tree({})", self.rank, csqs);
                let qp = format!(
                    "result=taxon&report={}&x={}&treeThreshold={}&includeEstimates=true&taxonomy={}&queryId=goat_cli_{}",
                    self.report_type,
                    percent_encode_query_value(&x_value),
                    self.threshold,
                    *TAXONOMY,
                    unique_ids[0],
                );
                url.set_query(Some(&qp));
                Ok(url.to_string())
            }
            // Report      | Required             | Optional
            // ------------|----------------------|-------------------------------------------
            // Histogram   | x                    | cat, catToX, rank, xOpts
            ReportType::Histogram => {
                let taxon_type = self.taxon_type;
                let taxa = self.search.join(",");
                let variable = self.x.as_deref().ok_or_else(|| {
                    Error::new(ErrorKind::Report(
                        "Histogram requires an x variable (--x-variable).".into(),
                    ))
                })?;

                let x_value = format!("{}({}) AND {}", taxon_type, taxa, variable);

                let base = format!("{}report", *GOAT_URL);
                let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
                let mut qp = format!(
                    "result=taxon&includeEstimates=true&taxonomy={}&report={}&rank={}&x={}&queryId=goat_cli_{}",
                    *TAXONOMY,
                    self.report_type,
                    percent_encode_query_value(&self.rank),
                    percent_encode_query_value(&x_value),
                    unique_ids[0],
                );
                if let Some(cat) = &self.category {
                    let cat_value = match self.size {
                        Some(sz) => format!("{}[{}]", cat, sz),
                        None => cat.clone(),
                    };
                    qp.push_str(&format!("&cat={}", percent_encode_query_value(&cat_value)));
                }
                if let Some(xopts) = &self.x_opts {
                    qp.push_str(&format!(
                        "&xOpts={}",
                        percent_encode_query_value(&xopts.to_string())
                    ));
                }
                url.set_query(Some(&qp));
                Ok(url.to_string())
            }
            // Scatter      | x, y, rank           | cat, xOpts, yOpts, scatterThreshold
            ReportType::Scatterplot => {
                let taxon_type = self.taxon_type;
                let taxa = self.search.join(",");
                let x_variable = self.x.as_deref().ok_or_else(|| {
                    Error::new(ErrorKind::Report(
                        "Scatter requires an x variable (--x-variable).".into(),
                    ))
                })?;
                let y_variable = self.y.as_deref().ok_or_else(|| {
                    Error::new(ErrorKind::Report(
                        "Scatter requires a y variable (--y-variable).".into(),
                    ))
                })?;

                let x_value = format!("{}({}) AND {}", taxon_type, taxa, x_variable);

                let base = format!("{}report", *GOAT_URL);
                let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
                let mut qp = format!(
                    "result=taxon&includeEstimates=true&taxonomy={}&report={}&rank={}&x={}&y={}&queryId=goat_cli_{}",
                    *TAXONOMY,
                    self.report_type,
                    percent_encode_query_value(&self.rank),
                    percent_encode_query_value(&x_value),
                    percent_encode_query_value(y_variable),
                    unique_ids[0],
                );
                if let Some(cat) = &self.category {
                    qp.push_str(&format!("&cat={}", percent_encode_query_value(cat)));
                }
                if let Some(xopts) = &self.x_opts {
                    qp.push_str(&format!(
                        "&xOpts={}",
                        percent_encode_query_value(&xopts.to_string())
                    ));
                }
                if let Some(yopts) = &self.y_opts {
                    qp.push_str(&format!(
                        "&yOpts={}",
                        percent_encode_query_value(&yopts.to_string())
                    ));
                }
                url.set_query(Some(&qp));
                Ok(url.to_string())
            }
            // Arc: x = numerator expression, y = denominator expression
            ReportType::Arc => {
                let x_filter = self.x.as_deref().ok_or_else(|| {
                    Error::new(ErrorKind::Report(
                        "Arc requires an x filter expression (--x-filter).".into(),
                    ))
                })?;

                // Without a taxon, x is used directly; with a taxon it is scoped to that clade.
                let x_value = if self.search.is_empty() {
                    x_filter.to_string()
                } else {
                    let taxon_type = self.taxon_type;
                    let taxa = self.search.join(",");
                    format!("{}({}) AND {}", taxon_type, taxa, x_filter)
                };

                let base = format!("{}report", *GOAT_URL);
                let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
                let mut qp = format!(
                    "result=taxon&taxonomy={}&includeEstimates=true&report={}&rank={}&x={}&queryId=goat_cli_{}",
                    *TAXONOMY,
                    self.report_type,
                    percent_encode_query_value(&self.rank),
                    percent_encode_query_value(&x_value),
                    unique_ids[0],
                );
                if let Some(y_filter) = &self.y {
                    qp.push_str(&format!("&y={}", percent_encode_query_value(y_filter)));
                }
                for (i, field) in self.exclude_missing.iter().enumerate() {
                    qp.push_str(&format!(
                        "&excludeMissing%5B{}%5D={}",
                        i,
                        percent_encode_query_value(field)
                    ));
                }
                for (i, field) in self.exclude_ancestral.iter().enumerate() {
                    qp.push_str(&format!(
                        "&excludeAncestral%5B{}%5D={}",
                        i,
                        percent_encode_query_value(field)
                    ));
                }
                url.set_query(Some(&qp));
                Ok(url.to_string())
            }
            // Sources      | -                    | -
            ReportType::Sources => {
                let taxon_type = self.taxon_type;
                let taxa = self.search.join(",");
                let x_value = format!("{}({})", taxon_type, taxa);

                let base = format!("{}report", *GOAT_URL);
                let mut url = Url::parse(&base).expect("GOAT_URL is a valid base");
                let qp = format!(
                    "result=taxon&includeEstimates=false&taxonomy={}&report={}&x={}&queryId=goat_cli_{}",
                    *TAXONOMY,
                    self.report_type,
                    percent_encode_query_value(&x_value),
                    unique_ids[0],
                );
                url.set_query(Some(&qp));
                Ok(url.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_report(report_type: ReportType) -> Report {
        Report {
            report_type,
            search: vec!["Homo sapiens".into()],
            rank: "species".into(),
            taxon_type: TaxType::Tree,
            ..Default::default()
        }
    }

    #[test]
    fn test_histogram_missing_x_returns_err() {
        let mut r = base_report(ReportType::Histogram);
        r.category = Some("sex".into());
        r.size = Some(10);
        let result = r.make_url(vec!["test_id".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("x variable"));
    }


    #[test]
    fn test_arc_missing_x_returns_err() {
        let r = base_report(ReportType::Arc);
        let result = r.make_url(vec!["test_id".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("x filter"));
    }

    #[test]
    fn test_arc_url_contains_required_parts() {
        let mut r = base_report(ReportType::Arc);
        r.x = Some("assembly_level >= scaffold".into());
        let url = r.make_url(vec!["arc_id".into()]).unwrap();
        assert!(url.contains("report=arc"));
        assert!(url.contains("assembly_level"));
        assert!(url.contains("result=taxon"));
        assert!(url.contains("queryId=goat_cli_arc_id"));
    }

    #[test]
    fn test_arc_url_with_y_filter() {
        let mut r = base_report(ReportType::Arc);
        r.x = Some("assembly_level >= scaffold".into());
        r.y = Some("assembly_span > 1000000000".into());
        let url = r.make_url(vec!["arc_id".into()]).unwrap();
        assert!(url.contains("assembly_span"));
        assert!(url.contains("&y="));
    }

    // ── Newick URL ───────────────────────────────────────────────────────────

    #[test]
    fn test_newick_url_contains_required_parts() {
        let r = Report {
            report_type: ReportType::Newick,
            search: vec!["Mammalia".into()],
            rank: "species".into(),
            taxon_type: TaxType::Tree,
            threshold: 2000,
            ..Default::default()
        };
        let url = r.make_url(vec!["test123".into()]).unwrap();
        assert!(url.contains("report=tree"));
        assert!(url.contains("Mammalia"));
        assert!(url.contains("treeThreshold=2000"));
        assert!(url.contains("queryId=goat_cli_test123"));
        assert!(url.contains("result=taxon"));
    }

    #[test]
    fn test_newick_url_multiple_taxa_joined() {
        let r = Report {
            report_type: ReportType::Newick,
            search: vec!["Mammalia".into(), "Aves".into()],
            rank: "species".into(),
            taxon_type: TaxType::Tree,
            threshold: 2000,
            ..Default::default()
        };
        let url = r.make_url(vec!["id1".into()]).unwrap();
        assert!(url.contains("Mammalia"));
        assert!(url.contains("Aves"));
        assert!(url.contains("%2C")); // URL-encoded comma joining the taxa
    }

    #[test]
    fn test_newick_url_threshold_value_in_url() {
        let mut r = base_report(ReportType::Newick);
        r.threshold = 500;
        let url = r.make_url(vec!["id1".into()]).unwrap();
        assert!(url.contains("treeThreshold=500"));
    }

    // ── Histogram URL success ────────────────────────────────────────────────

    #[test]
    fn test_histogram_url_success_contains_key_parts() {
        let mut r = base_report(ReportType::Histogram);
        r.x = Some("genome_size".into());
        r.category = Some("sex".into());
        r.size = Some(50);
        let url = r.make_url(vec!["id1".into()]).unwrap();
        assert!(url.contains("report=histogram"));
        assert!(url.contains("genome_size"));
        assert!(url.contains("sex"));
        assert!(url.contains("50"));
        assert!(url.contains("result=taxon"));
        assert!(url.contains("queryId=goat_cli_id1"));
    }

    #[test]
    fn test_histogram_url_no_cat_ok() {
        let mut r = base_report(ReportType::Histogram);
        r.x = Some("genome_size".into());
        let url = r.make_url(vec!["id1".into()]).unwrap();
        assert!(url.contains("report=histogram"));
        assert!(!url.contains("cat="));
    }

    // ── Scatterplot URL ──────────────────────────────────────────────────────

    #[test]
    fn test_scatter_missing_x_returns_err() {
        let mut r = base_report(ReportType::Scatterplot);
        r.y = Some("chromosome_number".into());
        let result = r.make_url(vec!["id1".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("x variable"));
    }

    #[test]
    fn test_scatter_missing_y_returns_err() {
        let mut r = base_report(ReportType::Scatterplot);
        r.x = Some("genome_size".into());
        let result = r.make_url(vec!["id1".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("y variable"));
    }

    #[test]
    fn test_scatter_url_contains_required_parts() {
        let mut r = base_report(ReportType::Scatterplot);
        r.x = Some("genome_size".into());
        r.y = Some("chromosome_number".into());
        let url = r.make_url(vec!["sc_id".into()]).unwrap();
        assert!(url.contains("report=scatter"));
        assert!(url.contains("genome_size"));
        assert!(url.contains("chromosome_number"));
        assert!(url.contains("result=taxon"));
        assert!(url.contains("queryId=goat_cli_sc_id"));
    }

    // ── Sources URL ──────────────────────────────────────────────────────────

    #[test]
    fn test_sources_url_contains_required_parts() {
        let r = base_report(ReportType::Sources);
        let url = r.make_url(vec!["src_id".into()]).unwrap();
        assert!(url.contains("report=sources"));
        assert!(url.contains("result=taxon"));
        assert!(url.contains("queryId=goat_cli_src_id"));
        assert!(url.contains("Homo%20sapiens"));
    }

    // ── Opts::try_from_string ────────────────────────────────────────────────

    #[test]
    fn test_opts_scale_valid() {
        let result = Opts::try_from_string("1,100,10,linear");
        assert!(result.is_ok());
    }

    #[test]
    fn test_opts_scale_invalid_returns_err() {
        let result = Opts::try_from_string("1,100,10,notascale");
        assert!(result.is_err());
    }

    #[test]
    fn test_opts_min_max_only() {
        let result = Opts::try_from_string("1,100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_opts_empty_returns_err() {
        let result = Opts::try_from_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_opts_non_integer_min_returns_err() {
        let result = Opts::try_from_string("notanumber,100");
        assert!(result.is_err());
    }

    #[test]
    fn test_opts_all_valid_scales() {
        for scale in Opts::SCALE_TYPES {
            let input = format!("1,100,10,{}", scale);
            assert!(
                Opts::try_from_string(&input).is_ok(),
                "scale '{}' should be valid",
                scale
            );
        }
    }
}
