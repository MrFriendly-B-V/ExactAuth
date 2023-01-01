mod token;
pub use token::*;

const REGION_NL_BASE: &str = "https://start.exactonline.nl";

pub fn get_exact_url(path: &str) -> String {
    format!("{REGION_NL_BASE}{path}")
}