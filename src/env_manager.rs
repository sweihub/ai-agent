#![allow(dead_code)]

use std::collections::HashMap;

pub struct EnvVar {
    pub key: String,
    pub value: String,
}

pub fn get_all_env_vars() -> Vec<EnvVar> {
    std::env::vars()
        .map(|(key, value)| EnvVar { key, value })
        .collect()
}

pub fn set_env_var(key: &str, value: &str) {
    std::env::set_var(key, value);
}
