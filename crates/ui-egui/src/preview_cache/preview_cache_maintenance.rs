use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use materialcraft_engine::LibraryCachePrefs;

use super::{PreviewCacheSettings, PreviewCacheState};

const BUSY_RETRY_DELAY: Duration = Duration::from_millis(100);

pub(crate) struct PreviewCacheMaintenance {
    pub settings: PreviewCacheSettings,
    pub(super) active: Option<(PreviewCacheState, Receiver<std::io::Result<bool>>)>,
    pub(super) applied: Option<u64>,
    retry_at: Option<Instant>,
}

impl PreviewCacheMaintenance {
    pub fn new(prefs: LibraryCachePrefs) -> Self {
        Self { settings: PreviewCacheSettings::new(prefs), active: None, applied: None, retry_at: None }
    }

    pub fn tick(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut error = None;
        if let Some((state, active)) = &self.active {
            let state = *state;
            let result = match active.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err(std::io::Error::other("preview cache worker stopped"))),
            };
            if let Some(result) = result {
                self.active = None;
                match result {
                    Ok(true) => {
                        self.applied = Some(state.revision);
                        self.settings.finish(state, false);
                    }
                    Ok(false) => self.retry_at = Some(Instant::now() + BUSY_RETRY_DELAY),
                    Err(reason) => {
                        self.applied = Some(state.revision);
                        self.settings.finish(state, true);
                        error = Some(format!("Library preview cache unavailable: {reason}"));
                    }
                }
            }
        }

        if self.active.is_none() && self.applied != Some(self.settings.get().revision) {
            if let Some(delay) = self.retry_at.and_then(|when| when.checked_duration_since(Instant::now())) {
                ctx.request_repaint_after(delay);
                return error;
            }

            self.retry_at = None;
            let settings = self.settings.clone();
            let state = settings.get();
            let context = ctx.clone();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            match std::thread::Builder::new().name("materialcraft-preview-cache".into()).spawn(move || {
                let result = match super::super::preview_disk_cache::PreviewDiskCache::configured(settings) {
                    Some(cache) => cache.maintain(state),
                    None => Ok(true),
                };

                let _ = sender.send(result);
                context.request_repaint();
            }) {
                Ok(_) => self.active = Some((state, receiver)),
                Err(reason) => {
                    self.applied = Some(state.revision);
                    self.settings.finish(state, true);
                    error = Some(format!("Library preview cache worker unavailable: {reason}"));
                }
            }
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stale_failure(clear: bool) {
        let context = egui::Context::default();
        let mut maintenance = PreviewCacheMaintenance::new(LibraryCachePrefs::default());
        let previous = maintenance.settings.get();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        maintenance.active = Some((previous, receiver));
        maintenance.retry_at = Some(Instant::now() + Duration::from_secs(60));
        maintenance.settings.update(LibraryCachePrefs { disk_mib: 128, max_previews: 8192 }.into(), clear);
        sender.send(Err(std::io::Error::other("failure from the previous request"))).unwrap();

        let error = maintenance.tick(&context);

        let latest = maintenance.settings.get();
        assert!(error.is_some());
        assert_eq!(latest.clear_pending, clear, "an old error must not consume a newer clear request");
        assert!(!latest.disabled, "an old error must not disable a newer configuration");
        assert_eq!(maintenance.applied, Some(previous.revision));
        assert!(maintenance.active.is_none(), "the retry delay keeps this test independent of disk workers");
    }

    #[test]
    fn old_worker_failure_preserves_a_new_clear_request() {
        stale_failure(true);
    }

    #[test]
    fn old_worker_failure_preserves_new_limits_without_a_clear() {
        stale_failure(false);
    }

    #[test]
    fn rapid_changes_keep_one_maintenance_worker_and_coalesce_the_latest_request() {
        let context = egui::Context::default();
        let mut maintenance = PreviewCacheMaintenance::new(LibraryCachePrefs::default());
        let previous = maintenance.settings.get();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        maintenance.active = Some((previous, receiver));
        maintenance.retry_at = Some(Instant::now() + Duration::from_secs(60));

        for size in 1..=100 {
            maintenance.settings.update(LibraryCachePrefs { disk_mib: size, max_previews: 4096 }.into(), true);

            assert!(maintenance.tick(&context).is_none());
            assert!(maintenance.applied.is_none());
        }

        // Completing the old request leaves the latest clear queued
        sender.send(Ok(true)).unwrap();
        assert!(maintenance.tick(&context).is_none());
        assert!(maintenance.active.is_none());
        assert_eq!(maintenance.applied, Some(previous.revision));
        assert!(maintenance.settings.get().clear_pending);

        let latest = maintenance.settings.get();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        maintenance.active = Some((latest, receiver));
        sender.send(Ok(true)).unwrap();

        assert!(maintenance.tick(&context).is_none());
        assert!(maintenance.active.is_none());
        assert_eq!(maintenance.applied, Some(latest.revision));
        assert!(!maintenance.settings.get().clear_pending);
    }
}
