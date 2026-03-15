use crate::error::{Error, ErrorKind, Result};
use crate::utils::{
    expression::Variable,
    utils::{did_you_mean, parse_comma_separated},
};
use std::collections::BTreeMap;

/// A struct to store the variables
/// passed in the `-v` flag on the
/// CLI.
pub struct Variables<'a> {
    /// Variables which need to be parsed.
    variables: &'a str,
}

impl<'a> Variables<'a> {
    /// Constructor for [`Variables`].
    pub fn new(str: &'a str) -> Self {
        Self { variables: str }
    }

    /// Parse a single variable. Used in report
    /// until something more sophisticated is made.
    pub fn parse_one(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
    ) -> Result<String> {
        let variable = self.variables;

        let var_vec_check = reference_data
            .iter()
            .map(|(e, _)| e.to_string())
            .collect::<Vec<String>>();

        if !var_vec_check.contains(&variable.to_string()) {
            let var_vec_mean = did_you_mean(&var_vec_check, variable);
            if let Some(value) = var_vec_mean {
                return Err(Error::new(ErrorKind::Variable(format!(
                    "you typed \"{}\" - did you mean \"{}\"?",
                    variable, value
                ))));
            }
        }
        Ok(variable.to_string())
    }

    /// Simple parsing of a comma separated string,
    /// which will error if the variable is not found
    /// with a suggestion as to which one you meant.
    ///
    /// Returns a plain comma-separated field list (no `&fields=` prefix, no percent-encoding).
    /// The caller is responsible for adding the field as a URL query parameter.
    pub fn parse(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
        // this is a pretty hacky way of adding this in.
        taxon_toggle_direct: bool,
    ) -> Result<String> {
        let split_vec = parse_comma_separated(self.variables);
        // check that all the strings in split_vec are real
        let var_vec_check = reference_data
            .iter()
            .map(|(e, _)| e.to_string())
            .collect::<Vec<String>>();

        for variable in &split_vec {
            // only if we find something which does not match...
            if !var_vec_check.contains(variable) {
                let var_vec_mean = did_you_mean(&var_vec_check, variable);
                if let Some(value) = var_vec_mean {
                    return Err(Error::new(ErrorKind::Variable(format!(
                        "you typed \"{}\" - did you mean \"{}\"?",
                        variable, value
                    ))));
                }
            }
        }

        let mut fields: Vec<String> = Vec::new();
        for el in split_vec {
            fields.push(el.clone());
            if taxon_toggle_direct {
                fields.push(format!("{}:direct", el));
                fields.push(format!("{}:ancestor", el));
                fields.push(format!("{}:descendant", el));
            }
        }

        Ok(fields.join(","))
    }

    /// Parse a variable name into key-value pairs for excluding missing and ancestral taxa.
    ///
    /// Returns `Vec<(param_name, field_name)>` pairs; the URL builder handles encoding.
    pub fn parse_exclude(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
    ) -> Result<Vec<(String, String)>> {
        let split_vec = parse_comma_separated(self.variables);
        // check that all the strings in split_vec are real
        let var_vec_check = reference_data
            .iter()
            .map(|(e, _)| e.to_string())
            .collect::<Vec<String>>();

        for variable in &split_vec {
            // only if we find something which does not match...
            if !var_vec_check.contains(variable) {
                let var_vec_mean = did_you_mean(&var_vec_check, variable);
                if let Some(value) = var_vec_mean {
                    return Err(Error::new(ErrorKind::Variable(format!(
                        "you typed \"{}\" - did you mean \"{}\"?",
                        variable, value
                    ))));
                }
            }
        }

        let mut pairs = Vec::new();
        for (exclude_index, field) in split_vec.into_iter().enumerate() {
            pairs.push((format!("excludeAncestral[{}]", exclude_index), field.clone()));
            pairs.push((format!("excludeMissing[{}]", exclude_index), field));
        }

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::variable_data::GOAT_TAXON_VARIABLE_DATA;

    // ── parse_one ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_one_known_variable_returns_name() {
        let v = Variables::new("genome_size");
        let result = v.parse_one(&GOAT_TAXON_VARIABLE_DATA).unwrap();
        assert_eq!(result, "genome_size");
    }

    #[test]
    fn test_parse_one_unknown_variable_with_typo_suggests_correction() {
        // "genome_siz" is close to "genome_size"
        let v = Variables::new("genome_siz");
        let err = v.parse_one(&GOAT_TAXON_VARIABLE_DATA).unwrap_err();
        assert!(
            err.to_string().contains("genome_size"),
            "expected suggestion 'genome_size' in: {}",
            err
        );
    }

    // ── parse ────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_single_variable_builds_fields_string() {
        let v = Variables::new("genome_size");
        let result = v.parse(&GOAT_TAXON_VARIABLE_DATA, false).unwrap();
        // Returns a plain field list — no &fields= prefix, no percent-encoding
        assert!(!result.starts_with("&fields="), "should not have &fields= prefix");
        assert!(result.contains("genome_size"));
        assert!(!result.ends_with(','), "trailing delimiter present");
    }

    #[test]
    fn test_parse_multiple_variables_all_present() {
        let v = Variables::new("genome_size,c_value");
        let result = v.parse(&GOAT_TAXON_VARIABLE_DATA, false).unwrap();
        assert!(result.contains("genome_size"));
        assert!(result.contains("c_value"));
        assert!(!result.ends_with(','));
    }

    #[test]
    fn test_parse_unknown_variable_returns_err() {
        let v = Variables::new("not_a_real_var_xyz");
        assert!(v.parse(&GOAT_TAXON_VARIABLE_DATA, false).is_err());
    }

    #[test]
    fn test_parse_toggle_direct_adds_extra_columns() {
        let v = Variables::new("genome_size");
        let result = v.parse(&GOAT_TAXON_VARIABLE_DATA, true).unwrap();
        assert!(result.contains("genome_size:direct"));
        assert!(result.contains("genome_size:ancestor"));
        assert!(result.contains("genome_size:descendant"));
        assert!(!result.ends_with(','));
    }

    // ── parse_exclude ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_exclude_contains_ancestral_and_missing_segments() {
        let v = Variables::new("genome_size");
        let pairs = v.parse_exclude(&GOAT_TAXON_VARIABLE_DATA).unwrap();
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        let vals: Vec<&str> = pairs.iter().map(|(_, v)| v.as_str()).collect();
        assert!(keys.iter().any(|k| k.contains("excludeAncestral")));
        assert!(keys.iter().any(|k| k.contains("excludeMissing")));
        assert!(vals.iter().all(|v| *v == "genome_size"));
    }

    #[test]
    fn test_parse_exclude_multiple_variables_all_indexed() {
        let v = Variables::new("genome_size,c_value");
        let pairs = v.parse_exclude(&GOAT_TAXON_VARIABLE_DATA).unwrap();
        // Two variables × two param types = 4 pairs
        assert_eq!(pairs.len(), 4);
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.iter().any(|k| k.contains("[0]")));
        assert!(keys.iter().any(|k| k.contains("[1]")));
    }
}
