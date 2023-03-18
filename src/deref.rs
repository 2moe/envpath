use crate::EnvPath;
use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// This implementation allows for mutable access to the underlying path value of `EnvPath`.
impl DerefMut for EnvPath {
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
impl Deref for EnvPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self.path {
            Some(ref p) => p.as_path(),
            _ => Path::new(""),
        }
    }
}
