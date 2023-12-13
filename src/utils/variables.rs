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
    pub fn parse(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
        // this is a pretty hacky way of adding this in.
        taxon_toggle_direct: bool,
    ) -> Result<String> {
        const BASE: &str = "&fields=";
        const DELIMITER: &str = "%2C";
        const COLON: &str = "%3A";

        let mut parsed_string = String::new();

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

        parsed_string += BASE;
        for el in split_vec {
            parsed_string += &el;
            parsed_string += DELIMITER;

            if taxon_toggle_direct {
                // first add direct
                parsed_string += &el;
                parsed_string += COLON;
                parsed_string += "direct";
                // now we need to push two more
                parsed_string += DELIMITER;
                parsed_string += &el;
                parsed_string += COLON;
                parsed_string += "ancestor";
                parsed_string += DELIMITER;
                parsed_string += &el;
                parsed_string += COLON;
                parsed_string += "descendant";
                parsed_string += DELIMITER;
            }
        }

        // should be okay to do an unchecked drain here
        parsed_string.drain(parsed_string.len() - 3..);

        Ok(parsed_string)
    }

    /// Parse a variable name into a string which will be entered in the final URL
    /// to exclude missing and ancestral taxa.
    pub fn parse_exclude(
        &self,
        reference_data: &BTreeMap<&'static str, Variable<'static>>,
    ) -> Result<String> {
        const ANCESTRAL: &str = "&excludeAncestral";
        const MISSING: &str = "&excludeMissing";
        const OPEN_ANGLE_BRACE: &str = "%5B";
        const CLOSE_ANGLE_BRACE: &str = "%5D";

        let mut exclusion_string = String::new();

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

        for (exclude_index, field) in split_vec.into_iter().enumerate() {
            exclusion_string += ANCESTRAL;
            exclusion_string += OPEN_ANGLE_BRACE;
            exclusion_string += &exclude_index.to_string();
            exclusion_string += CLOSE_ANGLE_BRACE;
            exclusion_string += &format!("={field}");

            // add missing
            exclusion_string += MISSING;
            exclusion_string += OPEN_ANGLE_BRACE;
            exclusion_string += &exclude_index.to_string();
            exclusion_string += CLOSE_ANGLE_BRACE;
            exclusion_string += &format!("={field}");
        }

        Ok(exclusion_string)
    }
}
