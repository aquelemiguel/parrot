use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::messaging::message::ParrotMessage;

lazy_static! {
    static ref LOCALES: HashMap<String, HashMap<String, String>> = HashMap::new();
}

impl ParrotMessage {
    pub fn localize(&self, locale: &str) -> String {
        let self_str = format!("{self}");
        if !LOCALES.contains_key(locale) {
            return self_str;
        }

        let mappings = &LOCALES[locale];
        if !mappings.contains_key(&self_str) {
            return self_str;
        }

        mappings[&self_str].clone()
    }
}
