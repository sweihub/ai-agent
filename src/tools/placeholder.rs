// Source: /data/home/swei/claudecode/openclaudecode/src/Tool.ts
//! Placeholder tool utilities

/// Macro to create a simple placeholder tool struct with Default trait
/// Usage: placeholder_tool!(MonitorTool);
/// This creates: struct MonitorTool, impl with new() -> Self, impl Default
#[macro_export]
macro_rules! placeholder_tool {
    ($struct_name:ident) => {
        pub struct $struct_name;

        impl $struct_name {
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
