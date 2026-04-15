// Source: /data/home/swei/claudecode/openclaudecode/src/utils/memory/versions.ts
use crate::git_root::find_git_root;

pub fn project_is_in_git_repo(cwd: &str) -> bool {
    find_git_root(cwd).is_some()
}
