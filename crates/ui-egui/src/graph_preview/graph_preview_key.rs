#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum GraphPreviewKey {
    Graph(String, String),
    Native(String),
}
