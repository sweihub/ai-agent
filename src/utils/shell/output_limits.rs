use std::env;

pub const BASH_MAX_OUTPUT_UPPER_LIMIT: u32 = 150_000;
pub const BASH_MAX_OUTPUT_DEFAULT: u32 = 30_000;

pub fn get_max_output_length() -> u32 {
    let var = env::var("BASH_MAX_OUTPUT_LENGTH").ok();
    if let Some(v) = var {
        if let Ok(parsed) = v.parse::<u32>() {
            if parsed <= BASH_MAX_OUTPUT_UPPER_LIMIT {
                return parsed;
            }
        }
    }
    BASH_MAX_OUTPUT_DEFAULT
}
