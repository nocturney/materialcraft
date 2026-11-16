use super::PreviewCacheLimits;

#[derive(Clone, Copy)]
pub(crate) struct PreviewCacheState {
    pub limits: PreviewCacheLimits,
    pub revision: u64,
    pub epoch: u64,
    pub clear_pending: bool,
    pub disabled: bool,
}
