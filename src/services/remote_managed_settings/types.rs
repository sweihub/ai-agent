// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RemoteManagedSettings {
    pub settings: HashMap<String, String>,
}

pub type RemoteSettingsFetchResult = Result<RemoteManagedSettings, String>;
