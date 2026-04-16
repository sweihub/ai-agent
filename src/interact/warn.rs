// Source: /data/home/swei/claudecode/openclaudecode/src/ink/warn.ts
pub fn if_not_integer(value: Option<f64>, name: &str) {
    if let Some(v) = value {
        if v.fract() != 0.0 {
            log::warn!("{} should be an integer, got {}", name, v);
        }
    }
}