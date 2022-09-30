use crate::utils::variable_data;
use crate::utils::{tax_ranks::TaxRanks, utils, variables::Variables};
use crate::{TaxType, GOAT_URL, TAXONOMY};
use anyhow::{bail, Context, Result};
use std::fmt;

// only taxon supported currently.
// TODO: think about how to support assembly or
// other indexes later.

/// The record type to return.
///
/// Should support all main report types, at least in their
/// basic forms.
#[derive(Default, Clone, Copy)]
pub enum ReportType {
    /// A Newick text string.
    #[default]
    Newick,
    /// A histogram, which is a single variable
    /// binned.
    Histogram,
    /// A histogram binned by category.
    CategoricalHistogram,
    /// A scatterplot, requiring two variables.
    Scatterplot,
}

impl fmt::Display for ReportType {
    /// Implement [`Display`] for [`ReportType`] so we can
    /// use `.to_string()` method.
    ///
    /// Only tree is different for newick, otherwise, use table.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportType::Newick => write!(f, "tree"),
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
    pub const SCALE_TYPES: [&str; 7] = [
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
    /// e.g. 1,10 () =>
    ///     
    /// Opts {
    ///     min: Some(1),
    ///     max: Some(10),
    ///     tick_count: None,
    ///     scale: None,
    ///     axis_title: None        
    /// }
    ///
    /// ,,20 (20 x axis ticks)
    /// Opts {
    ///     min: None,
    ///     max: None,
    ///     tick_count: Some(20),
    ///     scale: None,
    ///     axis_title: None        
    /// }
    /// TODO: test this
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
            let int =
                match int_str {
                    Some(e) => Some(e.trim().parse::<i32>().context(
                        "Could not convert what should be an integer in the options flag.",
                    )?),
                    None => None,
                };

            Ok(int)
        }

        match t.len() {
            0 => bail!("Can't split no tokens in the x or y options."),
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
                    .any(|e| **e == opts.scale.clone().unwrap_or("".into()))
                {
                    bail!("Did not recognise scale type supplied. The options are linear, sqrt, log10, log2, log, proportion, or ordinal.")
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
                    .any(|e| **e == opts.scale.clone().unwrap_or("".into()))
                {
                    bail!("Did not recognise scale type supplied. The options are linear, sqrt, log10, log2, log, proportion, or ordinal.")
                }
                opts.axis_title = t[4].map(|e| e.to_string());
            }
            _ => bail!("Too many tokens supplied to opts."),
        }

        Ok(opts)
    }
}

impl fmt::Display for Opts {
    /// Implement [`Display`] for [`Opts`] so we can
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
}

impl Report {
    /// Constructor function for [`Record`].
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
        report.search = utils::parse_comma_separated(&search);

        // safe to unwrap, as default is defined.
        report.rank = matches
            .get_one::<String>("rank")
            .expect("cli default = species")
            .to_string();
        // taxon type will be by default tax_tree(). change this here
        // for future reference. But will require a flag on the cli.

        // the x string will be just a variable.
        let x_variable = matches.get_one::<String>("x-variable");

        if let Some(xvar) = x_variable {
            let inner_x =
                Variables::new(xvar).parse_one(&variable_data::GOAT_TAXON_VARIABLE_DATA)?;
            // assign to struct
            report.x = Some(inner_x);
        };

        // parse size
        let size = *matches.get_one::<usize>("size").expect("cli default = 10");
        report.size = Some(size);

        // descendents (default) or not?
        let no_descendents = *matches
            .get_one::<bool>("no-descendents")
            .expect("cli default false");
        if no_descendents {
            report.taxon_type = TaxType::Name;
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
            // check this variable against the various lists
            let parsed_taxon_rank = TaxRanks::parse(&TaxRanks::init(), cat, true);
            let parsed_category =
                Variables::new(cat).parse_one(&variable_data::GOAT_TAXON_VARIABLE_DATA);

            match parsed_taxon_rank {
                Ok(tr) => {
                    report.category = Some(tr);
                    // kinda janky but works for now.
                    return Ok(report);
                }
                Err(e) => {
                    // propagate this error through
                    match parsed_category {
                        Ok(pc) => {
                            report.category = Some(pc);
                            return Ok(report);
                        }
                        Err(e2) => {
                            // TODO: this is quite a brutally large error, but can't think
                            // right now how to make it nicer.
                            bail!("In category: {}\n\nOr taxon rank: {}.", e2, e);
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
                url += "&includeEstimates=true";
                url += &format!("&taxonomy={}", &*TAXONOMY);
                // fix this for now, as only single requests can be submitted
                url += &format!("&queryId=goat_cli_{}", unique_ids[0]);
                Ok(url)
            }
            ReportType::Histogram => {
                let mut url = String::new();
                // add base URL
                url += &GOAT_URL;
                // it's a taxon report
                url += "report?result=taxon";
                // default stuff for now
                // no estimates
                url += "&includeEstimates=false";
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
                url += &format!("&x={}%28{}%29%20AND%20{}", taxon_type, taxa, variable);

                // add x options if any
                if let Some(xopts) = &self.x_opts {
                    url += &format!("&xOpts={}", xopts);
                }

                Ok(url)
            }
            ReportType::CategoricalHistogram => {
                let mut url = String::new();
                url += &GOAT_URL;
                // it's a taxon report
                url += "report?result=taxon";
                // default stuff for now
                // no estimates
                url += "&includeEstimates=false";
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
            ReportType::Scatterplot => todo!(),
        }
    }
}
