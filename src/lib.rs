/*!
Is there an executable file at the given path?

A small helper function which determines whether or not the given path points to
an executable file. If there is no file at the given path, or the file is not
executable, then `false` is returned. When there is a file and the file is
executable, then `true` is returend.

Answering this question is OS specific: some operating systems (Windows) do not
distinguish between executable and non-executable file permissions. On such
OSes, if there is a file at the given path, then `true` is returned.

The API comes in two flavors:

1. An extension trait to add an `is_executable` method on `std::path::Path` and
   `std::fs::Permissions`:

   ```rust
   use is_executable::IsExecutable;
   use std::fs;
   use std::path::Path;

   let path = Path::new("some/path/to/a/file");

   // Determine if `path` is executable.
   if path.is_executable() {
       println!("The path is executable!");
   } else {
       println!("The path is _not_ executable!");
   }

   // Determine if some `std::fs::Metadata`'s `std::fs::Permissions` are
   // executable.
   # let _foo = || -> ::std::io::Result<()> {
   if fs::metadata("some/path")?.permissions().is_executable() {
       println!("The permissions are executable!");
   } else {
       println!("The permissions are _not_ executable!");
   }
   # Ok(())
   # };
   ```

2. For convenience, a standalone `is_executable` function, which takes any
`AsRef<Path>`:

   ```rust
   use is_executable::is_executable;
   use std::path::Path;

   let path = Path::new("some/path/to/a/file");

   if is_executable(&path) {
       println!("The path is executable!");
   } else {
       println!("The path is _not_ executable!");
   }
   ```
 */

use std::fs;
use std::path::Path;

/// Returns `true` if there is a file at the given path and it is
/// executable. Returns `false` otherwise.
///
/// See the module documentation for details.
pub fn is_executable<P: AsRef<Path>>(path: P) -> bool {
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

impl IsExecutable for Path {
    fn is_executable(&self) -> bool {
        fs::metadata(self)
            .ok()
            .map_or(false, |meta| meta.permissions().is_executable())
    }
}

impl IsExecutable for fs::Permissions {
    #[cfg(unix)]
    fn is_executable(&self) -> bool {
        use std::os::unix::fs::PermissionsExt;
        self.mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    fn is_executable(&self) -> bool {
        true
    }
}
