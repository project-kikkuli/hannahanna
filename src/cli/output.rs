use crate::vcs::{VcsType, Worktree};
use serde_json::{json, Value};

/// Shared wire representation for companion tools; display-only diagnostics
/// stay on stderr and callers must still check the command's exit status.
pub fn worktree_json(worktree: &Worktree, vcs_type: VcsType) -> Value {
    json!({
        "name": worktree.name,
        "path": worktree.path,
        "branch": worktree.branch,
        "commit": worktree.commit,
        "parent": worktree.parent,
        "vcs_type": vcs_type.as_str(),
    })
}
