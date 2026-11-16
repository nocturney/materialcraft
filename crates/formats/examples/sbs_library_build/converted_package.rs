pub struct ConvertedPackage {
    pub xml: String,
    pub resources: Vec<&'static str>,
    pub report: serde_json::Value,
}
