// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionRule.ts
#![allow(dead_code)]

//! Permission rule types and schema definitions.

use serde::{Deserialize, Serialize};

// Re-exports for backwards compatibility
pub use crate::types::permissions::{
    PermissionBehavior, PermissionRule, PermissionRuleSource, PermissionRuleValue,
};

/// ToolPermissionBehavior is the behavior associated with a permission rule.
/// 'allow' means the rule allows the tool to run.
/// 'deny' means the rule denies the tool from running.
/// 'ask' means the rule forces a prompt to be shown to the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehaviorSchema {
    Allow,
    Deny,
    Ask,
}

impl From<PermissionBehavior> for PermissionBehaviorSchema {
    fn from(behavior: PermissionBehavior) -> Self {
        match behavior {
            PermissionBehavior::Allow => PermissionBehaviorSchema::Allow,
            PermissionBehavior::Deny => PermissionBehaviorSchema::Deny,
            PermissionBehavior::Ask => PermissionBehaviorSchema::Ask,
        }
    }
}

impl From<PermissionBehaviorSchema> for PermissionBehavior {
    fn from(schema: PermissionBehaviorSchema) -> Self {
        match schema {
            PermissionBehaviorSchema::Allow => PermissionBehavior::Allow,
            PermissionBehaviorSchema::Deny => PermissionBehavior::Deny,
            PermissionBehaviorSchema::Ask => PermissionBehavior::Ask,
        }
    }
}
