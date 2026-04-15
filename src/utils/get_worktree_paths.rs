use std::path::Path;

pub fn get_worktree_paths(cwd: &Path) -> Vec<WorktreePath> {
    let git_dir = cwd.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        return vec![WorktreePath {
            path: cwd.to_path_buf(),
            is_main: true,
        }];
    }

    let worktrees_dir = Path::new(".git").join("worktrees");
    if worktrees_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&worktrees_dir) {
            return entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let commondir = e.path().join("commondir");
                    if commondir.exists() {
                        if let Ok(content) = std::fs::read_to_string(&commondir) {
                            let path = Path::new(content.trim());
                            return Some(WorktreePath {
                                path: path.to_path_buf(),
                                is_main: false,
                            });
                        }
                    }
                    None
                })
                .collect();
        }
    }

    vec![WorktreePath {
        path: cwd.to_path_buf(),
        is_main: true,
    }]
}

#[derive(Clone, Debug)]
pub struct WorktreePath {
    pub path: std::path::PathBuf,
    pub is_main: bool,
}
