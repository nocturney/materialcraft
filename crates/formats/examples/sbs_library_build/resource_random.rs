pub(super) struct ResourceRandom(pub(super) u64);

impl ResourceRandom {
    pub(super) fn next(&mut self) -> u64 {
        // Xorshift64 uses the standard 13, 7, 17 shift sequence
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
