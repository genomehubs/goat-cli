//! Code generation for `src/utils/variable_data.rs`.
//!
//! Reads `vars/taxon_vars.json` and `vars/assembly_vars.json` (snapshots of
//! the GoaT `resultFields` API endpoint) and writes a `variable_data.rs` file
//! into `$OUT_DIR` that is `include!`'d by the actual source file.
//!
//! To update the data: run `vars/get_vars.bash` then `cargo build`.

use serde_json::Value;
use std::fmt::Write as FmtWrite;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=vars/taxon_vars.json");
    println!("cargo:rerun-if-changed=vars/assembly_vars.json");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    let taxon_json: Value = serde_json::from_str(
        &std::fs::read_to_string("vars/taxon_vars.json")
            .expect("vars/taxon_vars.json not found"),
    )
    .expect("failed to parse vars/taxon_vars.json");

    let assembly_json: Value = serde_json::from_str(
        &std::fs::read_to_string("vars/assembly_vars.json")
            .expect("vars/assembly_vars.json not found"),
    )
    .expect("failed to parse vars/assembly_vars.json");

    let mut code = String::new();
    writeln!(code, "lazy_static! {{").unwrap();
    generate_map(&mut code, &taxon_json, "GOAT_TAXON_VARIABLE_DATA");
    generate_map(&mut code, &assembly_json, "GOAT_ASSEMBLY_VARIABLE_DATA");
    writeln!(code, "}}").unwrap();

    std::fs::write(Path::new(&out_dir).join("variable_data.rs"), code)
        .expect("failed to write generated variable_data.rs");
}

/// Expression-relevant aggregation functions (subset of `VALID_EXPRESSION_FUNCTIONS`
/// that actually appear in `resultFields` summary arrays).
const EXPR_FUNCTIONS: &[&str] = &["min", "max", "count", "length", "sp_count", "range"];

fn generate_map(out: &mut String, json: &Value, name: &str) {
    let fields = json["fields"]
        .as_object()
        .unwrap_or_else(|| panic!("no 'fields' object in JSON for {}", name));

    writeln!(
        out,
        "    pub static ref {name}: BTreeMap<&'static str, Variable<'static>> = {{"
    )
    .unwrap();
    writeln!(out, "        let mut m = BTreeMap::new();").unwrap();

    // Sort keys for deterministic, diff-friendly output.
    let mut fields_vec: Vec<(&String, &Value)> = fields.iter().collect();
    fields_vec.sort_by_key(|(k, _)| k.as_str());

    for (field_name, field_data) in &fields_vec {
        let display_name = field_data["display_name"]
            .as_str()
            .unwrap_or(field_name.as_str());
        let type_of = map_type(field_data);
        let functions = map_functions(field_data);

        writeln!(
            out,
            "        m.insert({:?}, Variable {{ display_name: {:?}, type_of: {type_of}, functions: {functions} }});",
            field_name.as_str(),
            display_name,
        )
        .unwrap();
    }

    writeln!(out, "        m").unwrap();
    writeln!(out, "    }};").unwrap();
}

/// Map a `resultFields` JSON type string to a `TypeOf` variant.
fn map_type(field: &Value) -> String {
    match field["type"].as_str() {
        Some("long") => "TypeOf::Long".into(),
        Some("integer") => "TypeOf::Integer".into(),
        Some("short") => "TypeOf::Short".into(),
        Some("date") => "TypeOf::Date".into(),
        Some("half_float") | Some("float") | Some("double") => "TypeOf::HalfFloat".into(),
        Some("1dp") => "TypeOf::OneDP".into(),
        Some("2dp") | Some("4dp") => "TypeOf::TwoDP".into(),
        // keyword (or null/missing type) — check for an enum constraint
        Some("keyword") | None => map_keyword(field),
        // geo_point, object, nested, etc. — not usable in expressions
        Some(_) => "TypeOf::None".into(),
    }
}

fn map_keyword(field: &Value) -> String {
    if let Some(enums) = field["constraint"]["enum"].as_array() {
        let values: Vec<String> = enums
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| format!("{s:?}"))
            .collect();
        format!("TypeOf::Keyword(vec![{}])", values.join(", "))
    } else {
        r#"TypeOf::Keyword(vec![""])"#.into()
    }
}

/// Map a `resultFields` `summary` array to a `Function` variant.
///
/// Only retains values that are valid expression functions.
fn map_functions(field: &Value) -> String {
    if let Some(arr) = field["summary"].as_array() {
        let funcs: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| EXPR_FUNCTIONS.contains(s))
            .map(|s| format!("{s:?}"))
            .collect();
        if !funcs.is_empty() {
            return format!("Function::Some(vec![{}])", funcs.join(", "));
        }
    }
    "Function::None".into()
}
