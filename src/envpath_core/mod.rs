mod from;

use crate::OsCow;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub(crate) type EnvPathRaw = Vec<String>;

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnvPath {
    pub(crate) raw: EnvPathRaw,
    pub(crate) path: Option<PathBuf>,
}

impl EnvPath {
    /// Get a reference to the raw sequence of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::from(["$env: home ?? userprofile", "3D Print"]);
    /// assert_eq!(path.get_raw(), &["$env: home ?? userprofile", "3D Print"]);
    /// ```
    pub fn get_raw(&self) -> &[String] {
        self.raw.as_ref()
    }
    //

    /// Set the raw sequence of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let mut path = EnvPath::from(["$dir: cfg", "config.ron"]);
    /// assert_eq!(path.get_raw(), &["$dir: cfg", "config.ron"]);
    ///
    /// path.set_raw(vec!["$project( com. x. y ): cfg", "config.toml"]);
    /// assert_eq!(path.get_raw(), &["$project( com. x. y ): cfg", "config.toml"]);
    ///
    /// path.set_raw(vec![" $dir:  bin ?? first-path  "]);
    /// dbg!(path.de().display());
    /// ```
    pub fn set_raw<S: Into<String>>(&mut self, raw: Vec<S>) {
        self.raw = Self::new_raw(raw);
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
        self.raw = Vec::new();
    }

    /// Join a specific string with the SD directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::EnvPath;
    ///
    /// let android_dir = EnvPath::set_android_dir("Android/obb/com.x.y/");
    ///
    /// assert_eq!(
    ///     android_dir,
    ///     Some(std::borrow::Cow::from(OsStr::new(
    ///         "/storage/self/primary/Android/obb/com.x.y/"
    ///     )))
    /// );
    /// ```
    #[cfg(target_os = "android")]
    pub(crate) fn set_android_dir(s: &str) -> OsCow {
        const SD: &str = "/storage/self/primary";
        Self::into_os_cow(std::path::Path::new(SD).join(s))
    }

    /// Converts the given string into an `OsCow` object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::EnvPath;
    /// use std::{borrow::Cow, ffi::OsStr};
    ///
    /// let str = "/usr/bin";
    /// let os_cow = EnvPath::os_cow(str);
    ///
    /// assert_eq!(os_cow, Some(Cow::from(OsStr::new(str))));
    /// ```
    pub(crate) fn os_cow(s: &str) -> OsCow {
        Some(Cow::from(OsStr::new(s)))
    }

    /// Converts the given Path/OsStr into an `OsCow` object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::EnvPath;
    /// use std::{
    ///     borrow::Cow,
    ///     path::{Path, PathBuf},
    /// };
    ///
    /// let path = Path::new("/usr/bin");
    /// let os_cow = EnvPath::into_os_cow(path);
    ///
    /// let path_cow = Cow::from(path.as_os_str());
    /// assert_eq!(os_cow, Some(path_cow));
    ///
    /// let pathbuf = PathBuf::from("/usr/bin");
    /// let cow_os_string = Cow::from(pathbuf.into_os_string());
    /// assert_eq!(os_cow, Some(cow_os_string));
    /// ```
    pub(crate) fn into_os_cow<'a, I: Into<OsString>>(s: I) -> OsCow<'a> {
        Some(Cow::from(s.into())) // Converts the input into an OsString and wraps it in a Cow object
    }
}

#[cfg(test)]
mod tests {
    // use ron::from_str;
    use super::*;

    #[test]
    fn into_os_cow_doc() {
        use std::{
            borrow::Cow,
            path::{Path, PathBuf},
        };

        let path = Path::new("/usr/bin");
        let os_cow = EnvPath::into_os_cow(path);

        let path_cow = Cow::from(path.as_os_str());
        assert_eq!(os_cow, Some(path_cow));

        let pathbuf = PathBuf::from("/usr/bin");
        let cow_os_string = Cow::from(pathbuf.into_os_string());
        assert_eq!(os_cow, Some(cow_os_string));
    }

    #[test]
    fn os_cow_doc() {
        use std::{borrow::Cow, ffi::OsStr};

        let str = "/usr/bin";
        let os_cow = EnvPath::os_cow(str);

        assert_eq!(os_cow, Some(Cow::from(OsStr::new(str))));
    }

    #[cfg(target_os = "android")]
    #[test]
    fn set_android_dir_doc() {
        use std::ffi::OsStr;

        let android_dir = EnvPath::set_android_dir("Android/obb/com.x.y/");

        assert_eq!(
            android_dir,
            Some(std::borrow::Cow::from(OsStr::new(
                "/storage/self/primary/Android/obb/com.x.y/"
            )))
        );
    }

    #[test]
    fn clear_raw_doc() {
        let mut path =
            EnvPath::new(vec!["$env: xdg_data_home", "$const: pkg", "files"]);

        path.clear_raw();

        assert!(!path.exists());

        assert!(path.get_raw().is_empty());
    }

    #[test]
    fn set_raw_doc() {
        // use envpath::EnvPath;

        let mut path = EnvPath::new(vec!["$dir: cfg", "config.ron"]);
        assert_eq!(path.get_raw(), &["$dir: cfg", "config.ron"]);

        path.set_raw(vec!["$project( com. x. y )", "cfg.toml"]);
        assert_eq!(path.get_raw(), &["$project( com. x. y )", "cfg.toml"]);
    }

    #[test]
    fn from_vec_or_slice() {
        let _v1 = EnvPath::from(vec!["$env:home"]);
        let _v2 = EnvPath::from(&["$dir: home"]);

        let v3 = vec!["$env:home"];
        let path = EnvPath::from(&v3);
        assert_eq!(path.get_raw(), &["$env:home"]);
    }
}
