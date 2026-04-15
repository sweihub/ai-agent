//! Session discovery for assistant mode

use crate::Result;

/// Discover assistant sessions - returns empty list in current implementation
pub async fn discover_assistant_sessions() -> Result<Vec<()>> {
    Ok(vec![])
}
