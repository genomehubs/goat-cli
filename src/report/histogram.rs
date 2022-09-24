use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;

use crate::report::report::{Report, ReportType};

pub async fn get_histogram(matches: &clap::ArgMatches, unique_ids: Vec<String>) -> Result<()> {
    let record = Report::new(matches, ReportType::Histogram)?;
    let histogram_url = record.make_url(unique_ids);

    let print_url = matches.is_present("url");
    if print_url {
        println!("GoaT lookup API URL:\t{}", histogram_url);
        std::process::exit(0);
    }

    // for now, you can only submit a single request at once.
    let concurrent_requests = 1;

    // but for future work, might be useful to have concurrent requests
    // for now this is a bit of extra work for a single request.
    // but whatever!
    let url_vector_api = vec![histogram_url];

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| {
            client
                .get(&path)
                .header(ACCEPT, "text/tab-separated-values")
                .send()
        })
        .await
        {
            Ok(resp) => match resp.text().await {
                Ok(body) => Ok(body),
                Err(_) => bail!("ERROR reading {}", path),
            },
            Err(_) => bail!("ERROR downloading {}", path),
        }
    }))
    .buffered(concurrent_requests)
    .collect::<Vec<_>>();

    let awaited_fetches = fetches.await;

    let histogram = &awaited_fetches[0];

    match histogram {
        Ok(s) => print!("{}", s),
        Err(e) => bail!("{}", e),
    }

    Ok(())
}
