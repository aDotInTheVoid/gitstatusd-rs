use std::{ffi::OsStr, io, path::Path, process, fmt};

const REQ_SEP: u8 = 30;
const FIELD_SEP: u8 = 31;


pub struct Status {}

#[derive(Copy, Clone, Debug, Hash)]
pub enum ReadIndex {
    ReadAll = 0,
    DontRead = 1
}

pub struct Request {
    // TODO: Are these always utf-8
    pub id: String,
    pub dir: String,
    pub read_index: ReadIndex
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{id}\x1f{dir}\x1f{index}\x1e", id=self.id, dir=self.dir, index=self.read_index as u8)
    }
}

pub struct GitStatusd {
    proc: process::Child,
}



impl GitStatusd {
    pub fn new<C: AsRef<OsStr> + Default, P: AsRef<Path>>(name: C, path: P) -> io::Result<GitStatusd> {
        Ok(GitStatusd {
            proc: process::Command::new(name).current_dir(path)
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
            read_index: ReadIndex::ReadAll
        };
        let to_send = format!("{}", req);
        assert_eq!(to_send, "SomeID\x1fsome/path\x1f0\x1e");


        let req = Request {
            id: "SomeOtherID".to_owned(),
            dir: "some/other/path".to_owned(),
            read_index: ReadIndex::DontRead
        };
        let to_send = format!("{}", req);
        assert_eq!(to_send, "SomeOtherID\x1fsome/other/path\x1f1\x1e");
    }
}
