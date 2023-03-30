use crate::EnvPath;
use core::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

/// This implementation allows for mutable access to the underlying path value of `EnvPath`.
impl<'r> DerefMut for EnvPath<'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.path {
            Some(p) => p,
            n => {
                *n = Some(PathBuf::new());
                n.as_mut()
                    .expect("Failed to deref mut EnvPath.")
            }
        }
    }
}

/// This implementation provides a read-only reference to the underlying path value of `EnvPath`.
impl<'r> Deref for EnvPath<'r> {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self.path {
            Some(ref p) => p.as_path(),
            _ => Path::new(""),
        }
    }
}
