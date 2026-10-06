//!
//! Invoked by calling:
//! `goat-cli taxon record <args>` or `goat-cli assembly record <args>`
//!
//! Fetches whole records from the GoaT `record` endpoint: every field with
//! its value and sources, and the names and lineage.

use futures::StreamExt;
use serde_json::{json, Value};

use crate::cli::RecordArgs;
use crate::client::{concurrency, GoatClient};
use crate::error::{Error, ErrorKind, Result};
use crate::output::{Format, Table};
use crate::utils::url::percent_encode_query_value;
use crate::utils::utils::taxa_from_input;
use crate::{IndexType, GOAT_URL, TAXONOMY};

/// Which part of a record to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordView {
    /// One row per field (the default).
    Fields,
    /// Names (taxon) or identifiers (assembly).
    Names,
    /// The lineage, from the record up to the root.
    Lineage,
}

impl RecordView {
    /// The view chosen by `args` (clap ensures `--names` and `--lineage`
    /// aren't both given).
    pub fn from_args(args: &RecordArgs) -> Self {
        if args.names {
            RecordView::Names
        } else if args.lineage {
            RecordView::Lineage
        } else {
            RecordView::Fields
        }
    }
}

/// Main entry point for `goat-cli <index> record`.
pub async fn record(args: &RecordArgs, index_type: IndexType) -> Result<()> {
    let inputs = taxa_from_input(args.taxon.as_deref(), args.file.as_deref(), false)?;
    let client = GoatClient::new();

    let ids = futures::stream::iter(inputs.iter().map(|input| {
        let client = client.clone();
        async move { resolve_id(&client, input, index_type).await }
    }))
    .buffered(concurrency(inputs.len()))
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

    let urls = ids.iter().map(|id| record_url(id, index_type)).collect::<Vec<_>>();
    if args.url {
        for (index, url) in urls.iter().enumerate() {
            crate::outln!("{}.\tGoaT record API URL: {}", index, url)?;
        }
        return Ok(());
    }

    let responses = futures::stream::iter(urls.iter().map(|url| {
        let client = client.clone();
        async move { client.get_json(url).await }
    }))
    .buffered(concurrency(urls.len()))
    .collect::<Vec<_>>()
    .await;

    let mut records = Vec::new();
    for (input, response) in inputs.iter().zip(responses) {
        match found_record(response?) {
            Some(record) => records.push(record),
            None => eprintln!("No record found for \"{}\".", input),
        }
    }

    match args.format {
        Format::Json => {
            crate::outln!("{}", Value::Array(records))?;
            Ok(())
        }
        format => record_table(&records, RecordView::from_args(args), index_type)?.print(format),
    }
}

/// The `record` URL for an ID.
pub fn record_url(id: &str, index_type: IndexType) -> String {
    format!(
        "{}record?recordId={}&result={}&taxonomy={}",
        *GOAT_URL,
        percent_encode_query_value(id),
        index_type,
        *TAXONOMY
    )
}

/// The record ID for what the user typed. Assembly records are looked up by
/// accession as given; taxon names are resolved to an NCBI taxon ID, and
/// must match exactly one taxon.
async fn resolve_id(client: &GoatClient, input: &str, index_type: IndexType) -> Result<String> {
    if index_type == IndexType::Assembly || input.chars().all(|c| c.is_ascii_digit()) {
        return Ok(input.to_string());
    }
    let url = format!(
        "{}search?query={}&result=taxon&taxonomy={}&includeEstimates=true&size=10",
        *GOAT_URL,
        percent_encode_query_value(&format!("tax_name({})", input)),
        *TAXONOMY
    );
    let response = client.get_json(&url).await?;
    let matches = response["results"]
        .as_array()
        .map(|results| results.iter().map(|r| &r["result"]).collect::<Vec<_>>())
        .unwrap_or_default();
    match matches.as_slice() {
        [only] => only["taxon_id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::new(ErrorKind::Api(format!("no taxon_id for \"{}\"", input)))),
        [] => Err(Error::new(ErrorKind::GenericCli(format!(
            "no taxon found named \"{}\". Try `goat-cli taxon lookup -t \"{}\"`.",
            input, input
        )))),
        several => Err(Error::new(ErrorKind::GenericCli(format!(
            "\"{}\" matches several taxa; use a taxon ID instead: {}.",
            input,
            several
                .iter()
                .map(|r| format!(
                    "{} ({} {})",
                    r["taxon_id"].as_str().unwrap_or("?"),
                    r["taxon_rank"].as_str().unwrap_or("?"),
                    r["scientific_name"].as_str().unwrap_or("?")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )))),
    }
}

/// The record in a `/record` response, if one was found.
fn found_record(mut response: Value) -> Option<Value> {
    let record = response["records"].get_mut(0)?;
    if record["found"] != Value::Bool(true) {
        return None;
    }
    Some(record["record"].take())
}

/// The table of `records` for `view`.
pub fn record_table(records: &[Value], view: RecordView, index_type: IndexType) -> Result<Table> {
    let id_column = match index_type {
        IndexType::Taxon => "taxon_id",
        IndexType::Assembly => "assembly_id",
    };
    let mut table = match (view, index_type) {
        (RecordView::Fields, _) => Table::new(&[
            id_column,
            "scientific_name",
            "field",
            "value",
            "count",
            "aggregation_source",
            "aggregation_method",
            "sources",
        ]),
        (RecordView::Names, IndexType::Taxon) => {
            Table::new(&[id_column, "scientific_name", "name", "class", "source"])
        }
        (RecordView::Names, IndexType::Assembly) => {
            Table::new(&[id_column, "scientific_name", "identifier", "class"])
        }
        (RecordView::Lineage, _) => Table::new(&[
            id_column,
            "scientific_name",
            "node_depth",
            "lineage_taxon_id",
            "lineage_scientific_name",
            "lineage_rank",
        ]),
    };

    for record in records {
        let id = record[id_column].clone();
        let name = record["scientific_name"].clone();
        match (view, index_type) {
            (RecordView::Fields, _) => {
                for (field, attribute) in record["attributes"].as_object().into_iter().flatten() {
                    table.push(vec![
                        id.clone(),
                        name.clone(),
                        json!(field),
                        attribute["value"].clone(),
                        attribute["count"].clone(),
                        attribute["aggregation_source"].clone(),
                        attribute["aggregation_method"].clone(),
                        json!(sources(attribute)),
                    ]);
                }
            }
            (RecordView::Names, IndexType::Taxon) => {
                for entry in record["taxon_names"].as_array().into_iter().flatten() {
                    table.push(vec![
                        id.clone(),
                        name.clone(),
                        entry["name"].clone(),
                        entry["class"].clone(),
                        entry["source"].clone(),
                    ]);
                }
            }
            (RecordView::Names, IndexType::Assembly) => {
                for entry in record["identifiers"].as_array().into_iter().flatten() {
                    table.push(vec![
                        id.clone(),
                        name.clone(),
                        entry["identifier"].clone(),
                        entry["class"].clone(),
                    ]);
                }
            }
            (RecordView::Lineage, _) => {
                for node in record["lineage"].as_array().into_iter().flatten() {
                    table.push(vec![
                        id.clone(),
                        name.clone(),
                        node["node_depth"].clone(),
                        node["taxon_id"].clone(),
                        node["scientific_name"].clone(),
                        node["taxon_rank"].clone(),
                    ]);
                }
            }
        }
    }
    Ok(table)
}

/// The distinct sources of a field's values: taxon fields list their raw
/// values, each with a source; assembly fields have a single source.
fn sources(attribute: &Value) -> Vec<String> {
    let mut sources: Vec<String> = Vec::new();
    match attribute["values"].as_array() {
        Some(values) => {
            for source in values.iter().filter_map(|v| v["source"].as_str()) {
                if !sources.iter().any(|s| s == source) {
                    sources.push(source.to_string());
                }
            }
        }
        None => sources.extend(attribute["source"].as_str().map(String::from)),
    }
    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taxon_record() -> Value {
        json!({
            "taxon_id": "9606",
            "scientific_name": "Homo sapiens",
            "attributes": {
                "genome_size": {
                    "value": 3423000000_u64, "count": 2, "aggregation_source": "direct", "aggregation_method": "primary",
                    "values": [{"source": "AGSD"}, {"source": "AGSD"}, {"source": "Other"}]
                },
                "assembly_level": {"value": "complete genome", "count": 1, "aggregation_source": "direct", "aggregation_method": "enum", "values": []}
            },
            "taxon_names": [{"name": "human", "class": "genbank common name", "source": "NCBI Taxonomy"}],
            "lineage": [{"node_depth": 1, "taxon_id": "9605", "scientific_name": "Homo", "taxon_rank": "genus"}]
        })
    }

    #[test]
    fn test_fields_table_sorted_with_distinct_sources() {
        let table = record_table(&[taxon_record()], RecordView::Fields, IndexType::Taxon).unwrap();
        assert_eq!(table.header[0], "taxon_id");
        // fields are in alphabetical order
        assert_eq!(table.rows[0][2], json!("assembly_level"));
        assert_eq!(table.rows[1], vec![
            json!("9606"), json!("Homo sapiens"), json!("genome_size"), json!(3423000000_u64),
            json!(2), json!("direct"), json!("primary"), json!(["AGSD", "Other"]),
        ]);
    }

    #[test]
    fn test_names_and_lineage_tables() {
        let names = record_table(&[taxon_record()], RecordView::Names, IndexType::Taxon).unwrap();
        assert_eq!(names.rows, vec![vec![json!("9606"), json!("Homo sapiens"), json!("human"), json!("genbank common name"), json!("NCBI Taxonomy")]]);
        let lineage = record_table(&[taxon_record()], RecordView::Lineage, IndexType::Taxon).unwrap();
        assert_eq!(lineage.rows, vec![vec![json!("9606"), json!("Homo sapiens"), json!(1), json!("9605"), json!("Homo"), json!("genus")]]);
    }

    #[test]
    fn test_assembly_fields_use_single_source() {
        let record = json!({
            "assembly_id": "GCA_000001405.29", "scientific_name": "Homo sapiens",
            "attributes": {"assembly_span": {"value": 3099734149_u64, "count": 1, "source": "INSDC"}},
            "identifiers": [{"identifier": "GCF_000001405.40", "class": "refseq_accession"}]
        });
        let fields = record_table(&[record.clone()], RecordView::Fields, IndexType::Assembly).unwrap();
        assert_eq!(fields.header[0], "assembly_id");
        assert_eq!(fields.rows[0][7], json!(["INSDC"]));
        let names = record_table(&[record], RecordView::Names, IndexType::Assembly).unwrap();
        assert_eq!(names.rows[0][2], json!("GCF_000001405.40"));
    }

    #[test]
    fn test_found_record() {
        assert_eq!(found_record(json!({"records": [{"found": true, "record": {"taxon_id": "1"}}]})), Some(json!({"taxon_id": "1"})));
        assert_eq!(found_record(json!({"records": [{"found": false}]})), None);
        assert_eq!(found_record(json!({"records": []})), None);
    }

    #[test]
    fn test_record_url() {
        assert_eq!(
            record_url("GCA_000001405.29", IndexType::Assembly),
            format!("{}record?recordId=GCA_000001405.29&result=assembly&taxonomy=ncbi", *GOAT_URL)
        );
    }
}
