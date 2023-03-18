use crate::OsCow;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub(crate) type EnvPathRaw = Vec<String>;

#[derive(Debug, Default)]
pub struct EnvPath {
    pub(crate) raw: EnvPathRaw,
    pub(crate) path: Option<PathBuf>,
}

mod from {
    use crate::envpath_core::EnvPath;

    impl<'a, const N: usize> From<&'a [&'a str; N]> for EnvPath {
        /// # Examples
        ///
        /// ```
        /// use envpath::EnvPath;
        ///
        /// const ENV_PATH_RAW: [&str; 2] = ["$env: home", ".local"];
        /// let v = EnvPath::from(&ENV_PATH_RAW).de();
        /// assert_eq!(v.get_raw(), &["$env: home", ".local"]);
        /// dbg!(v.display());
        /// ```
        fn from(raw: &'a [&'a str; N]) -> Self {
            Self::from_str_slice(raw)
        }
    }

    impl<'a, const N: usize> From<[&'a str; N]> for EnvPath {
        /// # Examples
        ///
        /// ```
        /// use envpath::EnvPath;
        ///
        /// let arr = ["$env:home", "dev"];
        /// let path: EnvPath = arr.into();
        ///dbg!(path.de().display());
        /// ```
        fn from(raw: [&'a str; N]) -> Self {
            Self::from_str_slice(&raw)
        }
    }

    impl<S: Into<String>> From<Vec<S>> for EnvPath {
        /// This is similar to `new()` when you use `Vec<S>` (S: `Into<String>`) as an argument to `from()`.
        /// But the difference is that `new()` automatically converts the raw to path, whereas `from()` or `into()` needs to be done manually.
        ///
        /// # Examples
        ///
        /// ```
        /// use envpath::EnvPath;
        /// let v: EnvPath = vec!["$env:home"].into();
        /// assert_eq!(v.get_raw(), &["$env:home"]);
        /// dbg!(v.de().display());
        /// ```
        fn from(raw: Vec<S>) -> Self {
            Self {
                raw: Self::new_raw(raw),
                path: None,
            }
        }
    }

    impl<'a, T: AsRef<str>> From<&'a [T]> for EnvPath {
        /// The elements of an array slice can be of a type that implements `AsRef<str>`.
        fn from(raw: &'a [T]) -> Self {
            let raw_str = raw
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>();

            Self::from_str_slice(&raw_str)
        }
    }

    impl<'a> From<&'a Vec<&'a str>> for EnvPath {
        /// # Examples
        ///
        /// ```
        /// use envpath::EnvPath;
        ///
        /// let path = EnvPath::from(&vec!["$env:home"]);
        /// assert_eq!(path.get_raw(), &["$env:home"]);
        /// ```
        fn from(raw: &'a Vec<&'a str>) -> Self {
            Self::from_str_slice(raw)
        }
    }
}

impl EnvPath {
    /// Create a new instance of `EnvPath` from `Vec<S>`.
    ///
    /// Note: This function automatically converts the raw, and modifies the converted value to the path within the structure.
    /// If you just want to serialize it to the configuration, not deserialize cfg to it, please use `from()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::new(vec![
    ///     "$env: home",
    ///     ".local",
    ///     "share",
    ///     "$const: pkg",
    ///     "$const: ver"
    /// ]);
    ///
    /// dbg!(path.display(), path.exists());
    /// ```
    pub fn new<S: Into<String>>(raw: Vec<S>) -> Self {
        Self {
            raw: Self::new_raw(raw),
            path: None,
        }
        .de()
    }

    pub(crate) fn new_raw<S: Into<String>>(raw: Vec<S>) -> EnvPathRaw {
        raw.into_iter()
            .map(|x| x.into())
            .collect()
    }

    /// Converts from `&[&str]` type to raw, then converts raw to path, and then returns a new instance of EnvPath.
    ///
    /// | Methods                 | Similarities          | Differences               |
    /// | ----------------------- | --------------------- | ------------------------- |
    /// | create_from_str_slice() |                       | Auto convert raw to path  |
    /// | from_str_slice()        | Create a New Instance | Manually                  |
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    /// let v = EnvPath::create_from_str_slice(&["$env:home"]);
    /// dbg!(v.display(), v.exists());
    /// ```
    pub fn create_from_str_slice(raw: &[&str]) -> Self {
        Self::from_str_slice(raw).de()
    }

    /// Converts from `&[&str]` type to raw, then returns a new instance of EnvPath.
    ///
    /// Since `EnvPath` implements `From Trait`, you can use `EnvPath::from()` instead of `EnvPath::from_str_slice()`
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    /// let v1 = EnvPath::from(&["$env:home"]);
    /// assert_eq!(v1.get_raw(), &["$env:home"]);
    ///
    /// let v2 = EnvPath::from_str_slice(&["$env:home"]);
    ///
    /// assert_eq!(v2.get_raw(), &["$env:home"]);
    /// ```
    pub fn from_str_slice(raw: &[&str]) -> Self {
        Self {
            raw: Self::new_raw(
                raw.iter()
                    .map(ToString::to_string)
                    .collect(),
            ),
            path: None,
        }
    }

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
