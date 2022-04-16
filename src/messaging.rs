use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, OpenOptions},
    io::BufReader,
    path::Path,
};

use lazy_static::lazy_static;

use crate::strings::{AUTOPAUSE_OFF, AUTOPAUSE_ON, LOOP_DISABLED, LOOP_ENABLED, PAUSED};

pub const TRANSLATIONS_DIR: &str = "translations";

lazy_static! {
    pub static ref TRANSLATIONS: TranslationMap = read_translations(TRANSLATIONS_DIR);
    pub static ref TRANSLATION_FILE_REGEX: Regex = Regex::new(r"^(?P<lang>.*).json$").unwrap();
}

type TranslationMap = HashMap<String, HashMap<Message, String>>;

#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Message {
    AutopauseOn,
    AutopauseOff,
    LoopEnabled,
    LoopDisabled,
    Paused,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutopauseOn => f.write_str(AUTOPAUSE_ON),
            Self::AutopauseOff => f.write_str(AUTOPAUSE_OFF),
            Self::LoopEnabled => f.write_str(LOOP_ENABLED),
            Self::LoopDisabled => f.write_str(LOOP_DISABLED),
            Self::Paused => f.write_str(PAUSED),
        }
    }
}

pub fn translate(message: Message, lang: Option<&str>) -> String {
    let lang = match lang {
        Some(lang) => lang,
        _ => return format!("{message}"),
    };

    let translation_map = match TRANSLATIONS.get(lang) {
        Some(translation_map) => translation_map,
        _ => return format!("{message}"),
    };

    let translation = match translation_map.get(&message) {
        Some(translation) => translation,
        _ => return format!("{message}"),
    };

    translation.to_string()
}

pub fn read_translations(translations_dir: &str) -> TranslationMap {
    fs::read_dir(translations_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| valid_translation(path))
        .map(|path| (translation_key(&path), read_translation(&path)))
        .collect::<TranslationMap>()
}

pub fn read_translation(path: &Path) -> HashMap<Message, String> {
    let file = OpenOptions::new().read(true).open(path).unwrap();
    serde_json::from_reader(BufReader::new(file)).unwrap()
}

pub fn valid_translation(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_str().unwrap();
    let is_file = path.is_file();
    let is_match = TRANSLATION_FILE_REGEX.is_match(name);
    is_file && is_match
}

pub fn translation_key(path: &Path) -> String {
    let name = path.file_name().unwrap().to_str().unwrap();
    let captures = TRANSLATION_FILE_REGEX.captures(name).unwrap();
    let lang = captures.name("lang").unwrap().as_str();
    lang.to_string()
}
