//!
//! Runtime field registry — fetches the live GoaT `resultFields` endpoint
//! once at startup and caches the result.
//!
//! This lets the expression engine accept fields that were added to GoaT
//! after the last time the static variable-data file was regenerated, without
//! requiring a binary rebuild.
//!
//! # Fallback behaviour
//!
//! If the HTTP request fails (network error, API down) the registry is
//! silently left empty and all field lookups return `false`.  The static
//! `GOAT_TAXON_VARIABLE_DATA` / `GOAT_ASSEMBLY_VARIABLE_DATA` maps are
//! still used for detailed type-checking; the dynamic registry only widens
//! the set of *accepted* field names.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde_json::Value;

use crate::client::GoatClient;
use crate::{IndexType, GOAT_URL, TAXONOMY};

static TAXON_FIELDS: OnceLock<HashSet<String>> = OnceLock::new();
static ASSEMBLY_FIELDS: OnceLock<HashSet<String>> = OnceLock::new();

fn cell_for(index_type: IndexType) -> &'static OnceLock<HashSet<String>> {
    match index_type {
        IndexType::Taxon => &TAXON_FIELDS,
        IndexType::Assembly => &ASSEMBLY_FIELDS,
    }
}

/// Fetch all field names from the GoaT `resultFields` endpoint.
///
/// Returns an empty set on any error so callers can treat this as a
/// best-effort operation.
async fn fetch_fields(index_type: IndexType) -> HashSet<String> {
    let url = format!(
        "{}resultFields?result={}&taxonomy={}",
        *GOAT_URL,
        index_type,
        *TAXONOMY
    );
    let client = GoatClient::new();
    let v: Value = match client.get_json(&url).await {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };
    v["fields"]
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

/// Initialise the dynamic field registry for `index_type`.
///
/// Safe to call multiple times — the HTTP request is made at most once.
/// Errors are swallowed; the registry will simply be empty.
pub async fn init(index_type: IndexType) {
    let cell = cell_for(index_type);
    if cell.get().is_some() {
        return;
    }
    let fields = fetch_fields(index_type).await;
    // Ignore the error — it just means another thread beat us to it.
    let _ = cell.set(fields);
}

/// Returns a clone of all field names in the registry for `index_type`.
///
/// Returns an empty set if the registry has not been initialised yet.
pub fn get_all_fields(index_type: IndexType) -> HashSet<String> {
    cell_for(index_type)
        .get()
        .cloned()
        .unwrap_or_default()
}

/// Returns `true` if `field` appears in the live GoaT field list for
/// `index_type`.
///
/// Returns `false` both when the field is unknown **and** when the registry
/// has not been initialised yet (i.e. [`init`] was not awaited before the
/// expression was validated).
pub fn is_dynamic_field(field: &str, index_type: IndexType) -> bool {
    cell_for(index_type)
        .get()
        .map(|fields| fields.contains(field))
        .unwrap_or(false)
}
