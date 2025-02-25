use crate::error::{Error, ErrorKind, Result};
use crate::utils::variable_data;
use crate::utils::{tax_ranks::TaxRanks, utils, variables::Variables};
use crate::{TaxType, GOAT_URL, TAXONOMY};
use std::fmt;

// Report      | Required             | Optional
// ------------|----------------------|-------------------------------------------
// Files       | x                    | checkedFiles
// Histogram   | x                    | cat, catToX, rank, xOpts
// Map         | x                    | cat, rank
// Oxford      | x                    | cat, xOpts, yOpts
// Scatter     | x, y, rank           | cat, xOpts, yOpts, scatterThreshold
// Table       | x, y                 | cat, rank, xOpts, yOpts, scatterThreshold
// Sources     | -                    | -
// Tree        | x                    | y, cat, xOpts, yOpts, collapseMonotypic, treeThreshold
// arc         | x, rank              | y
// xPerRank    | x                    | ranks

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
    /// Implement [`fmt::Display`] for [`ReportType`] so we can
    /// use `.to_string()` method.
    ///
    /// Only tree is different for newick, otherwise, use table.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportType::Newick => write!(f, "tree"),
            ReportType::Sources => write!(f, "sources"),
            _ => write!(f, "table"),
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
    /// ```rust
    /// Opts {
    ///     min: Some(1),
    ///     max: Some(10),
    ///     tick_count: None,
    ///     scale: None,
    ///     axis_title: None        
    /// }
    /// ```
    ///
    /// ",,20":
    ///
    /// ```rust
    /// Opts {
    ///     min: None,
    ///     max: None,
    ///     tick_count: Some(20),
    ///     scale: None,
    ///     axis_title: None        
    /// }
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
                    .any(|e| **e == opts.scale.clone().unwrap_or_else(|| "".into()))
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
                    .any(|e| **e == opts.scale.clone().unwrap_or_else(|| "".into()))
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
            [min, max, tick_count, scale, axis_title].join("%2C")
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
}

impl Report {
    /// Constructor function for [`Report`].
    pub fn new(matches: &clap::ArgMatches, report_type: ReportType) -> Result<Self> {
        // create the default struct
        let mut report: Report = Report {
            report_type,
            ..Default::default()
        };

        // fill the mandatory fields.
        // search from CLI
        let search = matches
            .get_one::<String>("taxon")
            .expect("cli requires input");
        // TODO: could also take from file
        report.search = utils::parse_comma_separated(search);

        // safe to unwrap, as default is defined.
        report.rank = matches
            .get_one::<String>("rank")
            .expect("cli default = species")
            .to_string();
        // taxon type will be by default tax_tree(). change this here
        // for future reference. But will require a flag on the cli.

        // for newick
        let threshold = matches
            .get_one::<i32>("threshold")
            .expect("cli default 2000");
        report.threshold = *threshold;

        // the x string will be just a variable.
        let x_variable = matches.get_one::<String>("x-variable");

        if let Some(xvar) = x_variable {
            let inner_x =
                Variables::new(xvar).parse_one(&variable_data::GOAT_TAXON_VARIABLE_DATA)?;
            // assign to struct
            report.x = Some(inner_x);
        };

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

        // now the optionals.
        let y_variable = matches.get_one::<String>("y-variable");
        if let Some(y_var) = y_variable {
            report.y = Some(y_var.to_string());
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
                let mut url = String::new();
                url += &GOAT_URL;
                // add report API, and result=taxon
                url += "report?result=taxon";
                // it's a tree we're returning
                url += &format!("&report={}", self.report_type);
                // get a string of comma separated queries
                let csqs = match self.search.len() {
                    // one entry
                    1 => self.search[0].clone(),
                    // or greater (zero entries handled by cli)
                    _ => self.search.join("%2C"),
                };
                // the x value source
                let x_value_source = format!(
                    "&x=tax_rank%28{}%29%20AND%20tax_tree%28{}%29",
                    self.rank, csqs
                );
                url += &x_value_source;
                // default stuff for now
                url += &format!("&treeThreshold={}", self.threshold);
                url += "&includeEstimates=true";
                url += &format!("&taxonomy={}", &*TAXONOMY);
                // fix this for now, as only single requests can be submitted
                url += &format!("&queryId=goat_cli_{}", unique_ids[0]);
                Ok(url)
            }
            // Report      | Required             | Optional
            // ------------|----------------------|-------------------------------------------
            // Histogram   | x                    | cat, catToX, rank, xOpts
            ReportType::Histogram => {
                let mut url = String::new();
                // add base URL
                url += &GOAT_URL;
                // it's a taxon report
                url += "report?result=taxon";
                // default stuff for now
                // no estimates
                // not sure whether this should be true or not
                url += "&includeEstimates=true";
                // standard taxonomy
                url += &format!("&taxonomy={}", &*TAXONOMY);
                // it's a table
                url += &format!("&report={}", self.report_type);
                // add the rank from the CLI
                url += &format!("&rank={}", self.rank);
                // taxon type: tax_rank/tax_tree
                let taxon_type = self.taxon_type;
                // and the taxa
                let taxa = self.search.join("%2C");
                // and the variable
                let variable = self.x.clone().unwrap();
                // safe to unwrap this.
                let cat = self.category.clone().unwrap();
                url += &format!(
                    "&x={}%28{}%29%20AND%20{}&cat={}%5B{}%5D",
                    taxon_type,
                    taxa,
                    variable,
                    cat,
                    self.size.unwrap(),
                );
                // add x options if any
                if let Some(xopts) = &self.x_opts {
                    url += &format!("&xOpts={}", xopts);
                }

                Ok(url)
            }
            ReportType::Scatterplot => Err(Error::new(ErrorKind::Report(
                "Scatter plots are not yet implemented; please check back in the future!".into(),
            ))),
            ReportType::Arc => todo!(),
            ReportType::Sources => {
                Err(Error::new(ErrorKind::Report(
                    "Sources are not yet implemented; please check back in the future!".into(),
                )))
                // https://goat.genomehubs.org/reporturl?
                // query=tax_tree%2891896%5BOrobanchaceae%5D%29
                // &includeEstimates=false
                // &includeRawValues=false
                // &summaryValues=count
                // &result=taxon
                // &taxonomy=ncbi
                // &size=50
                // &queryId=goat_cli_du8TOca5YBVF4rL
                // &report=sources

                // this would work, but it's unimplemented yet
                // let mut url = String::new();
                // url += &GOAT_URL;
                // // it's a taxon report
                // url += "report?result=taxon";
                // url += "&includeEstimates=false";
                // url += "&includeRawValues=false";
                // // standard taxonomy
                // url += &format!("&taxonomy={}", &*TAXONOMY);
                // // it's a source
                // url += &format!("&report={}", self.report_type);
                // // taxon type: tax_rank/tax_tree
                // let taxon_type = self.taxon_type;
                // // and the taxa
                // let taxa = self.search.join("%2C");
                // url += &format!("&query={}%28{}%29", taxon_type, taxa);
                // // add unique_id?

                // Ok(url)
            }
        }
    }
}
