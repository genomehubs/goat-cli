use clap::Parser;
use futures::try_join;
use std::process::ExitCode;

use goat_cli::cli::{
    write_completions, AssemblyCommand, Cli, Index, SearchRequest, TaxonCommand,
};
use goat_cli::error::Result;
use goat_cli::report::fetch::fetch_report;
use goat_cli::report::report::{ReportOptions, ReportType};
use goat_cli::utils::field_registry;
use goat_cli::utils::utils::{
    generate_one_unique_id, generate_unique_ids, print_variables, taxa_from_input,
};
use goat_cli::{count, lookup, progress, search, IndexType};

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
    match Cli::parse().index {
        Index::Taxon { command } => match command {
            TaxonCommand::Search(args) => run_search(&SearchRequest::from(&args)).await,
            TaxonCommand::Count(args) => run_count(&SearchRequest::from(&args)).await,
            TaxonCommand::Lookup(args) => lookup::lookup(&args, true, IndexType::Taxon)
                .await
                .map(|_| ()),
            TaxonCommand::Sources(args) => {
                run_report(&ReportOptions::from(&args), ReportType::Sources).await
            }
            TaxonCommand::Newick(args) => {
                let options = ReportOptions::from(&args);
                if args.progress_bar && !args.url {
                    let ids = vec![generate_one_unique_id()];
                    try_join!(
                        fetch_report(&options, ids.clone(), ReportType::Newick),
                        progress::progress_bar(None, ids)
                    )?;
                    Ok(())
                } else {
                    run_report(&options, ReportType::Newick).await
                }
            }
            TaxonCommand::Hist(args) => {
                run_report(&ReportOptions::from(&args), ReportType::Histogram).await
            }
            TaxonCommand::Scatter(args) => {
                run_report(&ReportOptions::from(&args), ReportType::Scatterplot).await
            }
            TaxonCommand::Arc(args) => {
                let filters = [Some(args.x_filter.as_str()), args.y_filter.as_deref()];
                let filters = filters.into_iter().flatten().collect::<Vec<_>>();
                field_registry::prepare(&filters, None, IndexType::Taxon).await;
                run_report(&ReportOptions::from(&args), ReportType::Arc).await
            }
        },
        Index::Completions { shell } => {
            write_completions(shell, &mut std::io::stdout().lock())?;
            Ok(())
        }
        Index::Assembly { command } => match command {
            AssemblyCommand::Search(args) => run_search(&SearchRequest::from(&args)).await,
            AssemblyCommand::Count(args) => run_count(&SearchRequest::from(&args)).await,
            AssemblyCommand::Lookup(args) => lookup::lookup(&args, true, IndexType::Assembly)
                .await
                .map(|_| ()),
        },
    }
}

/// Common set up for `search` and `count`: handle `--print-expression`, load
/// the live field registry if `-e`/`-v` need it, and make a query ID per
/// taxon. Returns `None` if there is nothing more to do.
async fn prepare_search(request: &SearchRequest) -> Result<Option<Vec<String>>> {
    if request.output.print_expression {
        print_variables(request.index_type)?;
        return Ok(None);
    }
    let query = &request.query;
    let expressions = query.expression.as_deref().into_iter().collect::<Vec<_>>();
    field_registry::prepare(&expressions, query.variables.as_deref(), request.index_type).await;

    let taxa = taxa_from_input(
        query.taxon.as_deref(),
        query.file.as_deref(),
        query.expression.is_some(),
    )?;
    Ok(Some(generate_unique_ids(taxa.len())))
}

async fn run_search(request: &SearchRequest) -> Result<()> {
    let Some(unique_ids) = prepare_search(request).await? else {
        return Ok(());
    };
    if request.output.progress_bar {
        try_join!(
            search::search(request, unique_ids.clone()),
            progress::progress_bar(Some(request), unique_ids)
        )?;
    } else {
        search::search(request, unique_ids).await?;
    }
    Ok(())
}

async fn run_count(request: &SearchRequest) -> Result<()> {
    let Some(unique_ids) = prepare_search(request).await? else {
        return Ok(());
    };
    count::count(request, true, false, unique_ids).await?;
    Ok(())
}

/// Reports make a single request, so need a single query ID.
async fn run_report(options: &ReportOptions, report_type: ReportType) -> Result<()> {
    fetch_report(options, vec![generate_one_unique_id()], report_type).await?;
    Ok(())
}
