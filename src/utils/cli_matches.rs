use crate::cli::SearchRequest;
use crate::error::{Error, ErrorKind, Result};
use crate::utils::{tax_ranks, url, utils};
use crate::{GOAT_URL, TAXONOMY, UPPER_CLI_SIZE_LIMIT};

pub enum CliAction {
    Continue {
        size: u64,
        taxa: Vec<String>,
        urls: Vec<String>,
    },
    PrintedAndExit,
}

/// Build the GoaT API URLs for a `search` or `count` (`api`), one per taxon.
///
/// If `-u`/`-U` were given, prints the URLs and returns
/// [`CliAction::PrintedAndExit`]; otherwise returns the size, the taxa and
/// their URLs.
pub fn process_cli_args(
    request: &SearchRequest,
    api: &str,
    unique_ids: Vec<String>,
) -> Result<CliAction> {
    let query = &request.query;
    let index_type = request.index_type;

    let expression = match &query.expression {
        Some(s) => url::format_expression(s, index_type)?,
        None => "".to_string(),
    };
    let tax_rank = match &query.tax_rank {
        Some(t) => tax_ranks::TaxRanks::init().parse(t, false)?,
        None => "".to_string(),
    };

    if query.size as usize > *UPPER_CLI_SIZE_LIMIT {
        let limit_string = utils::pretty_print_usize(*UPPER_CLI_SIZE_LIMIT);
        return Err(Error::new(ErrorKind::GenericCli(format!(
            "searches with more than {} results are not currently supported.",
            limit_string
        ))));
    }

    // tree includes all descendents of a node; clap ensures -d and -l
    // aren't both given.
    let tax_tree = if query.descendents {
        "tree"
    } else if query.lineage {
        "lineage"
    } else {
        "name"
    };

    let url_vector = utils::taxa_from_input(
        query.taxon.as_deref(),
        query.file.as_deref(),
        query.expression.is_some(),
    )?;

    let url_vector_api = url::make_goat_urls(
        api,
        &url_vector,
        &GOAT_URL,
        tax_tree,
        request.include_estimates,
        request.include_raw_values,
        query.exclude,
        "count",
        &index_type.to_string(),
        &TAXONOMY,
        query.size,
        &query.ranks,
        request.fields,
        query.variables.as_deref(),
        &expression,
        &tax_rank,
        unique_ids,
        index_type,
    )?;

    if request.output.url {
        for (index, url) in url_vector_api.iter().enumerate() {
            crate::outln!("{}.\tGoaT API URL: {}", index, url)?;
        }
        return Ok(CliAction::PrintedAndExit);
    } else if request.output.goat_ui_url {
        for (index, url) in url_vector_api.iter().enumerate() {
            let new_url = url.replace("api/v2/", "");
            crate::outln!("{}.\tGoaT UI URL: {}", index, new_url)?;
        }
        return Ok(CliAction::PrintedAndExit);
    }

    Ok(CliAction::Continue {
        size: query.size,
        taxa: url_vector,
        urls: url_vector_api,
    })
}
