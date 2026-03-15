use crate::client::GoatClient;
use crate::error::{Error, ErrorKind, Result};
use crate::report::report::{Report, ReportType};
use futures::StreamExt;
use std::io::Write;

pub enum ReportAction {
    Continue,
    PrintedAndExit,
}

/// CLI entry point to get the Newick file from the GoaT API.
pub async fn fetch_report(
    matches: &clap::ArgMatches,
    unique_ids: Vec<String>,
    report_type: ReportType,
) -> Result<ReportAction> {
    let report = Report::new(matches, report_type)?;
    let url = report.make_url(unique_ids)?;

    let print_url = *matches.get_one::<bool>("url").expect("cli default false");
    if print_url {
        println!("GoaT lookup API URL:\t{}", url);
        return Ok(ReportAction::PrintedAndExit);
    }

    // otherwise we get schema errors.
    // more schemas could be defined here.
    let header_value = match report_type {
        ReportType::Newick => "text/x-nh",
        _ => "text/tab-separated-values",
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
                return Err(Error::new(ErrorKind::Report(
                    "no data found. If it was a `taxon newick` call, try increasing the threshold."
                        .to_string(),
                )));
            }

            let mut stdout = std::io::stdout();
            writeln!(stdout, "{}", s)?;
        }
        Err(e) => return Err(e),
    }

    Ok(ReportAction::Continue)
}
