//!
//! Runtime field registry, from the live GoaT `resultFields` endpoint.
//!
//! This lets the expression engine (`-e`) and variables (`-v`) accept fields
//! that were added to GoaT after the last time the static variable-data file
//! was regenerated, without requiring a binary rebuild.
//!
//! The registry is only loaded when an expression or variable list names a
//! field the static data does not know (see [`prepare`]), and the field
//! names are cached on disk for a day, so most runs make no request for it.
//!
//! # Fallback behaviour
//!
//! If the HTTP request fails (network error, API down) the registry is
//! silently left empty and all field lookups return `false`.  The static
//! `GOAT_TAXON_VARIABLE_DATA` / `GOAT_ASSEMBLY_VARIABLE_DATA` maps are
//! still used for detailed type-checking; the dynamic registry only widens
//! the set of *accepted* field names.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::client::GoatClient;
use crate::utils::args::ArgMatchesExt;
use crate::utils::expression::{canonical_field, CLIexpression};
use crate::utils::utils::parse_comma_separated;
use crate::utils::variable_data::{GOAT_ASSEMBLY_VARIABLE_DATA, GOAT_TAXON_VARIABLE_DATA};
use crate::{IndexType, GOAT_URL, TAXONOMY};

/// How long the on-disk cache of field names is used before refetching.
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

static TAXON_FIELDS: OnceLock<HashSet<String>> = OnceLock::new();
static ASSEMBLY_FIELDS: OnceLock<HashSet<String>> = OnceLock::new();

fn cell_for(index_type: IndexType) -> &'static OnceLock<HashSet<String>> {
    match index_type {
        IndexType::Taxon => &TAXON_FIELDS,
        IndexType::Assembly => &ASSEMBLY_FIELDS,
    }
}

/// Where the field names for `index_type` are cached: under
/// `$XDG_CACHE_HOME`, `~/.cache` or `%LOCALAPPDATA%`, in `goat-cli/`.
fn cache_path(index_type: IndexType) -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))?;
    Some(base.join("goat-cli").join(format!("{}_fields.json", index_type)))
}

/// Read cached field names, if the cache exists and is fresh.
fn read_cache(path: &Path) -> Option<HashSet<String>> {
    let age = SystemTime::now()
        .duration_since(std::fs::metadata(path).ok()?.modified().ok()?)
        .ok()?;
    if age > CACHE_TTL {
        return None;
    }
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Cache field names. Best effort: failing to cache is not an error.
fn write_cache(path: &Path, fields: &HashSet<String>) {
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
        // write then rename, so a concurrent run never reads a partial file
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&tmp, serde_json::to_string(fields)?)?;
        std::fs::rename(&tmp, path)
    };
    let _ = write();
}

/// Fetch all field names from the GoaT `resultFields` endpoint.
///
/// Returns `None` on any error, so that a failure is not cached.
async fn fetch_fields(index_type: IndexType) -> Option<HashSet<String>> {
    let url = format!(
        "{}resultFields?result={}&taxonomy={}",
        *GOAT_URL,
        index_type,
        *TAXONOMY
    );
    let v: Value = GoatClient::new().get_json(&url).await.ok()?;
    v["fields"]
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
}

/// Initialise the dynamic field registry for `index_type`, from the disk
/// cache if it is fresh, otherwise from the API.
///
/// Safe to call multiple times — the work is done at most once.
/// Errors are swallowed; the registry will simply be empty.
pub async fn init(index_type: IndexType) {
    let cell = cell_for(index_type);
    if cell.get().is_some() {
        return;
    }
    let path = cache_path(index_type);
    let fields = match path.as_deref().and_then(read_cache) {
        Some(fields) => fields,
        None => match fetch_fields(index_type).await {
            Some(fields) => {
                if let Some(path) = &path {
                    write_cache(path, &fields);
                }
                fields
            }
            None => HashSet::new(),
        },
    };
    // Ignore the error — it just means another task beat us to it.
    let _ = cell.set(fields);
}

/// Load the registry for `index_type` if the expression (`-e`), variables
/// (`-v`) or report filters (`arc -x/-y`) in `matches` name a field that the
/// static data does not know.
///
/// Call this before the arguments are parsed, so that parsing can accept
/// fields added to GoaT since this binary was built.
pub async fn prepare(matches: &clap::ArgMatches, index_type: IndexType) {
    let data = match index_type {
        IndexType::Taxon => &*GOAT_TAXON_VARIABLE_DATA,
        IndexType::Assembly => &*GOAT_ASSEMBLY_VARIABLE_DATA,
    };
    let unknown_in_expression = ["expression", "x-filter", "y-filter"]
        .iter()
        .filter_map(|id| matches.opt_one::<String>(id))
        .any(|e| CLIexpression::new(e).has_unknown_field(data));
    let unknown_in_variables = matches
        .opt_one::<String>("variables")
        .map_or(false, |v| {
            parse_comma_separated(v)
                .iter()
                .any(|field| canonical_field(field, data).is_none())
        });
    if unknown_in_expression || unknown_in_variables {
        init(index_type).await;
    }
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

/// Like [`is_dynamic_field`], for either index.
pub fn is_dynamic_field_any(field: &str) -> bool {
    is_dynamic_field(field, IndexType::Taxon) || is_dynamic_field(field, IndexType::Assembly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_round_trip() {
        let path = std::env::temp_dir()
            .join(format!("goat_cli_registry_{}", std::process::id()))
            .join("taxon_fields.json");
        let fields: HashSet<String> = ["genome_size", "brand_new_field"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        write_cache(&path, &fields);
        assert_eq!(read_cache(&path), Some(fields));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn test_missing_cache_is_none() {
        assert_eq!(read_cache(Path::new("/nonexistent/goat-cli/taxon_fields.json")), None);
    }

    #[test]
    fn test_has_unknown_field() {
        let data = &*GOAT_TAXON_VARIABLE_DATA;
        assert!(!CLIexpression::new("genome_size > 1 OR max(c_value) < 2").has_unknown_field(data));
        assert!(!CLIexpression::new("ebp_metric_date >= 2023").has_unknown_field(data));
        assert!(CLIexpression::new("genome_size > 1 AND brand_new_field = x").has_unknown_field(data));
    }
}
