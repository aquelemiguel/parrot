use crate::sources::SourceGetter;

pub struct YouTubeGetter {}

impl SourceGetter for YouTubeGetter {
    fn fetch(&self, url: &str) {
        println!("{}", url);
    }
}
