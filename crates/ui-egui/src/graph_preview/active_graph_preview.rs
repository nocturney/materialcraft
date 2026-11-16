use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

use materialcraft_engine::NodeLibrary;

use super::GraphPreviewKey;

pub(super) struct ActiveGraphPreview {
    pub(super) library: Arc<NodeLibrary>,
    pub(super) key: GraphPreviewKey,
    pub(super) cancel: Arc<AtomicBool>,
    pub(super) result: Receiver<Option<Vec<u8>>>,
}

impl Drop for ActiveGraphPreview {
    fn drop(&mut self) {
        // Dropping preview state cancels its background cook
        self.cancel.store(true, Ordering::Relaxed);
    }
}
