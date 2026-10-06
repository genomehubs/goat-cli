//! Output formats (`--format tsv|csv|json`) and a small table type that the
//! CLI builds itself (e.g. for `count`, `lookup`, `record` and reports) and
//! can write in any format.

use std::io::Write;

use serde_json::{Map, Value};

use crate::error::Result;

/// The output format, chosen with `--format`.
//
// No doc comments on the variants: clap would show them as help for each
// value, which switches every command's `--help` to its long layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    // Tab separated values.
    #[default]
    Tsv,
    // Comma separated values.
    Csv,
    // JSON.
    Json,
}

impl Format {
    /// The `Accept` header to request this format from GoaT's `/search`.
    pub fn accept(self) -> &'static str {
        match self {
            Format::Tsv => "text/tab-separated-values",
            Format::Csv => "text/csv",
            Format::Json => "application/json",
        }
    }
}

/// A table of values, written as TSV, CSV, or a JSON array of objects.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table {
    /// The column names.
    pub header: Vec<String>,
    /// The rows; each has one cell per column.
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    /// A table with the given column names and no rows.
    pub fn new(header: &[&str]) -> Self {
        Self {
            header: header.iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    /// Add a row. Panics if it has the wrong number of cells, which is a
    /// programming error.
    pub fn push(&mut self, row: Vec<Value>) {
        assert_eq!(row.len(), self.header.len(), "row length must match header");
        self.rows.push(row);
    }

    /// Write the table to `out` in `format`.
    pub fn write(&self, format: Format, out: &mut impl Write) -> Result<()> {
        match format {
            Format::Tsv | Format::Csv => {
                let line = |cells: Vec<String>| -> String {
                    match format {
                        Format::Csv => cells.iter().map(|c| csv_escape(c)).collect::<Vec<_>>().join(","),
                        _ => cells.iter().map(|c| tsv_escape(c)).collect::<Vec<_>>().join("\t"),
                    }
                };
                writeln!(out, "{}", line(self.header.clone()))?;
                for row in &self.rows {
                    writeln!(out, "{}", line(row.iter().map(cell_text).collect()))?;
                }
            }
            Format::Json => {
                let objects = self
                    .rows
                    .iter()
                    .map(|row| {
                        Value::Object(
                            self.header.iter().cloned().zip(row.iter().cloned()).collect::<Map<_, _>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                writeln!(out, "{}", serde_json::to_string_pretty(&Value::Array(objects))?)?;
            }
        }
        Ok(())
    }

    /// Write the table to stdout.
    pub fn print(&self, format: Format) -> Result<()> {
        let mut out = std::io::BufWriter::new(std::io::stdout().lock());
        self.write(format, &mut out)?;
        out.flush()?;
        Ok(())
    }
}

/// How a cell is shown in TSV/CSV: strings as is, lists joined with `;`
/// (as GoaT does), and null as empty.
pub fn cell_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(values) => values.iter().map(cell_text).collect::<Vec<_>>().join(";"),
        other => other.to_string(),
    }
}

/// TSV has no quoting, so tabs and newlines in a value become spaces.
fn tsv_escape(text: &str) -> String {
    text.replace(['\t', '\n', '\r'], " ")
}

/// Quote a CSV field if needed (RFC 4180).
fn csv_escape(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

/// Format milliseconds since the Unix epoch (as GoaT uses for dates in
/// reports) as `yyyy-mm-dd`.
pub fn epoch_millis_to_date(millis: i64) -> String {
    // days since 1970-01-01, then Howard Hinnant's civil_from_days
    let z = millis.div_euclid(86_400_000) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Table {
        let mut table = Table::new(&["name", "count", "tags"]);
        table.push(vec![json!("Homo sapiens"), json!(3), json!(["a", "b"])]);
        table.push(vec![json!("comma, \"quote\""), Value::Null, json!([])]);
        table
    }

    fn render(table: &Table, format: Format) -> String {
        let mut out = Vec::new();
        table.write(format, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn test_tsv() {
        assert_eq!(
            render(&sample(), Format::Tsv),
            "name\tcount\ttags\nHomo sapiens\t3\ta;b\ncomma, \"quote\"\t\t\n"
        );
    }

    #[test]
    fn test_csv_quotes_when_needed() {
        assert_eq!(
            render(&sample(), Format::Csv),
            "name,count,tags\nHomo sapiens,3,a;b\n\"comma, \"\"quote\"\"\",,\n"
        );
    }

    #[test]
    fn test_json_objects_keep_types() {
        let parsed: Value = serde_json::from_str(&render(&sample(), Format::Json)).unwrap();
        assert_eq!(parsed[0], json!({"name": "Homo sapiens", "count": 3, "tags": ["a", "b"]}));
        assert_eq!(parsed[1]["count"], Value::Null);
    }

    #[test]
    fn test_tsv_replaces_tabs_and_newlines() {
        let mut table = Table::new(&["x"]);
        table.push(vec![json!("a\tb\nc")]);
        assert_eq!(render(&table, Format::Tsv), "x\na b c\n");
    }

    #[test]
    fn test_epoch_millis_to_date() {
        assert_eq!(epoch_millis_to_date(0), "1970-01-01");
        assert_eq!(epoch_millis_to_date(1_293_840_000_000), "2011-01-01");
        assert_eq!(epoch_millis_to_date(951_782_400_000), "2000-02-29");
        assert_eq!(epoch_millis_to_date(-86_400_000), "1969-12-31");
    }
}
