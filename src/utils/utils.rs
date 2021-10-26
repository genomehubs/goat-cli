// taxids should be comma separated
// remove whitespace from beginning and end of each element of the vec.
// also remove duplicates
pub fn parse_multiple_taxids(taxids: &str) -> Vec<String> {
    let res: Vec<&str> = taxids.split(",").map(|s| s).collect();

    let mut res2 = Vec::new();
    for mut str in res {
        // sort the rights
        while str.ends_with(" ") {
            let len = str.len();
            let new_len = len.saturating_sub(" ".len());
            str = &str[..new_len];
        }
        // sort the rights
        let mut index = 0;
        while str.starts_with(" ") {
            index += 1;
            str = &str[index..];
        }
        res2.push(str.to_string());
    }
    res2
}

// make the GoaT API URLs
pub fn make_goat_search_urls(
    taxids: Vec<String>,
    goat_url: &str,
    tax_tree: &str,
    include_estimates: bool,
    include_raw_values: bool,
    summarise_values_by: &str,
    result: &str,
    taxonomy: &str,
) -> Vec<String> {
    let mut res = Vec::new();
    for el in taxids {
        let url = format!(
        "{}search?query=tax_{}%28{}%29&includeEstimates={}&includeRawValues={}&summaryValues={}&result={}&taxonomy={}",
        goat_url, tax_tree, el, include_estimates, include_raw_values, summarise_values_by, result, taxonomy
    );
        res.push(url);
    }
    res
}
