use materialcraft_engine::LibraryCachePrefs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PreviewCacheLimits {
    pub budget_bytes: u64,
    pub max_entries: usize,
}

impl From<LibraryCachePrefs> for PreviewCacheLimits {
    fn from(prefs: LibraryCachePrefs) -> Self {
        let prefs = prefs.sanitized();
        Self { budget_bytes: u64::from(prefs.disk_mib) * 1024 * 1024, max_entries: prefs.max_previews as usize }
    }
}
