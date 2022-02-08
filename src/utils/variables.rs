// this module allows the user to have
// more fine grained control over which fields are
// returned

use crate::utils::expression::GOAT_VARIABLE_DATA;
use crate::utils::utils::parse_comma_separated;
use anyhow::{ensure, Result};

pub struct Variables<'a> {
    variables: &'a str,
}

impl<'a> Variables<'a> {
    pub fn new(str: &'a str) -> Self {
        Self { variables: str }
    }
    // split comma sep list
    // check against the database
    // format the string.
    pub fn parse(&self) -> Result<String> {
        let base = "&fields=";
        let delimiter = "%2C";

        let mut parsed_string = String::new();

        let split_vec = parse_comma_separated(&self.variables);
        // check that all the strings in split_vec are real
        let var_vec_check = &*GOAT_VARIABLE_DATA
            .iter()
            .map(|(e, _)| e.to_string())
            .collect::<Vec<String>>();

        // TODO: perhaps say which one it is?
        ensure!(split_vec.iter().all(|item| var_vec_check.contains(item)), "One of the variables you passed does not match the database. Please check all of your variables are spelled correctly.\nError: Run `goat search --print-expressions` to see a list of possible variables.");

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
