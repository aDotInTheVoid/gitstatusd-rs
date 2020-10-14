use std::{ffi::OsStr, fmt, io, path::Path, process};

///////////////////////////////////////////////////////////////////////////////
// Responce
///////////////////////////////////////////////////////////////////////////////

pub struct Responce {
    /// Request id. The same as the first field in the request.
    pub id: String,
    /// The inner responce.
    pub inner: ResponceInner,
}

/// Most of a responce, depending on if we were in git.
pub enum ResponceInner {
    /// We arn't in git
    NotGit,
    /// We're in it, details inside.
    Git(ResponceGit),
}

/// Details about git state.
///
/// Note: Renamed files are reported as deleted plus new.
pub struct ResponceGit {
    /// Absolute path to the git repository workdir.
    pub abspath: String,
    /// Commit hash that HEAD is pointing to. 40 hex digits.
    // TODO: Change the type
    pub head_commit_hash: String,
    // TODO: Docs unclear
    /// Local branch name or empty if not on a branch.
    pub local_branch: String,
    /// Upstream branch name. Can be empty.
    pub upstream_branch: String,
    /// The remote name, e.g. "upstream" or "origin".
    pub remote_name: String,
    /// Remote URL. Can be empty.
    pub remote_url: String,
    /// Repository state, A.K.A. action. Can be empty.
    pub repository_state: String,
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
    pub push_remote_url: String,
    /// Number of commits the current branch is ahead of push remote.
    pub commits_ahead_push_remote: u32,
    /// Number of commits the current branch is behind push remote.
    pub commits_behind_push_remote: u32,
    /// Number of files in the index with skip-worktree bit set.
    pub num_index_skip_worktree: u32,
    /// Number of files in the index with assume-unchanged bit set.
    pub num_index_assume_unchanged: u32,
}

pub enum ResponceParseError {
    /// Not Enought Parts were recieved
    TooShort,
    /// A part was sent, but we cant parse it.
    InvalidPart,
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for ResponceParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

macro_rules! munch {
    ($expr:expr) => {
        match $expr.next() {
            Some(v) => v,
            None => return Err($crate::ResponceParseError::TooShort),
        }
    };
}

impl std::str::FromStr for Responce {
    type Err = ResponceParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("\x1f");
        let id = munch!(parts);
        let is_repo = munch!(parts);
        match is_repo {
            "0" => {
                return Ok(Responce {
                    id: id.to_owned(),
                    inner: ResponceInner::NotGit,
                })
            }
            // 1 indicated a git repo, so we do the real stuff
            "1" => {}
            // If not 0 or 1, give up
            _ => return Err(ResponceParseError::InvalidPart),
        }

        let mut abspath = munch!(parts);
        let head_commit_hash = munch!(parts);
        let local_branch = munch!(parts);
        let upstream_branch = munch!(parts);
        let remote_name = munch!(parts);
        let remote_url = munch!(parts);
        let repository_state = munch!(parts);

        let num_files_in_index: u32 = munch!(parts).parse()?;
        let num_staged_changes: u32 = munch!(parts).parse()?;
        let num_unstaged_changes: u32 = munch!(parts).parse()?;
        let num_conflicted_changes: u32 = munch!(parts).parse()?;
        let num_untrached_files: u32 = munch!(parts).parse()?;
        let commits_ahead: u32 = munch!(parts).parse()?;
        let commits_behind: u32 = munch!(parts).parse()?;
        let num_stashes: u32 = munch!(parts).parse()?;
        let last_tag = munch!(parts);
        let num_unstaged_deleted: u32 = munch!(parts).parse()?;
        let num_staged_new: u32 = munch!(parts).parse()?;
        let num_staged_deleted: u32 = munch!(parts).parse()?;
        let push_remote_name = munch!(parts);
        let push_remote_url: &str = munch!(parts);
        let commits_ahead_push_remote: u32 = munch!(parts).parse()?;
        let commits_behind_push_remote: u32 = munch!(parts).parse()?;
        let num_index_skip_worktree: u32 = munch!(parts).parse()?;
        let num_index_assume_unchanged: u32 = munch!(parts).parse()?;

        // Only do ownership once we have all the stuff
        let git_part = ResponceGit {
            abspath: abspath.to_owned(),
            head_commit_hash: head_commit_hash.to_owned(),
            local_branch: local_branch.to_owned(),
            upstream_branch: upstream_branch.to_owned(),
            remote_name: remote_name.to_owned(),
            remote_url: remote_url.to_owned(),
            repository_state: repository_state.to_owned(),
            num_files_in_index,
            num_staged_changes,
            num_unstaged_changes,
            num_conflicted_changes,
            num_untrached_files,
            commits_ahead,
            commits_behind,
            num_stashes,
            last_tag: last_tag.to_owned(),
            num_unstaged_deleted,
            num_staged_new,
            num_staged_deleted,
            push_remote_name: push_remote_name.to_owned(),
            push_remote_url: push_remote_url.to_owned(),
            commits_ahead_push_remote,
            commits_behind_push_remote,
            num_index_skip_worktree,
            num_index_assume_unchanged,
        };

        Ok(Responce {
            id: id.to_owned(),
            inner: ResponceInner::Git(git_part),
        })
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
