use futures::try_join;
use std::process::ExitCode;
use goat_cli::error::Result;
use goat_cli::utils::args::ArgMatchesExt;

use goat_cli::report::fetch::ReportAction;
use goat_cli::{
    cli, count, lookup, progress,
    report::{self, report::ReportType},
    search,
    utils::{field_registry, utils::{generate_one_unique_id, generate_unique_strings, UniqueIdAction}},
    IndexType,
};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        // stdout was closed early (e.g. piped into `head`), which is fine.
        Err(e) if e.is_broken_pipe() => ExitCode::SUCCESS,
        // format the errors nicely
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    // If -e or -v name a field this binary doesn't know, load the live GoaT
    // field list (cached on disk) so newly added fields are still accepted.
    if let Some((index, index_matches)) = matches.subcommand() {
        if let Some(("search" | "count" | "arc", leaf)) = index_matches.subcommand() {
            let index_type = match index {
                "assembly" => IndexType::Assembly,
                _ => IndexType::Taxon,
            };
            field_registry::prepare(leaf, index_type).await;
        }
    }

    // nested matching on subcommands
    match matches.subcommand() {
        // outer == taxon/assembly
        Some(("taxon", taxon_matches)) => match taxon_matches.subcommand() {
            // inner are all the taxon matches here.
            Some(("search", taxon_search_matches)) => {
                let progress_bar = *taxon_search_matches
                    .opt_one::<bool>("progress-bar")
                    .expect("cli default false");
                let unique_ids =
                    match generate_unique_strings(taxon_search_matches, IndexType::Taxon)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(
                                taxon_search_matches,
                                unique_ids.clone(),
                                IndexType::Taxon
                            ),
                            progress::progress_bar(
                                taxon_search_matches,
                                "search",
                                unique_ids,
                                IndexType::Taxon
                            )
                        )?;
                    }
                    false => {
                        search::search(taxon_search_matches, unique_ids, IndexType::Taxon).await?;
                    }
                }
            }
            Some(("sources", taxon_sources_matches)) => {
                let unique_ids =
                    match generate_unique_strings(taxon_sources_matches, IndexType::Taxon)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                match report::fetch::fetch_report(
                    taxon_sources_matches,
                    unique_ids,
                    ReportType::Sources,
                )
                .await?
                {
                    ReportAction::Continue => {}
                    ReportAction::PrintedAndExit => return Ok(()),
                };
            }
            Some(("count", taxon_count_matches)) => {
                let unique_ids =
                    match generate_unique_strings(taxon_count_matches, IndexType::Taxon)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                count::count(
                    taxon_count_matches,
                    true,
                    false,
                    unique_ids,
                    IndexType::Taxon,
                )
                .await?;
            }
            Some(("lookup", taxon_lookup_matches)) => {
                match lookup::lookup(taxon_lookup_matches, true, IndexType::Taxon).await? {
                    lookup::LookupAction::Continue => {}
                    lookup::LookupAction::PrintedAndExit => return Ok(()),
                }
            }
            Some(("hist", taxon_hist_matches)) => {
                let unique_ids =
                    match generate_unique_strings(taxon_hist_matches, IndexType::Taxon)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                match report::fetch::fetch_report(
                    taxon_hist_matches,
                    unique_ids,
                    ReportType::Histogram,
                )
                .await?
                {
                    ReportAction::Continue => {}
                    ReportAction::PrintedAndExit => return Ok(()),
                };
            }
            Some(("scatter", scatter_matches)) => {
                let unique_ids = match generate_unique_strings(scatter_matches, IndexType::Taxon)? {
                    UniqueIdAction::Continue(ids) => ids,
                    UniqueIdAction::PrintedAndExit => return Ok(()),
                };

                match report::fetch::fetch_report(
                    scatter_matches,
                    unique_ids,
                    ReportType::Scatterplot,
                )
                .await?
                {
                    ReportAction::Continue => {}
                    ReportAction::PrintedAndExit => return Ok(()),
                };
            }
            Some(("arc", arc_matches)) => {
                // Arc may have no taxon (global query), so only call generate_unique_strings
                // when a taxon or file is present; otherwise generate a single ID.
                let unique_ids =
                    if arc_matches.opt_one::<String>("taxon").is_some()
                        || arc_matches.opt_one::<std::path::PathBuf>("file").is_some()
                    {
                        match generate_unique_strings(arc_matches, IndexType::Taxon)? {
                            UniqueIdAction::Continue(ids) => ids,
                            UniqueIdAction::PrintedAndExit => return Ok(()),
                        }
                    } else {
                        vec![generate_one_unique_id()]
                    };

                match report::fetch::fetch_report(arc_matches, unique_ids, ReportType::Arc).await? {
                    ReportAction::Continue => {}
                    ReportAction::PrintedAndExit => return Ok(()),
                };
            }
            Some(("newick", taxon_newick_matches)) => {
                let progress_bar = *taxon_newick_matches
                    .opt_one::<bool>("progress-bar")
                    .expect("cli default false");
                // TODO: check that the CLI has a 'url' option
                let print_url = taxon_newick_matches
                    .opt_one::<bool>("url")
                    .copied()
                    .unwrap_or(false);

                let unique_ids =
                    match generate_unique_strings(taxon_newick_matches, IndexType::Taxon)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                if print_url {
                    match report::fetch::fetch_report(
                        taxon_newick_matches,
                        unique_ids,
                        ReportType::Newick,
                    )
                    .await?
                    {
                        ReportAction::Continue => {}
                        ReportAction::PrintedAndExit => return Ok(()),
                    }
                } else if progress_bar {
                    let (report_action, _) = try_join!(
                        report::fetch::fetch_report(
                            taxon_newick_matches,
                            unique_ids.clone(),
                            ReportType::Newick
                        ),
                        progress::progress_bar(
                            taxon_newick_matches,
                            "newick",
                            unique_ids,
                            IndexType::Taxon
                        )
                    )?;

                    match report_action {
                        ReportAction::Continue => {}
                        ReportAction::PrintedAndExit => return Ok(()),
                    }
                } else {
                    match report::fetch::fetch_report(
                        taxon_newick_matches,
                        unique_ids,
                        ReportType::Newick,
                    )
                    .await?
                    {
                        ReportAction::Continue => {}
                        ReportAction::PrintedAndExit => return Ok(()),
                    }
                }
            }
            _ => {
                unreachable!()
            }
        },
        // and now assembly
        Some(("assembly", assembly_matches)) => match assembly_matches.subcommand() {
            // and the three implemented subcommands currently.
            Some(("search", assembly_search_matches)) => {
                let progress_bar = *assembly_search_matches
                    .opt_one::<bool>("progress-bar")
                    .expect("cli default false");
                let unique_ids =
                    match generate_unique_strings(assembly_search_matches, IndexType::Assembly)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                match progress_bar {
                    true => {
                        try_join!(
                            search::search(
                                assembly_search_matches,
                                unique_ids.clone(),
                                IndexType::Assembly
                            ),
                            progress::progress_bar(
                                assembly_search_matches,
                                "search",
                                unique_ids,
                                IndexType::Assembly
                            )
                        )?;
                    }
                    false => {
                        search::search(assembly_search_matches, unique_ids, IndexType::Assembly)
                            .await?;
                    }
                }
            }
            Some(("count", assembly_count_matches)) => {
                let unique_ids =
                    match generate_unique_strings(assembly_count_matches, IndexType::Assembly)? {
                        UniqueIdAction::Continue(ids) => ids,
                        UniqueIdAction::PrintedAndExit => return Ok(()),
                    };

                count::count(
                    assembly_count_matches,
                    true,
                    false,
                    unique_ids,
                    IndexType::Assembly,
                )
                .await?;
            }
            Some(("lookup", assembly_lookup_matches)) => {
                match lookup::lookup(assembly_lookup_matches, true, IndexType::Assembly).await? {
                    lookup::LookupAction::Continue => {}
                    lookup::LookupAction::PrintedAndExit => return Ok(()),
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(())
}
