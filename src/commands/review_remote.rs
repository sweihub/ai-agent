use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverageGateKind {
    Proceed,
    NotEnabled,
    LowBalance,
    NeedsConfirm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverageGate {
    pub kind: OverageGateKind,
    #[serde(rename = "billingNote")]
    pub billing_note: Option<String>,
    pub available: Option<i32>,
}

impl OverageGate {
    pub fn proceed(billing_note: &str) -> Self {
        Self {
            kind: OverageGateKind::Proceed,
            billing_note: Some(billing_note.to_string()),
            available: None,
        }
    }

    pub fn not_enabled() -> Self {
        Self {
            kind: OverageGateKind::NotEnabled,
            billing_note: None,
            available: None,
        }
    }

    pub fn low_balance(available: i32) -> Self {
        Self {
            kind: OverageGateKind::LowBalance,
            billing_note: None,
            available: Some(available),
        }
    }

    pub fn needs_confirm() -> Self {
        Self {
            kind: OverageGateKind::NeedsConfirm,
            billing_note: None,
            available: None,
        }
    }
}

pub fn confirm_overage() {}

pub fn check_overage_gate() -> OverageGate {
    OverageGate::proceed("")
}
