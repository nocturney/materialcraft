use std::sync::{Arc, Mutex};

use materialcraft_engine::LibraryCachePrefs;

use super::{PreviewCacheLimits, PreviewCacheState};

#[derive(Clone)]
pub(crate) struct PreviewCacheSettings(Arc<Mutex<PreviewCacheState>>);

impl PreviewCacheSettings {
    pub fn new(prefs: LibraryCachePrefs) -> Self {
        Self(Arc::new(Mutex::new(PreviewCacheState { limits: prefs.into(), revision: 0, epoch: 0, clear_pending: false, disabled: false })))
    }

    pub fn get(&self) -> PreviewCacheState {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn update(&self, limits: PreviewCacheLimits, clear: bool) {
        let mut state = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.limits != limits || clear {
            state.limits = limits;
            state.revision = state.revision.wrapping_add(1);
            state.disabled = false;
            if clear {
                state.epoch = state.epoch.wrapping_add(1);
                state.clear_pending = true;
            }
        }
    }

    pub fn finish(&self, completed: PreviewCacheState, failed: bool) {
        let mut state = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

        // An old failure must not disable newer limits or consume a newer clear request
        if state.epoch == completed.epoch && (!failed || state.revision == completed.revision) {
            state.clear_pending = false;

            // Failed cleanup must bypass stale files and rebuild from source
            state.disabled = failed;
        }
    }
}
