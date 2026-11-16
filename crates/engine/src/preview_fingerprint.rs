//! Stable keys for disposable library thumbnails, independent of process addresses and timestamps.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

use crate::NodeLibrary;

const MAX_DEPENDENCY_PACKAGES: usize = 4096;
const RESOURCE_HASH_CHUNK_BYTES: usize = 64 * 1024;

impl NodeLibrary {
    /// Hash the source packages and resource bytes used by a library preview. The caller must
    /// also key its rendering implementation and display size. Missing dependencies or an
    /// excessive dependency closure disable persistence; they must not leave a stale disk hit.
    pub fn preview_fingerprint(&self, package: &str, graph: &str, cancel: &AtomicBool) -> Option<[u8; 32]> {
        self.packages.get(package)?.graph_by_name(graph)?;
        let mut hash = Sha256::new();
        serde_json::to_writer(&mut hash, &("materialcraft-library-preview-v1", package, graph)).ok()?;
        let mut pending = BTreeSet::from([package]);
        let mut visited = BTreeSet::new();
        while let Some(path) = pending.pop_first() {
            if cancel.load(Ordering::Relaxed) || visited.len() >= MAX_DEPENDENCY_PACKAGES {
                return None;
            }

            if !visited.insert(path) {
                continue;
            }

            let source = self.packages.get(path)?;

            // Stream the model to include bytes omitted from package serialization without a copy
            serde_json::to_writer(&mut hash, &(path, source.as_ref())).ok()?;
            for resource in &source.resources {
                hash.update((resource.bytes.len() as u64).to_le_bytes());
                for chunk in resource.bytes.as_slice().chunks(RESOURCE_HASH_CHUNK_BYTES) {
                    if cancel.load(Ordering::Relaxed) {
                        return None;
                    }

                    hash.update(chunk);
                }
            }

            for instance in source.graphs.iter().flat_map(|graph| graph.instances()) {
                if !instance.package.is_empty() && !visited.contains(instance.package.as_str()) {
                    pending.insert(&instance.package);
                    if pending.len().saturating_add(visited.len()) > MAX_DEPENDENCY_PACKAGES {
                        return None;
                    }
                }
            }
        }

        Some(hash.finalize().into())
    }
}

#[cfg(test)]
#[path = "preview_fingerprint.test.rs"]
mod tests;
