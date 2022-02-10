use crate::utils::expression::{Function, TypeOf, Variable};
use lazy_static::lazy_static;
use std::collections::BTreeMap;

// https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

lazy_static! {
    pub static ref GOAT_VARIABLE_DATA: BTreeMap<&'static str, Variable<'static>> = collection!(
        // automated input start
        // automated input end
    );
}
