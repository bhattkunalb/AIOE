pub struct SwapManager {
    _enabled: bool,
}

impl Default for SwapManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SwapManager {
    pub fn new() -> Self {
        Self { _enabled: true }
    }
}
