use crate::client::GoatClient;
use crate::error::{Error, ErrorKind, Result};
use crate::output::Format;
use crate::report::report::{Report, ReportOptions, ReportType};
use crate::report::table::report_table;
use futures::StreamExt;
use std::io::Write;

pub enum ReportAction {
    Continue,
    PrintedAndExit,
}

/// CLI entry point to get the Newick file from the GoaT API.
pub async fn fetch_report(
    options: &ReportOptions,
    unique_ids: Vec<String>,
    report_type: ReportType,
) -> Result<ReportAction> {
    let report = Report::new(options, report_type)?;
    let url = report.make_url(unique_ids)?;

    if options.url {
        crate::outln!("GoaT report API URL:\t{}", url)?;
        return Ok(ReportAction::PrintedAndExit);
    }

    let header_value = match report_type {
        ReportType::Newick => "text/x-nh",
        _ => "application/json",
    };

    // for now, you can only submit a single request at once.
    let concurrent_requests = 1;

    // but for future work, might be useful to have concurrent requests
    // for now this is a bit of extra work for a single request.
    // but whatever!
    let url_vector_api = vec![url];

    let client = GoatClient::new();
    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| {
        let client = client.clone();
        async move { client.get_text(&path, header_value).await }
    }))
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let mut awaited_fetches = fetches.await;

    let report = awaited_fetches.remove(0);

    match report {
        Ok(ref s) => {
            // check the length of the string
            // if it's zero, then we have an error
            if s.is_empty() || s == ";" {
                let hint = match report_type {
                    ReportType::Newick if options.no_descendents => format!(
                        " With -n, the taxa are the tips, so they must be at the rank ({}).",
                        options.rank
                    ),
                    ReportType::Newick => " Try increasing the threshold.".to_string(),
                    _ => String::new(),
                };
                return Err(Error::new(ErrorKind::Report(format!("no data found.{}", hint))));
            }

            if report_type == ReportType::Newick || options.format == Format::Json {
                let mut stdout = std::io::stdout();
                writeln!(stdout, "{}", s)?;
            } else {
                let response: serde_json::Value = serde_json::from_str(s)?;
                report_table(report_type, &response)?.print(options.format)?;
            }
        }
        Err(e) => return Err(e),
    }

    Ok(ReportAction::Continue)
}
