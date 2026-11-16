use std::ops::Range;

pub struct XmlEdit {
    pub range: Range<usize>,
    pub replacement: String,
}
