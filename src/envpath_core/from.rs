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
