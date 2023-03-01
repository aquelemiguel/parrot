use lazy_static::lazy_static;
use std::{collections::HashMap, env};

const DEFAULT_LOCALES_PATH: &str = "data/locales";

lazy_static! {
    static ref LOCALES_PATH: String =
        env::var("DEFAULT_LOCALES_PATH").unwrap_or(DEFAULT_LOCALES_PATH.to_string());
    static ref LOCALES: HashMap<String, HashMap<String, String>> = HashMap::new();
}

pub fn localize(key: &str, locale: &str) -> String {
    if !LOCALES.contains_key(locale) {
        return key.to_owned();
    }

    let mappings = &LOCALES[locale];
    if !mappings.contains_key(key) {
        return key.to_owned();
    }

    mappings[key].clone()
}
