use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest;
use reqwest::header::ACCEPT;

use crate::record::record::{Record, RecordType};

// TODO: maybe newick can accept multiple taxa for separate newicks?
// e.g. goat newick -f "taxon 1, taxon2" "taxon3" ?

/// CLI entry point to get the Newick file from the GoaT API.
pub async fn get_newick(matches: &clap::ArgMatches) -> Result<()> {
    let record = Record::new(matches)?;
    let newick_url = record.make_url(RecordType::Newick);

    let print_url = matches.is_present("url");
    if print_url {
        println!("GoaT lookup API URL:\t{}", newick_url);
        std::process::exit(0);
    }

    // for now, you can only submit a single request at once.
    let concurrent_requests = 1;

    // but for future work, might be useful to have concurrent requests
    // for now this is a bit of extra work for a single request.
    // but whatever!
    let url_vector_api = vec![newick_url];

    let fetches = futures::stream::iter(url_vector_api.into_iter().map(|path| async move {
        // possibly make a again::RetryPolicy
        // to catch all the values in a *very* large request.
        let client = reqwest::Client::new();

        match again::retry(|| client.get(&path).header(ACCEPT, "text/x-nh").send()).await {
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

    let newick = &awaited_fetches[0];

    match newick {
        Ok(s) => print!("{}", s),
        Err(e) => bail!("{}", e),
    }

    Ok(())
}
