// Source: /data/home/swei/claudecode/openclaudecode/src/query/deps.ts
pub type CallModelFn = fn() -> Result<String, String>;
pub type MicrocompactFn = fn() -> Result<String, String>;
pub type AutocompactFn = fn() -> Result<(), String>;
pub type UuidFn = fn() -> String;

#[derive(Debug, Clone)]
pub struct QueryDeps {
    pub call_model: CallModelFn,
    pub microcompact: MicrocompactFn,
    pub autocompact: AutocompactFn,
    pub uuid: UuidFn,
}

impl QueryDeps {
    pub fn production() -> Self {
        Self {
            call_model: || Ok("response".to_string()),
            microcompact: || Ok("compact".to_string()),
            autocompact: || Ok(()),
            uuid: || uuid::Uuid::new_v4().to_string(),
        }
    }
}
