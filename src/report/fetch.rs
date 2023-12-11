use crate::error::{Error, ErrorKind, Result};
use crate::report::report::{Report, ReportType};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;

/// CLI entry point to get the Newick file from the GoaT API.
pub async fn fetch_report(
    matches: &clap::ArgMatches,
    unique_ids: Vec<String>,
    report_type: ReportType,
) -> Result<()> {
    let report = Report::new(matches, report_type)?;
    let url = report.make_url(unique_ids)?;

    let print_url = *matches.get_one::<bool>("url").expect("cli default false");
    if print_url {
        println!("GoaT lookup API URL:\t{}", url);
        std::process::exit(0);
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

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(&path).header(ACCEPT, header_value).send()).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => Ok(body),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }))
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    let newick = &awaited_fetches[0];

    match newick {
        Ok(s) => print!("{}", s),
        Err(e) => return Err(Error::new(ErrorKind::Report(e.to_string()))),
    }

    Ok(())
}
