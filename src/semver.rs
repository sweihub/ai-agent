// Source: /data/home/swei/claudecode/openclaudecode/src/utils/semver.ts
use std::process::Command;

pub fn gt(a: &str, b: &str) -> bool {
    compare(a, b) > 0
}

pub fn gte(a: &str, b: &str) -> bool {
    compare(a, b) >= 0
}

pub fn lt(a: &str, b: &str) -> bool {
    compare(a, b) < 0
}

pub fn lte(a: &str, b: &str) -> bool {
    compare(a, b) <= 0
}

pub fn satisfies(version: &str, range: &str) -> bool {
    let output = Command::new("semver").args(["-r", range, version]).output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => satisfies_fallback(version, range),
    }
}

pub fn order(a: &str, b: &str) -> i32 {
    compare(a, b)
}

fn compare(a: &str, b: &str) -> i32 {
    let parts_a: Vec<&str> = a.trim().split('.').collect();
    let parts_b: Vec<&str> = b.trim().split('.').collect();

    let max_len = parts_a.len().max(parts_b.len());

    for i in 0..max_len {
        let v_a = parts_a
            .get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let v_b = parts_b
            .get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        if v_a > v_b {
            return 1;
        } else if v_a < v_b {
            return -1;
        }
    }
    0
}

fn satisfies_fallback(version: &str, range: &str) -> bool {
    let range_clean = range
        .trim()
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches(">")
        .trim_start_matches("<=")
        .trim_start_matches("<")
        .trim_start_matches("=");

    let version_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    let range_parts: Vec<u32> = range_clean
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if let (Some(v_major), Some(r_major)) = (version_parts.first(), range_parts.first()) {
        if v_major != r_major {
            return false;
        }
        if let Some(r_minor) = range_parts.get(1) {
            if let Some(v_minor) = version_parts.get(1) {
                if v_minor < r_minor {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gt() {
        assert!(gt("2.0.0", "1.0.0"));
        assert!(!gt("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_lt() {
        assert!(lt("1.0.0", "2.0.0"));
        assert!(!lt("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_compare_equal() {
        assert_eq!(compare("1.0.0", "1.0.0"), 0);
        assert_eq!(compare("1.2.3", "1.2.3"), 0);
    }
}
