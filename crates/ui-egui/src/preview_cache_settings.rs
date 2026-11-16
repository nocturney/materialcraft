//! Shared limits for existing workers and one background maintenance job at a time.

#[path = "preview_cache/preview_cache_limits.rs"]
mod limits;
#[path = "preview_cache/preview_cache_maintenance.rs"]
mod maintenance;
#[path = "preview_cache/preview_cache_settings.rs"]
mod settings;
#[path = "preview_cache/preview_cache_state.rs"]
mod state;

pub(super) use limits::PreviewCacheLimits;
pub(super) use maintenance::PreviewCacheMaintenance;
pub(super) use settings::PreviewCacheSettings;
pub(super) use state::PreviewCacheState;

#[cfg(test)]
mod tests {
    use super::*;
    use materialcraft_engine::LibraryCachePrefs;

    #[test]
    fn an_old_cleanup_cannot_finish_a_new_clear_request() {
        let settings = PreviewCacheSettings::new(LibraryCachePrefs::default());
        settings.update(settings.get().limits, true);
        let old = settings.get();
        settings.update(LibraryCachePrefs { disk_mib: 128, max_previews: 8192 }.into(), true);
        settings.finish(old, false);

        assert!(settings.get().clear_pending);

        let latest = settings.get();
        settings.finish(latest, true);

        assert!(!settings.get().clear_pending, "failed cleanup must allow rendering to resume");
        assert!(settings.get().disabled, "failed cleanup must bypass old files");
    }
}
