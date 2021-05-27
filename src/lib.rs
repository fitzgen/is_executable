/*!
Is there an executable file at the given path?

[![](https://docs.rs/is_executable/badge.svg)](https://docs.rs/is_executable/) [![](http://meritbadge.herokuapp.com/is_executable) ![](https://img.shields.io/crates/d/is_executable.png)](https://crates.io/crates/is_executable) [![Unix Build Status](https://travis-ci.org/fitzgen/is_executable.png?branch=master)](https://travis-ci.org/fitzgen/is_executable) [![Windows Build Status](https://ci.appveyor.com/api/projects/status/github/fitzgen/is_executable?branch=master&svg=true)](https://ci.appveyor.com/project/fitzgen/is-executable)

A small helper function which determines whether or not the given path points to
an executable file. If there is no file at the given path, or the file is not
executable, then `false` is returned. When there is a file and the file is
executable, then `true` is returned.

This crate works on both unix-based operating systems (mac, linux, freebsd, etc.) and Windows.

The API comes in two flavors:

1. An extension trait to add an `is_executable` method on `std::path::Path`:

    ```rust
    use std::path::Path;

    use is_executable::IsExecutable;

    fn main() {
        let path = Path::new("some/path/to/a/file");

        // Determine if `path` is executable.
        if path.is_executable() {
            println!("The path is executable!");
        } else {
            println!("The path is _not_ executable!");
        }
    }
    ```

2. For convenience, a standalone `is_executable` function, which takes any
`AsRef<Path>`:

    ```rust
    use std::path::Path;

    use is_executable::is_executable;

    fn main() {
        let path = Path::new("some/path/to/a/file");

        // Determine if `path` is executable.
        if is_executable(&path) {
            println!("The path is executable!");
        } else {
            println!("The path is _not_ executable!");
        }
    }
    ```
 */

#[cfg(target_os = "windows")]
extern crate winapi;

use std::io;
use std::env;
use std::os;
use std::path::Path;

/// Returns `true` if there is a file at the given path and it is
/// executable. Returns `false` otherwise.
///
/// See the module documentation for details.
pub fn is_executable<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    path.as_ref().is_executable()
}



/// An extension trait for `std::fs::Path` providing an `is_executable` method.
///
/// See the module documentation for examples.
pub trait IsExecutable {
    /// Returns `true` if there is a file at the given path and it is
    /// executable. Returns `false` otherwise.
    ///
    /// See the module documentation for details.
    fn is_executable(&self) -> bool;
}

/// Returns `Result<Path, io::Error>` if there is a file at the given path and the
/// current run-level is permitted to execute it.
///
/// See the module documentation for details.
pub fn is_permitted<P>(path: P) -> io::Result<Path>
where
    P: AsRef<Path>
{
    path.as_ref().is_permitted()
}

/// An extension trait for `std::fs::Path` providing an `is_permitted` method.
///
/// See the module documentation for examples.
pub trait IsPermitted {
    /// Returns `Result<Path, io::Error>` that describes if a particular file
    /// exists at the given path and the run-level of the current context meets
    /// the appropriate user, group, admin, root/system-level membership.
    ///
    /// Note: *this does not inspect whether the `Path` is executable.*
    fn is_permitted(&self) -> io::Result<Path>;
}

#[cfg(unix)]
mod unix {
    use std::os::unix::fs::MetadataExt;
    use users::{
        Group, Groups,
        User, Users,
        group_access_list,
        get_effective_uid,
        get_group_by_gid
    };
    use std::path::Path;
    use std::fs;
    use std::io;
    use super::IsExecutable;
    use super::IsPermitted;

    impl IsExecutable for Path {
        fn is_executable(&self) -> bool {
            let metadata = match self.metadata() {
                Ok(metadata) => metadata,
                Err(_) => return false,
            };
            let permissions = metadata.permissions();
            metadata.is_file() && permissions.mode() & 0o111 != 0
        }
    }

    macro_rules! vecomp {
    [$expr:expr; for $pat:pat in $iter:expr $(; if $cond:expr )?] => {
        IntoIterator::into_iter($iter)
            $(
                .filter(|$pat| $cond)
            )?
            .map(|$pat| $expr)
            .collect::<Vec<_>>()
        }
    }

    impl IsPermitted for Path {
        fn is_permitted(&self) -> io::Result<Path> {
            if let Ok(metadata) = self.borrow().metadata() {
                /** If the metadata's mode's octal bits match any
                 * one of the combinations listed below, then we
                 * have sufficient reasons to believe it's executable
                 */
                match metadata.borrow().is_file() {
                    true => {
                        /** Could this target path be ran with executable permissions
                         *  by this runtime's run-level user?
                         *
                         *  Let's find out.
                         */
                        
                         },
                    false => {
                        io::Error::new(io::ErrorKind::NotFound,
                            format!("{:?} was not readable or present on the local filesystem",
                                self.to_string()))
                    }
                }
            } else {
                io::Error::last_os_error()
            }
        }
    }

    impl From<io::Result<Path>> for AsRef<Path>
}

#[cfg(target_os = "windows")]
mod windows {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use winapi::ctypes::{c_ulong, wchar_t};
    use winapi::um::winbase::GetBinaryTypeW;

    use super::IsExecutable;

    impl IsExecutable for Path {
        fn is_executable(&self) -> bool {
            // Check using file extension
            if let Some(pathext) = std::env::var_os("PATHEXT") {
                if let Some(extension) = self.extension() {
                    // Restructure pathext as Vec<String>
                    // https://github.com/nushell/nushell/blob/93e8f6c05e1e1187d5b674d6b633deb839c84899/crates/nu-cli/src/completion/command.rs#L64-L74
                    let pathext = pathext
                        .to_string_lossy()
                        .split(';')
                        // Filter out empty tokens and ';' at the end
                        .filter(|f| f.len() > 1)
                        // Cut off the leading '.' character
                        .map(|ext| ext[1..].to_string())
                        .collect::<Vec<_>>();
                    let extension = extension.to_string_lossy();

                    return pathext
                        .iter()
                        .any(|ext| extension.eq_ignore_ascii_case(ext));
                }
            }

            // Check using file properties
            // This code is only reached if there is no file extension or retrieving PATHEXT fails
            let windows_string = self
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<wchar_t>>();
            let windows_string_ptr = windows_string.as_ptr();

            let mut binary_type: c_ulong = 42;
            let binary_type_ptr = &mut binary_type as *mut c_ulong;

            let ret = unsafe { GetBinaryTypeW(windows_string_ptr, binary_type_ptr) };
            if binary_type_ptr.is_null() {
                return false;
            }
            if ret != 0 {
                let binary_type = unsafe { *binary_type_ptr };
                match binary_type {
                    0   // A 32-bit Windows-based application
                    | 1 // An MS-DOS-based application
                    | 2 // A 16-bit Windows-based application
                    | 3 // A PIF file that executes an MS-DOS-based application
                    | 4 // A POSIX – based application
                    | 5 // A 16-bit OS/2-based application
                    | 6 // A 64-bit Windows-based application
                    => return true,
                    _ => (),
                }
            }

            false
        }
    }
}
