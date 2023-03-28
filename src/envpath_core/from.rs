use crate::envpath_core::{EnvPath, EnvPathRaw};

impl<S: Into<String>> FromIterator<S> for EnvPath {
    /// This is similar to `new()`.
    /// But the difference is that `new()` automatically converts the raw to path, whereas `from_iter()` needs to be done manually.
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Self {
            raw: Self::new_raw(iter),
            path: None,
        }
    }
}

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
        let raw = Self::new_raw(
            raw.iter()
                .map(|s| s.as_ref().to_string()),
        );

        Self { raw, path: None }
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
    pub fn new<S, V>(raw: V) -> Self
    where
        S: Into<String>,
        V: IntoIterator<Item = S>,
    {
        Self {
            raw: Self::new_raw(raw),
            path: None,
        }
        .de()
    }

    pub(crate) fn new_raw<S, V>(raw: V) -> EnvPathRaw
    where
        S: Into<String>,
        V: IntoIterator<Item = S>,
    {
        raw.into_iter()
            .map(|x| x.into())
            .collect()
    }

    /// Converts from `&[&str]` (`&[AsRef<str>]`) type to raw, then converts raw to path, and then returns a new instance of EnvPath.
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
    pub fn create_from_str_slice<S: AsRef<str>>(raw: &[S]) -> Self {
        Self::from_str_slice(raw).de()
    }

    /// Converts from `&[&str]` (`&[AsRef<str>]`) type to raw, then returns a new instance of EnvPath.
    ///
    /// Since `EnvPath` implements `From Trait`, you can use `EnvPath::from()` instead of `EnvPath::from_str_slice()`
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let home = ["$env:home"];
    /// let v1 = EnvPath::from(&home);
    /// assert_eq!(v1.get_raw(), &home);
    ///
    /// let v2 = EnvPath::from_str_slice(&home);
    ///
    /// assert_eq!(v2.get_raw(), &home);
    /// ```
    pub fn from_str_slice<S: AsRef<str>>(raw: &[S]) -> Self {
        Self {
            raw: Self::new_raw(
                raw.iter()
                    .map(|x| x.as_ref().to_string()),
            ),
            path: None,
        }
    }
}
