// Copyright 2020 Google LLC
// Copyright 2022 Martin Pool
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

//! Tests that run only on native Unix platforms, and that check this library's behavior against the external OS.

#![cfg(unix)]

use nix::sys::stat;
use nix::unistd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixListener;
use std::path::Path;
use tempfile::tempdir;
use unix_mode::*;

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

mod to_string {
    use super::*;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    fn shells(chmod_to: &str, expect_mode: &str) {
        let tmp_dir = tempdir().unwrap();
        // We're gonna be mucking around with setuid files, so exercise a little bit of caution
        std::fs::set_permissions(tmp_dir.path(), Permissions::from_mode(0o700)).unwrap();
        let f = &tmp_dir.path().join("f");
        std::fs::write(f, [0]).unwrap();
        std::fs::set_permissions(f, Permissions::from_mode(0o0)).unwrap();
        let chmod = Command::new("chmod").arg(chmod_to).arg(f).output().unwrap();
        println!("chmod {:#?}", chmod);
        assert_eq!(to_string(file_mode(f)), expect_mode);
        // For good measure, also compare against ls
        let ls = Command::new("ls").arg("-l").arg(f).output().unwrap();
        println!("{:#?}", ls);
        assert_eq!(std::str::from_utf8(&ls.stdout[0..10]), Ok(expect_mode));
    }

    #[test]
    fn rwx() {
        shells("a+r", "-r--r--r--");
        shells("a+w", "--w--w--w-");
        shells("a+x", "---x--x--x");
    }

    #[test]
    fn extrabits() {
        shells("+t", "---------T");
        shells("+xt", "---x--x--t");
        shells("+s", "---S--S---");
        shells("+xs", "---s--s--x");
    }

    #[test]
    fn nothing_with_left_beef() {
        shells("u+wx,g+r", "--wxr-----");
    }
}
