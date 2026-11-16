use serde::{Deserialize, Serialize};

/// Disposable library thumbnails; independent of package contents and undo history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibraryCachePrefs {
    /// Zero disables disk persistence.
    pub disk_mib: u32,
    pub max_previews: u32,
}

impl Default for LibraryCachePrefs {
    fn default() -> Self {
        Self { disk_mib: 64, max_previews: 4096 }
    }
}

impl LibraryCachePrefs {
    pub const MAX_DISK_MIB: u32 = 65_536;

    // Bound directory scans and metadata allocations even with externally supplied settings
    pub const MAX_PREVIEWS: u32 = 65_536;

    pub fn sanitized(self) -> Self {
        Self { disk_mib: self.disk_mib.min(Self::MAX_DISK_MIB), max_previews: self.max_previews.clamp(1, Self::MAX_PREVIEWS) }
    }
}
