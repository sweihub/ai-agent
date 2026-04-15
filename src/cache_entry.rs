//! Cache entry types.

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            expires_at: None,
        }
    }

    pub fn with_ttl(value: T, ttl_secs: u64) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + std::time::Duration::from_secs(ttl_secs)),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() > expires_at
        } else {
            false
        }
    }
}
