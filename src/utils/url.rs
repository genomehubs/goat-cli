use lazy_static::lazy_static;

const GOAT_URL_BASE: &str = "https://goat.genomehubs.org/api/";
const GOAT_API_VERSION: &str = "v0.0.1/";

lazy_static! {
    pub static ref GOAT_URL: String = format!("{}{}", GOAT_URL_BASE, GOAT_API_VERSION);
    pub static ref TAXONOMY: String = "ncbi".to_string();
}
