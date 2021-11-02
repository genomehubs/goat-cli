use crate::search::agg_values::Records;
use crate::search::raw_values::RawRecords;
#[derive(Clone)]
pub struct CombinedValues {
    pub raw: Option<RawRecords>,
    pub agg: Option<Records>,
}
