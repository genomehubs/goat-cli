use crate::utils::{
    expression::Variable,
    utils::{did_you_mean, parse_comma_separated},
};
use anyhow::{bail, Result};
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
                bail!(
                    "In your variable (`-v`) you typed \"{}\" - did you mean \"{}\"?",
                    variable,
                    value
                )
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
    ) -> Result<String> {
        let base = "&fields=";
        let delimiter = "%2C";

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
                    bail!(
                        "In your variable (`-v`) you typed \"{}\" - did you mean \"{}\"?",
                        variable,
                        value
                    )
                }
            }
        }

        parsed_string += base;
        for el in split_vec {
            parsed_string += &el;
            parsed_string += delimiter;
        }

        // should be okay to do an unchecked drain here
        parsed_string.drain(parsed_string.len() - 3..);

        Ok(parsed_string)
    }
}
