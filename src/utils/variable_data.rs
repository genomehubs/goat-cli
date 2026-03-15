use crate::utils::expression::{Function, TypeOf, Variable};
use lazy_static::lazy_static;
use std::collections::BTreeMap;

// Generated at build time from vars/taxon_vars.json and vars/assembly_vars.json.
// To regenerate: run vars/get_vars.bash then cargo build.
include!(concat!(env!("OUT_DIR"), "/variable_data.rs"));
