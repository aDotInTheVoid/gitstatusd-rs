//TODO: Serde Support
//TODO: Clean up docs
//TODO: Build instructions

use std::{
    ffi::OsStr,
    fmt,
    io::{self, BufRead, Write},
    path::Path,
    process,
};

///////////////////////////////////////////////////////////////////////////////
// Responce
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
pub struct Responce {
    /// Request id. The same as the first field in the request.
    pub id: String,
    /// The inner responce.
    pub inner: ResponceInner,
}

/// Most of a responce, depending on if we were in git.
#[derive(Debug, PartialEq)]
pub enum ResponceInner {
    /// We arn't in git
    NotGit,
    /// We're in it, details inside.
    Git(ResponceGit),
}

/// Details about git state.
///
/// Note: Renamed files are reported as deleted plus new.
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
        Responce::from_str(s)
    }
}

impl Responce {
    // TODO: Make this run on &[u8]
    fn from_str(s: &str) -> Result<Self, ResponceParseError> {
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

        let abspath = munch!(parts);
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
    // TODO: borrow these
    pub id: String,
    pub dir: String,
    pub read_index: ReadIndex,
}

// TODO, this should probably work for non utf8.
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
    // I need to store the child so it's pipes don't close
    _proc: process::Child,
    stdin: process::ChildStdin,
    stdout: io::BufReader<process::ChildStdout>,
    // TODO: decide if I need this
    _stderr: process::ChildStderr,
}

impl GitStatusd {
    // TODO: does the path matter
    // TODO: binary detection
    pub fn new<C: AsRef<OsStr> + Default, P: AsRef<Path>>(
        name: C,
        path: P,
    ) -> io::Result<GitStatusd> {
        let mut proc = process::Command::new(name)
            .current_dir(path)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;

        let stdin = proc.stdin.take().ok_or(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Couldn't obtain stdin",
        ))?;
        let stdout = io::BufReader::new(proc.stdout.take().ok_or(
            io::Error::new(io::ErrorKind::BrokenPipe, "Couldn't obtain stdout"),
        )?);
        let stderr = proc.stderr.take().ok_or(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Couldn't obtain stderr",
        ))?;

        Ok(GitStatusd {
            _proc: proc,
            stdin,
            stdout,
            _stderr: stderr,
        })
    }

    //TODO: Better Error Handling
    //TODO: Non blocking version
    //TODO: Id generation
    pub fn request(&mut self, r: Request) -> io::Result<Responce> {
        write!(self.stdin, "{}", r)?;
        let mut read = Vec::with_capacity(256);
        self.stdout.read_until(0x1e, &mut read)?;
        assert_eq!(read.last(), Some(&0x1e));
        // Drop the controll byte
        read.truncate(read.len().saturating_sub(1));

        // TODO: Handle error
        // TODO: Check the id's are the same.
        let read = String::from_utf8(read).unwrap();
        let responce = Responce::from_str(&read).unwrap();
        Ok(responce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn pickup_gsd() {
        let gsd = GitStatusd::new("./gitstatusd/usrbin/gitstatusd", ".");

        assert!(gsd.is_ok());
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

    #[test]
    fn parse_responce_no_git() {
        let resp1 = "id1\x1f0";
        let r1p = resp1.parse();
        assert_eq!(
            r1p,
            Ok(Responce {
                id: "id1".to_owned(),
                inner: ResponceInner::NotGit,
            })
        );
    }

    fn responce_test(s: &str, resp: Result<Responce, ResponceParseError>) {
        let r_got = s.parse();
        assert_eq!(r_got, resp);
    }

    #[test]
    fn parse_responce_no_git_no_id() {
        responce_test(
            "\x1f0",
            Ok(Responce {
                id: "".to_owned(),
                inner: ResponceInner::NotGit,
            }),
        );
    }

    #[test]
    fn parse_responce_empty() {
        responce_test("", Err(ResponceParseError::TooShort));
    }

    #[test]
    fn parse_responce_git_full() {
        responce_test(
            "id\u{1f}1\u{1f}/Users/nixon/dev/rs/gitstatusd\u{1f}1c9be4fe5460a30e70de9cbf99c3ec7064296b28\u{1f}master\u{1f}\u{1f}\u{1f}\u{1f}\u{1f}7\u{1f}0\u{1f}1\u{1f}0\u{1f}1\u{1f}0\u{1f}0\u{1f}0\u{1f}\u{1f}0\u{1f}0\u{1f}0\u{1f}\u{1f}\u{1f}0\u{1f}0\u{1f}0\u{1f}0",
            Ok(Responce {
                id: "id".to_owned(),
                inner: ResponceInner::Git(ResponceGit{
                    abspath: "/Users/nixon/dev/rs/gitstatusd".to_owned(),
                    head_commit_hash: "1c9be4fe5460a30e70de9cbf99c3ec7064296b28".to_owned(),
                    local_branch: "master".to_owned(),
                    upstream_branch: "".to_owned(),
                    remote_name: "".to_owned(),
                    remote_url: "".to_owned(),
                    repository_state: "".to_owned(),
                    num_files_in_index: 7,
                    num_staged_changes: 0,
                    num_unstaged_changes: 1,
                    num_conflicted_changes: 0,
                    num_untrached_files: 1,
                    commits_ahead: 0,
                    commits_behind: 0,
                    num_stashes: 0,
                    last_tag: "".to_owned(),
                    num_unstaged_deleted: 0,
                    num_staged_new: 0,
                    num_staged_deleted: 0,
                    push_remote_name: "".to_owned(),
                    push_remote_url: "".to_owned(),
                    commits_ahead_push_remote: 0,
                    commits_behind_push_remote: 0,
                    num_index_skip_worktree: 0,
                    num_index_assume_unchanged: 0,
                })
            })
        );
    }

    #[test]
    fn run_this_dir_is_git() {
        let req = Request {
            id: "Request1".to_owned(),
            dir: env!("CARGO_MANIFEST_DIR").to_owned(),
            read_index: ReadIndex::ReadAll,
        };
        let mut gsd =
            GitStatusd::new("./gitstatusd/usrbin/gitstatusd", ".").unwrap();
        let responce = gsd.request(req).unwrap();
        assert!(matches!(responce.inner, ResponceInner::Git(_)));
    }
}
