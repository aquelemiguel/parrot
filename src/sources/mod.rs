pub mod youtube;

pub trait SourceGetter {
    fn fetch(&self, url: &str);
}
