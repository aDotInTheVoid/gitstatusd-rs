use std::{ffi::OsStr, fmt, io, path::Path, process};

///////////////////////////////////////////////////////////////////////////////
// Responce
///////////////////////////////////////////////////////////////////////////////

pub struct Responce {
    /// Request id. The same as the first field in the request.
    pub id: String,
    /// The inner responce.
    pub inner: ResponceInner
}

/// Most of a responce, depending on if we were in git.
pub enum ResponceInner {
    /// We arn't in git
    NotGit,
    /// We're in it, details inside.
    Git(ResponceGit)
}

/// Details about git state.
///
/// Note: Renamed files are reported as deleted plus new.
pub struct ResponceGit {
    /// Absolute path to the git repository workdir.
    pub abspath: String,
    /// Commit hash that HEAD is pointing to. 40 hex digits.
    pub head_commit_hash: [char; 40],
    // TODO: Docs unclear
    /// Local branch name or empty if not on a branch.
    pub local_branch: Option<String>,
    /// Upstream branch name. Can be empty.
    pub upstream_branch: Option<String>,
    /// The remote name, e.g. "upstream" or "origin".
    pub remote_name: String,
    /// Remote URL. Can be empty.
    pub remote_url: Option<String>,
    /// Repository state, A.K.A. action. Can be empty.
    pub repository_state: Option<String>,
    /// The number of files in the index.
    pub num_files_in_index: u32,
    /// The number of staged changes.
    pub num_staged_changes: u32,
    /// The number of unstaged changes.
    pub num_unstaged_changes: u32,
    /// The number of conflicted changes.
    pub num_conflicted_changes: u32,
    /// The number of untracked files.
    pub num_untrached_files: u32,
    /// Number of commits the current branch is ahead of upstream.
    pub commits_ahead: u32,
    /// Number of commits the current branch is behind upstream.
    pub commits_behind: u32,
    /// The number of stashes.
    pub num_stashes: u32,
    /// The last tag (in lexicographical order) that points to the same 
    /// commit as HEAD.
    pub last_tag: String,
    /// The number of unstaged deleted files.
    pub num_unstaged_deleted: u32,
    /// The number of staged new files.
    pub num_staged_new: u32,
    /// The number of staged deleted files.
    pub num_staged_deleted: u32,
    /// The push remote name, e.g. "upstream" or "origin".
    pub push_remote_name: String,
    /// Push remote URL. Can be empty.
    pub push_remote_url: Option<String>,
    /// Number of commits the current branch is ahead of push remote.
    pub commits_ahead_push_remote: u32,
    /// Number of commits the current branch is behind push remote.
    pub commits_behind_push_remote: u32,
    /// Number of files in the index with skip-worktree bit set.
    pub num_index_skip_worktree: u32,
    /// Number of files in the index with assume-unchanged bit set.
    pub num_index_assume_unchanged: u32,
}

pub enum ResponceParseError {}

impl std::str::FromStr for Responce {
    type Err = ResponceParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Request
///////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Hash)]
pub enum ReadIndex {
    ReadAll = 0,
    DontRead = 1,
}

pub struct Request {
    // TODO: Are these always utf-8
    pub id: String,
    pub dir: String,
    pub read_index: ReadIndex,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{id}\x1f{dir}\x1f{index}\x1e",
            id = self.id,
            dir = self.dir,
            index = self.read_index as u8
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Status
///////////////////////////////////////////////////////////////////////////////

pub struct GitStatusd {
    proc: process::Child,
}

impl GitStatusd {
    pub fn new<C: AsRef<OsStr> + Default, P: AsRef<Path>>(
        name: C,
        path: P,
    ) -> io::Result<GitStatusd> {
        Ok(GitStatusd {
            proc: process::Command::new(name)
                .current_dir(path)
                .stdin(process::Stdio::piped())
                .stdout(process::Stdio::piped())
                .stderr(process::Stdio::piped())
                .spawn()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn pickup_gsd() {
        let gsd = GitStatusd::new("./gitstatusd/usrbin/gitstatusd", ".").unwrap();
    }

    #[test]
    fn write_request() {
        let req = Request {
            id: "SomeID".to_owned(),
            dir: "some/path".to_owned(),
            read_index: ReadIndex::ReadAll,
        };
        let to_send = format!("{}", req);
        assert_eq!(to_send, "SomeID\x1fsome/path\x1f0\x1e");

        let req = Request {
            id: "SomeOtherID".to_owned(),
            dir: "some/other/path".to_owned(),
            read_index: ReadIndex::DontRead,
        };
        let to_send = format!("{}", req);
        assert_eq!(to_send, "SomeOtherID\x1fsome/other/path\x1f1\x1e");
    }
}
