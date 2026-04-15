// Source: /data/home/swei/claudecode/openclaudecode/src/services/analytics/growthbook.ts
pub fn get_feature_value<T>(_feature_key: &str, _default: T) -> T
where
    T: Default,
{
    T::default()
}

pub fn is_feature_enabled(_feature_key: &str) -> bool {
    false
}
