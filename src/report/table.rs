//! Turn GoaT report responses (JSON) into tables, for `--format tsv|csv`.
//!
//! Each function takes the whole `/report` response.

use serde_json::{json, Value};

use crate::error::{Error, ErrorKind, Result};
use crate::output::{epoch_millis_to_date, Table};
use crate::report::report::ReportType;

fn unexpected(what: &str) -> Error {
    Error::new(ErrorKind::Report(format!(
        "unexpected report response: {what} missing (please report if you get this error!)"
    )))
}

/// The report body, e.g. `response.report.report.histogram`.
fn body<'a>(response: &'a Value, name: &str) -> Result<&'a Value> {
    let body = &response["report"]["report"][name];
    if body.is_null() {
        return Err(unexpected(name));
    }
    Ok(body)
}

fn array<'a>(value: &'a Value, what: &str) -> Result<&'a Vec<Value>> {
    value.as_array().ok_or_else(|| unexpected(what))
}

/// Show a bucket edge: dates come as milliseconds since the epoch.
fn edge(value: &Value, value_type: &str) -> Value {
    match (value_type, value.as_i64()) {
        ("date", Some(millis)) => json!(epoch_millis_to_date(millis)),
        _ => value.clone(),
    }
}

/// One axis of a histogram or scatter.
///
/// Numeric and date axes have bucket edges, so bin `i` is
/// `[buckets[i], buckets[i + 1])` and the last count (past the last edge)
/// has no bin. Keyword axes have one bucket per value, so bin `i` is just
/// `buckets[i]`, with a trailing `null` bucket.
struct Axis<'a> {
    field: &'a str,
    value_type: &'a str,
    buckets: &'a [Value],
    keyword: bool,
}

impl<'a> Axis<'a> {
    fn new(field: &'a str, value_type: &'a str, buckets: &'a [Value]) -> Self {
        let keyword = value_type == "keyword" || buckets.iter().any(Value::is_string);
        Self {
            field,
            value_type,
            buckets,
            keyword,
        }
    }

    /// The column names: `<field>` for keywords, else `<field>_from` and
    /// `<field>_to`.
    fn columns(&self) -> Vec<String> {
        if self.keyword {
            vec![self.field.to_string()]
        } else {
            vec![format!("{}_from", self.field), format!("{}_to", self.field)]
        }
    }

    /// Whether count `i` falls in a real bin (not the open-ended last
    /// count of a numeric axis, or the `null` bucket of a keyword axis).
    fn has_bin(&self, i: usize) -> bool {
        if self.keyword {
            self.buckets.get(i).map_or(false, |b| !b.is_null())
        } else {
            i + 1 < self.buckets.len()
        }
    }

    /// The cells for count `i`; open-ended bins have a null edge.
    fn cells(&self, i: usize) -> Vec<Value> {
        let edge_at = |j: usize| self.buckets.get(j).map_or(Value::Null, |b| edge(b, self.value_type));
        if self.keyword {
            vec![self.buckets.get(i).cloned().unwrap_or(Value::Null)]
        } else {
            vec![edge_at(i), edge_at(i + 1)]
        }
    }

    /// Error if there are more counts than buckets, which GoaT doesn't send.
    fn check(&self, counts: usize, what: &str) -> Result<()> {
        if counts > self.buckets.len() {
            return Err(unexpected(&format!("{what} (more counts than buckets)")));
        }
        Ok(())
    }
}

/// Counts outside every bin are always zero; skip them, but keep any that
/// aren't so no values are lost.
fn skip(count: &Value, in_bins: bool) -> bool {
    !in_bins && count.as_u64() == Some(0)
}

/// The table for a report, other than a newick tree.
pub fn report_table(report_type: ReportType, response: &Value) -> Result<Table> {
    match report_type {
        ReportType::Histogram => histogram_table(response),
        ReportType::Scatterplot => scatter_table(response),
        ReportType::Arc => arc_table(response),
        ReportType::Sources => sources_table(response),
        ReportType::Newick | ReportType::None => Err(Error::new(ErrorKind::Report(
            "this report has no table format".to_string(),
        ))),
    }
}

/// One row per bin, with the count, then a count per category (`-c`).
///
/// Numeric and date bins are `[<field>_from, <field>_to)`; keyword
/// histograms have one row per value.
pub fn histogram_table(response: &Value) -> Result<Table> {
    let histogram = body(response, "histogram")?;
    let field = histogram["field"].as_str().unwrap_or("value");
    let bins = &histogram["histograms"];
    let buckets = array(&bins["buckets"], "histograms.buckets")?;
    let counts = array(&bins["allValues"], "histograms.allValues")?;
    let axis = Axis::new(field, bins["valueType"].as_str().unwrap_or(""), buckets);
    axis.check(counts.len(), "histograms.allValues")?;
    let categories = histogram["cats"]
        .as_array()
        .map(|cats| cats.iter().filter_map(|c| c["key"].as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut header = axis.columns();
    header.push("count".to_string());
    header.extend(categories.iter().map(|c| c.to_string()));
    let mut table = Table::new(&header.iter().map(String::as_str).collect::<Vec<_>>());

    for (i, count) in counts.iter().enumerate() {
        if skip(count, axis.has_bin(i)) {
            continue;
        }
        let mut row = axis.cells(i);
        row.push(count.clone());
        for category in &categories {
            row.push(bins["byCat"][category].get(i).cloned().unwrap_or(json!(0)));
        }
        table.push(row);
    }
    Ok(table)
}

/// One row per cell of the x/y grid: the x bin, the y bin and the count.
/// Each axis is a value (keyword) or a `_from`/`_to` range, as in
/// [`histogram_table`].
pub fn scatter_table(response: &Value) -> Result<Table> {
    let scatter = body(response, "scatter")?;
    let bins = &scatter["histograms"];
    let x = Axis::new(
        bins["xLabel"].as_str().unwrap_or("x"),
        bins["valueType"].as_str().unwrap_or(""),
        array(&bins["buckets"], "histograms.buckets")?,
    );
    let y = Axis::new(
        bins["yLabel"].as_str().unwrap_or("y"),
        bins["yValueType"].as_str().unwrap_or(""),
        array(&bins["yBuckets"], "histograms.yBuckets")?,
    );
    let grid = array(&bins["allYValues"], "histograms.allYValues")?;
    x.check(grid.len(), "histograms.allYValues")?;

    let mut header = x.columns();
    header.extend(y.columns());
    header.push("count".to_string());
    let mut table = Table::new(&header.iter().map(String::as_str).collect::<Vec<_>>());
    for (i, column) in grid.iter().enumerate() {
        let column = array(column, "histograms.allYValues[]")?;
        y.check(column.len(), "histograms.allYValues[]")?;
        for (j, count) in column.iter().enumerate() {
            if skip(count, x.has_bin(i) && y.has_bin(j)) {
                continue;
            }
            let mut row = x.cells(i);
            row.extend(y.cells(j));
            row.push(count.clone());
            table.push(row);
        }
    }
    Ok(table)
}

/// A single row: the two queries, their counts and the proportion.
pub fn arc_table(response: &Value) -> Result<Table> {
    let arc = body(response, "arc")?;
    let mut table = Table::new(&["x_query", "y_query", "rank", "x", "y", "proportion"]);
    table.push(vec![
        arc["xQuery"]["query"].clone(),
        arc["yQuery"]["query"].clone(),
        arc["rank"].clone(),
        arc["x"].clone(),
        arc["y"].clone(),
        arc["arc"].clone(),
    ]);
    Ok(table)
}

/// One row per data source: how many values it provides, for which fields.
pub fn sources_table(response: &Value) -> Result<Table> {
    let sources = body(response, "sources")?
        .as_object()
        .ok_or_else(|| unexpected("sources"))?;
    let mut table = Table::new(&["source", "count", "date", "url", "attributes"]);
    for (name, source) in sources {
        table.push(vec![
            json!(name),
            source["count"].clone(),
            source["date"].clone(),
            source["url"].clone(),
            source["attributes"].clone(),
        ]);
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(name: &str, body: Value) -> Value {
        json!({"status": {"success": true}, "report": {"report": {name: body}}})
    }

    #[test]
    fn test_numeric_histogram_with_categories() {
        let response = wrap("histogram", json!({
            "field": "genome_size",
            "cats": [{"key": "scaffold"}, {"key": "chromosome"}],
            "histograms": {
                "buckets": [1, 10, 100],
                "allValues": [5, 2, 0],
                "valueType": "integer",
                "byCat": {"scaffold": [4, 1, 0], "chromosome": [1, 1, 0]}
            }
        }));
        let table = histogram_table(&response).unwrap();
        assert_eq!(table.header, ["genome_size_from", "genome_size_to", "count", "scaffold", "chromosome"]);
        assert_eq!(table.rows, vec![
            vec![json!(1), json!(10), json!(5), json!(4), json!(1)],
            vec![json!(10), json!(100), json!(2), json!(1), json!(1)],
        ]);
    }

    #[test]
    fn test_keyword_histogram() {
        let response = wrap("histogram", json!({
            "field": "assembly_level",
            "histograms": {"buckets": ["scaffold", "chromosome", null], "allValues": [199, 40], "valueType": "keyword"}
        }));
        let table = histogram_table(&response).unwrap();
        assert_eq!(table.header, ["assembly_level", "count"]);
        assert_eq!(table.rows, vec![vec![json!("scaffold"), json!(199)], vec![json!("chromosome"), json!(40)]]);
    }

    #[test]
    fn test_date_histogram_edges_are_dates() {
        let response = wrap("histogram", json!({
            "field": "assembly_date",
            "histograms": {"buckets": [1_293_840_000_000_i64, 1_325_376_000_000_i64], "allValues": [7, 0], "valueType": "date"}
        }));
        let table = histogram_table(&response).unwrap();
        assert_eq!(table.rows, vec![vec![json!("2011-01-01"), json!("2012-01-01"), json!(7)]]);
    }

    #[test]
    fn test_scatter_grid() {
        let response = wrap("scatter", json!({
            "histograms": {
                "xLabel": "genome_size", "yLabel": "c_value",
                "valueType": "integer", "yValueType": "float",
                "buckets": [1, 2, 3], "yBuckets": [0.5, 1.5],
                "allYValues": [[4, 0], [6, 0], [0, 0]]
            }
        }));
        let table = scatter_table(&response).unwrap();
        assert_eq!(table.header, ["genome_size_from", "genome_size_to", "c_value_from", "c_value_to", "count"]);
        assert_eq!(table.rows, vec![
            vec![json!(1), json!(2), json!(0.5), json!(1.5), json!(4)],
            vec![json!(2), json!(3), json!(0.5), json!(1.5), json!(6)],
        ]);
    }

    #[test]
    fn test_scatter_keyword_axis_is_one_column() {
        // as GoaT sends it: keyword buckets end with null
        let response = wrap("scatter", json!({
            "histograms": {
                "xLabel": "assembly_level", "yLabel": "assembly_span",
                "valueType": "keyword", "yValueType": "integer",
                "buckets": ["scaffold", "chromosome", null], "yBuckets": [10, 20, 30],
                "allYValues": [[1, 2, 0], [0, 3, 0]]
            }
        }));
        let table = scatter_table(&response).unwrap();
        assert_eq!(table.header, ["assembly_level", "assembly_span_from", "assembly_span_to", "count"]);
        assert_eq!(table.rows, vec![
            vec![json!("scaffold"), json!(10), json!(20), json!(1)],
            vec![json!("scaffold"), json!(20), json!(30), json!(2)],
            vec![json!("chromosome"), json!(10), json!(20), json!(0)],
            vec![json!("chromosome"), json!(20), json!(30), json!(3)],
        ]);
    }

    #[test]
    fn test_open_ended_bins_kept_if_not_zero() {
        let response = wrap("scatter", json!({
            "histograms": {
                "xLabel": "x", "yLabel": "y", "valueType": "integer", "yValueType": "integer",
                "buckets": [1, 2], "yBuckets": [5, 6],
                "allYValues": [[1, 0], [0, 7]]
            }
        }));
        let rows = scatter_table(&response).unwrap().rows;
        assert_eq!(rows, vec![
            vec![json!(1), json!(2), json!(5), json!(6), json!(1)],
            vec![json!(2), Value::Null, json!(6), Value::Null, json!(7)],
        ]);

        let response = wrap("histogram", json!({
            "field": "f", "histograms": {"buckets": [1, 10], "allValues": [5, 2], "valueType": "integer"}
        }));
        assert_eq!(histogram_table(&response).unwrap().rows, vec![
            vec![json!(1), json!(10), json!(5)],
            vec![json!(10), Value::Null, json!(2)],
        ]);
    }

    #[test]
    fn test_more_counts_than_buckets_is_an_error() {
        let response = wrap("histogram", json!({
            "field": "f", "histograms": {"buckets": [1], "allValues": [5, 2, 1], "valueType": "integer"}
        }));
        let err = histogram_table(&response).unwrap_err();
        assert!(err.to_string().contains("more counts than buckets"), "{}", err);

        let response = wrap("scatter", json!({
            "histograms": {"buckets": [1, 2], "yBuckets": [1], "allYValues": [[1, 2]]}
        }));
        assert!(scatter_table(&response).is_err());
    }

    #[test]
    fn test_arc_and_sources() {
        let arc = wrap("arc", json!({
            "arc": 0.25, "x": 1, "y": 4, "rank": "species",
            "xQuery": {"query": "tax_tree(Primates) AND assembly_span"}, "yQuery": {"query": "tax_rank(species)"}
        }));
        assert_eq!(arc_table(&arc).unwrap().rows, vec![vec![
            json!("tax_tree(Primates) AND assembly_span"), json!("tax_rank(species)"), json!("species"), json!(1), json!(4), json!(0.25),
        ]]);

        let sources = wrap("sources", json!({
            "INSDC": {"count": 7, "date": "2026-10-02", "url": "https://x", "attributes": ["assembly_span"]},
            "CNGB": {"count": 2, "attributes": ["assembly_level"]}
        }));
        let table = sources_table(&sources).unwrap();
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0][0], json!("CNGB"));
        assert_eq!(table.rows[0][2], Value::Null);
    }

    #[test]
    fn test_unexpected_shape_is_an_error() {
        let err = histogram_table(&json!({"report": {}})).unwrap_err();
        assert!(err.to_string().contains("histogram missing"), "{}", err);
    }
}
