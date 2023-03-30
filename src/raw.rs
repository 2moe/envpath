use crate::{parser::parse, EnvPath};
use std::{borrow::Cow, path::PathBuf};

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum EnvPathRaw<'r> {
    Ref(Vec<&'r str>),
    Cow(Vec<Cow<'r, str>>),
    Owned(Vec<String>),
}

impl<'r> EnvPathRaw<'r> {
    pub fn is_empty(&self) -> bool {
        use EnvPathRaw::*;
        match self {
            Ref(x) => x.is_empty(),
            Cow(x) => x.is_empty(),
            Owned(x) => x.is_empty(),
        }
    }
    pub fn parse(&self) -> Option<PathBuf> {
        use EnvPathRaw::*;
        match self {
            Ref(x) => parse(x),
            Cow(x) => parse(x),
            Owned(x) => parse(x),
        }
    }
}

impl<'r> Default for EnvPathRaw<'r> {
    fn default() -> Self {
        EnvPathRaw::Ref(Vec::new())
    }
}

impl<'r> EnvPath<'r> {
    /// Get a reference to the raw sequence of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::from(["$env: home ?? userprofile", "3D Print"]);
    /// dbg!(path.get_raw());
    /// ```
    // pub fn get_raw(&self) -> &[&str] {
    //     self.raw.as_ref()
    // }
    pub fn get_raw(&self) -> &EnvPathRaw {
        &self.raw
    }

    /// `get_raw_mut` is a public method of the `EnvPath` struct that returns a mutable reference to the raw sequence of strings.
    ///
    /// This method can be used to modify the raw sequence and update the `EnvPath` object accordingly. It takes no arguments and returns a mutable reference to an `EnvPathRaw` object.
    pub fn get_raw_mut(&mut self) -> &mut EnvPathRaw<'r> {
        &mut self.raw
    }

    /// Set the raw sequence of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let mut path = EnvPath::from(["$dir: cfg", "config.ron"]);
    /// dbg!(path.get_raw());
    ///
    /// path.set_raw(vec!["$project( com. x. y ): cfg", "config.toml"]);
    /// dbg!(path.get_raw());
    ///
    /// path.set_raw([" $dir:  bin ?? first-path  "]);
    /// dbg!(path.de().display());
    /// ```
    pub fn set_raw<V: IntoIterator<Item = &'r str>>(&mut self, raw: V) {
        self.raw = Self::create_ref_raw(raw);
    }

    /// Clear the raw sequence of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let mut path =
    /// EnvPath::from(["$env: xdg_data_home", "$const: pkg", "files"]);
    ///
    /// path.clear_raw();
    ///
    /// assert!(path.get_raw().is_empty());
    /// ```
    pub fn clear_raw(&mut self) {
        self.raw = EnvPathRaw::Ref(Vec::new());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn clear_raw_doc() {
        let mut path =
            EnvPath::new(vec!["$env: xdg_data_home", "$const: pkg", "files"]);

        path.clear_raw();

        assert!(!path.exists());
    }
}
