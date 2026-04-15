// Source: /data/home/swei/claudecode/openclaudecode/src/constants/keys.ts
use crate::constants::env::{ai, system};
use crate::utils::env_utils::is_env_truthy;

pub fn get_growthbook_client_key() -> String {
    let user_type = std::env::var(ai::USER_TYPE).unwrap_or_default();
    let enable_growthbook_dev = std::env::var(system::ENABLE_GROWTHBOOK_DEV).unwrap_or_default();

    if user_type == "ant" {
        if is_env_truthy(Some(&enable_growthbook_dev)) {
            "sdk-yZQvlplybuXjYh6L".to_string()
        } else {
            "sdk-xRVcrliHIlrg4og4".to_string()
        }
    } else {
        "sdk-zAZezfDKGoZuXXKe".to_string()
    }
}
