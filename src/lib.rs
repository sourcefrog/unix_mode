// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Manipulate Unix file mode bits.
//!
//! Every filesystem entry (or inode) on Unix has a bit field of
//! [mode bits](https://en.wikipedia.org/wiki/Modes_(Unix))
//! that describe both the type of the file and its permissions.
//!
//! These are classically displayed in the left of `ls` output, and the permissions
//! can be changed with `chmod`. For example:
//!
//! ```
//! assert_eq!(unix_mode::to_string(0o0040755), "drwxr-xr-x");
//! assert_eq!(unix_mode::to_string(0o0100640), "-rw-r-----");
//! ```
//!
//! The encoding is fairly standard across unices, and occurs in some file
//! formats and network protocols that might be seen on non-Unix platforms.
//!
//! This library isn't Unix-specific and doesn't depend on the underlying OS to
//! interpret the bits.
//!
//! For example, this can be used with the return value from
//! `std::os::unix::fs::MetadataExt::mode()`.
//!
//! The names of the predicate functions match `std::fs::FileType` and
//! `std::os::unix::fs::FileTypeExt`.

/// Return just the bits representing the type of file.
fn type_bits(mode: u32) -> u32 {
    (mode >> 12) & 0o17
}

/// The different types of files known to this library
///
/// Can be constructed `From<u32>`.
/// ```
/// assert_eq!(unix_mode::Type::from(0o0100640), unix_mode::Type::File);
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Type {
    File,
    Dir,
    Symlink,
    Socket,
    Fifo,
    BlockDevice,
    CharDevice,
    /// Removed file in union filesystems
    Whiteout,
    /// File type not recognized by this version of this library
    ///
    /// More types might be added in the future, so the semantics of this variant may change.
    Unknown,
}

impl From<u32> for Type {
    /// Parse type from mode
    ///
    fn from(mode: u32) -> Type {
        use Type::*;
        match type_bits(mode) {
            0o001 => Fifo,
            0o002 => CharDevice,
            0o004 => Dir,
            0o006 => BlockDevice,
            0o010 => File,
            0o012 => Symlink,
            0o014 => Socket,
            0o016 => Whiteout,
            _ => Unknown,
        }
    }
}

/// Enum for specifying the context / "who" accesses in [is_allowed]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Accessor {
    Other,
    Group,
    User,
}

/// Enum for specifying the type of access in [is_allowed]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Access {
    /// (Beware: execute has various meanings depending on the type of file)
    Execute,
    Write,
    Read,
}

/// Check whether `mode` represents an allowed (`true`) or denied (`false`) access
pub fn is_allowed(by: Accessor, ty: Access, mode: u32) -> bool {
    use Access::*;
    use Accessor::*;
    let by = match by {
        User => 2,
        Group => 1,
        Other => 0,
    };
    let bits = (mode >> (3 * by)) & 0o7;
    let ty = match ty {
        Read => 2,
        Write => 1,
        Execute => 0,
    };
    bits & (1 << ty) != 0
}

/// Returns true if this mode represents a regular file.
///
/// ```
/// assert_eq!(unix_mode::is_file(0o0041777), false);
/// assert_eq!(unix_mode::is_file(0o0100640), true);
/// ```
pub fn is_file(mode: u32) -> bool {
    Type::from(mode) == Type::File
}

/// Returns true if this mode represents a directory.
///
/// ```
/// assert_eq!(unix_mode::is_dir(0o0041777), true);
/// assert_eq!(unix_mode::is_dir(0o0100640), false);
/// ```
pub fn is_dir(mode: u32) -> bool {
    Type::from(mode) == Type::Dir
}

/// Returns true if this mode represents a symlink.
///
/// ```
/// assert_eq!(unix_mode::is_symlink(0o0040755), false);
/// assert_eq!(unix_mode::is_symlink(0o0120755), true);
/// ```
pub fn is_symlink(mode: u32) -> bool {
    Type::from(mode) == Type::Symlink
}

/// Returns true if this mode represents a fifo, also known as a named pipe.
pub fn is_fifo(mode: u32) -> bool {
    Type::from(mode) == Type::Fifo
}

/// Returns true if this mode represents a character device.
pub fn is_char_device(mode: u32) -> bool {
    Type::from(mode) == Type::CharDevice
}

/// Returns true if this mode represents a block device.
pub fn is_block_device(mode: u32) -> bool {
    Type::from(mode) == Type::BlockDevice
}

/// Returns true if this mode represents a Unix-domain socket.
pub fn is_socket(mode: u32) -> bool {
    Type::from(mode) == Type::Socket
}

/// Returns true if the set-user-ID bit is set
pub fn is_setuid(mode: u32) -> bool {
    mode & 0o4000 != 0
}

/// Returns true if the set-group-ID bit is set
pub fn is_setgid(mode: u32) -> bool {
    mode & 0o2000 != 0
}

/// Returns true if the sticky bit is set
pub fn is_sticky(mode: u32) -> bool {
    mode & 0o1000 != 0
}

/// Convert Unix mode bits to a text string describing type and permissions,
/// as shown in `ls`.
///
/// Examples:
/// ```
/// assert_eq!(unix_mode::to_string(0o0040755), "drwxr-xr-x");
/// assert_eq!(unix_mode::to_string(0o0100640), "-rw-r-----");
///
/// // Classic "sticky" directory
/// assert_eq!(unix_mode::to_string(0o0041777), "drwxrwxrwt");
///
/// // Char and block devices
/// assert_eq!(unix_mode::to_string(0o0020600), "crw-------");
/// assert_eq!(unix_mode::to_string(0o0060600), "brw-------");
///
/// // Symlink
/// assert_eq!(unix_mode::to_string(0o0120777), "lrwxrwxrwx");
///
/// ```
pub fn to_string(mode: u32) -> String {
    // This is decoded "by hand" here so that it'll work
    // on non-Unix platforms.
    use Access::*;
    use Accessor::*;
    use Type::*;

    let setuid = is_setuid(mode);
    let setgid = is_setgid(mode);
    let sticky = is_sticky(mode);

    let mut s = String::with_capacity(10);
    s.push(match Type::from(mode) {
        Fifo => 'p',
        CharDevice => 'c',
        Dir => 'd',
        BlockDevice => 'b',
        File => '-',
        Symlink => 'l',
        Socket => 's',
        Whiteout => 'w',
        Unknown => '?',
    });
    for accessor in [User, Group, Other] {
        for access in [Read, Write, Execute] {
            s.push(
                match (access, accessor, is_allowed(accessor, access, mode)) {
                    (Execute, User, true) if setuid => 's',
                    (Execute, User, false) if setuid => 'S',
                    (Execute, Group, true) if setgid => 's',
                    (Execute, Group, false) if setgid => 'S',
                    (Execute, Other, true) if sticky => 't',
                    (Execute, Other, false) if sticky => 'T',
                    (Execute, _, true) => 'x',
                    (Write, _, true) => 'w',
                    (Read, _, true) => 'r',
                    (_, _, false) => '-',
                },
            );
        }
    }
    s
}

#[cfg(unix)]
#[cfg(test)]
mod unix_tests {
    use super::*;
    use nix::sys::stat;
    use nix::unistd;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::net::UnixListener;
    use std::path::Path;
    use tempfile::tempdir;

    fn file_mode<S: AsRef<Path>>(s: S) -> u32 {
        let mode = std::fs::symlink_metadata(s.as_ref()).unwrap().mode();
        println!("Mode of {:?} is 0o{:07o}", s.as_ref(), mode);
        mode
    }

    /// Test predicates against files likely to already exist on a Unix system.
    #[test]
    fn stat_existing_files() {
        assert!(is_dir(file_mode("/")));
        assert!(!is_file(file_mode("/")));
        assert!(is_file(file_mode("/etc/passwd")));
        assert!(is_char_device(file_mode("/dev/null")));
        assert!(is_sticky(file_mode("/tmp/")));

        // I don't know how to reliably find a block device across OSes, and
        // we can't make one (without root.)
    }

    /// Test [is_allowed] against files likely to already exist on a Unix system.
    #[test]
    fn existing_file_perms() {
        use Access::*;
        use Accessor::*;
        for by in [User, Group, Other] {
            assert!(is_allowed(by, Read, file_mode("/")));
            assert!(is_allowed(by, Execute, file_mode("/")));
            assert!(is_allowed(by, Write, file_mode("/dev/null")));
        }
        assert!(!is_allowed(Other, Write, file_mode("/dev/")));
        assert!(!is_allowed(Other, Execute, file_mode("/dev/null")));
    }

    #[test]
    fn stat_created_symlink() {
        let tmp_dir = tempdir().unwrap();
        let link_path = tmp_dir.path().join("sym");
        unistd::symlinkat(".", None, &link_path).unwrap();
        assert!(is_symlink(file_mode(link_path)));
    }

    #[test]
    fn stat_created_fifo() {
        let tmp_dir = tempdir().unwrap();
        let fifo_path = tmp_dir.path().join("fifo");
        unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU).unwrap();
        assert!(is_fifo(file_mode(fifo_path)));
    }

    #[test]
    fn stat_created_socket() {
        let tmp_dir = tempdir().unwrap();
        let sock_path = tmp_dir.path().join("sock");
        let _ = UnixListener::bind(&sock_path).unwrap();
        assert!(is_socket(file_mode(sock_path)));
    }
}
