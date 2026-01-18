use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Default, Zeroize, ZeroizeOnDrop)]
struct SecretKey {
    key: String,
}

#[derive(Clone)]
pub struct MasterKeyCache {
    inner: Arc<RwLock<Option<SecretKey>>>,
}

impl Default for MasterKeyCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterKeyCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set(&self, key: String) {
        let mut guard = self.inner.write().unwrap();
        *guard = Some(SecretKey { key });
    }

    pub fn get(&self) -> Option<String> {
        let guard = self.inner.read().unwrap();
        guard.as_ref().map(|s| s.key.clone())
    }

    pub fn is_set(&self) -> bool {
        let guard = self.inner.read().unwrap();
        guard.is_some()
    }

    pub fn clear(&self) {
        let mut guard = self.inner.write().unwrap();
        *guard = None;
    }
}
