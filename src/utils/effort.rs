// Source: /data/home/swei/claudecode/openclaudecode/src/utils/effort.ts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Minimal,
    Low,
    Medium,
    High,
    Maximum,
}

impl Effort {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minimal" => Effort::Minimal,
            "low" => Effort::Low,
            "medium" => Effort::Medium,
            "high" => Effort::High,
            "maximum" => Effort::Maximum,
            _ => Effort::Medium,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Effort::Minimal => "minimal",
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Maximum => "maximum",
        }
    }

    pub fn max_tokens(&self) -> u32 {
        match self {
            Effort::Minimal => 500,
            Effort::Low => 1000,
            Effort::Medium => 2000,
            Effort::High => 4000,
            Effort::Maximum => 8000,
        }
    }

    pub fn temperature(&self) -> f32 {
        match self {
            Effort::Minimal => 0.0,
            Effort::Low => 0.1,
            Effort::Medium => 0.3,
            Effort::High => 0.5,
            Effort::Maximum => 0.7,
        }
    }
}

impl Default for Effort {
    fn default() -> Self {
        Effort::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effort_parsing() {
        assert_eq!(Effort::from_str("high"), Effort::High);
        assert_eq!(Effort::from_str("HIGH"), Effort::High);
        assert_eq!(Effort::from_str("unknown"), Effort::Medium);
    }

    #[test]
    fn test_effort_values() {
        assert_eq!(Effort::High.max_tokens(), 4000);
        assert_eq!(Effort::Medium.temperature(), 0.3);
    }
}
