use std::borrow::Cow;

use crate::{EnvPath, Raw};

impl FromIterator<String> for EnvPath<'_> {
    /// This is similar to `new()`.
    /// But the difference is that `new()` automatically converts the raw to path, whereas `from_iter()` needs to be done manually.
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            raw: Raw::Owned(iter.into_iter().collect()),
            path: None,
        }
    }
}

impl<'r> FromIterator<Cow<'r, str>> for EnvPath<'r> {
    fn from_iter<I: IntoIterator<Item = Cow<'r, str>>>(iter: I) -> Self {
        Self {
            raw: Raw::Cow(iter.into_iter().collect()),
            path: None,
        }
    }
}

impl<'r> FromIterator<&'r str> for EnvPath<'r> {
    fn from_iter<I: IntoIterator<Item = &'r str>>(iter: I) -> Self {
        Self {
            raw: Self::create_ref_raw(iter),
            path: None,
        }
    }
}

impl<'r, const N: usize> From<&[&'r str; N]> for EnvPath<'r> {
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// const ENV_PATH_RAW: [&str; 2] = ["$env: home", ".local"];
    /// let v = EnvPath::from(&ENV_PATH_RAW).de();
    /// dbg!(v.get_raw());
    /// dbg!(v.display());
    /// ```
    fn from(raw: &[&'r str; N]) -> Self {
        Self {
            raw: Raw::Ref(raw.to_vec()),
            path: None,
        }
    }
}

impl<'r, const N: usize> From<[&'r str; N]> for EnvPath<'r> {
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let arr = ["$env:home", "dev"];
    /// let path: EnvPath = arr.into();
    ///dbg!(path.de().display());
    /// ```
    fn from(raw: [&'r str; N]) -> Self {
        Self {
            raw: Raw::Ref(raw.to_vec()),
            path: None,
        }
    }
}

impl<'r> From<Vec<&'r str>> for EnvPath<'r> {
    /// This is similar to `new()` when you use `Vec<S>` (S: `Into<String>`) as an argument to `from()`.
    /// But the difference is that `new()` automatically converts the raw to path, whereas `from()` or `into()` needs to be done manually.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    /// let v: EnvPath = vec!["$env:home"].into();
    /// dbg!(v.get_raw());
    /// dbg!(v.de().display());
    /// ```
    fn from(raw: Vec<&'r str>) -> Self {
        Self {
            raw: Raw::Ref(raw),
            path: None,
        }
    }
}

impl<'r, T: AsRef<str>> From<&[T]> for EnvPath<'r> {
    /// The elements of an array slice can be of a type that implements `AsRef<str>`.
    fn from(raw: &[T]) -> Self {
        Self {
            raw: Raw::Owned(
                raw.iter()
                    .map(|s| s.as_ref().to_owned())
                    .collect(),
            ),
            path: None,
        }
    }
}

impl<'r> From<&Vec<&'r str>> for EnvPath<'r> {
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::from(&vec!["$env:home"]);
    /// dbg!(path.get_raw());
    /// ```
    fn from(raw: &Vec<&'r str>) -> Self {
        Self {
            raw: Raw::Ref(raw.to_vec()),
            path: None,
        }
    }
}

impl<'r> EnvPath<'r> {
    /// Create a new instance of `EnvPath` from an iterator over borrowed strings.
    ///
    /// The function takes an iterator over borrowed strings and creates an instance of `EnvPath`.
    /// The raw value is automatically converted to represent the path structure.
    ///
    /// If you only need to serialize the path to a configuration and do not require deserialization support,
    /// consider using the `from()` method instead of this constructor.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::new([
    ///     "$env: home",
    ///     ".local",
    ///     "share",
    ///     "$const: pkg",
    ///     "$const: ver"
    /// ]);
    ///
    /// dbg!(path.display(), path.exists());
    /// ```
    pub fn new<V>(iter: V) -> Self
    where
        V: IntoIterator<Item = &'r str>,
    {
        Self {
            raw: Self::create_ref_raw(iter),
            path: None,
        }
        .de()
    }

    /// Create a new instance of `EnvPath` from an iterator over owned strings.
    ///
    /// Note: `new_owned()` will convert `&str` to `String`, which may result in additional heap memory allocation.
    ///
    /// To avoid additional heap memory allocation, you can use `new()` instead of `new_owned()`.
    ///
    /// The function takes an iterator over owned strings and creates an instance of `EnvPath`.
    /// The raw value is automatically converted to represent the path structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let arr = [
    ///     "$env: home",
    ///     ".local",
    ///     "share",
    ///     "$const: pkg",
    ///     "$const: ver",
    /// ];
    ///
    /// let path = EnvPath::new_owned(arr);
    ///
    /// dbg!(path.display(), path.exists());
    /// ```
    pub fn new_owned<V, S>(iter: V) -> Self
    where
        V: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            raw: Raw::Owned(
                iter.into_iter()
                    .map(|s| s.into())
                    .collect(),
            ),
            path: None,
        }
        .de()
    }

    /// Create a new instance of `EnvPath` from an iterator over borrowed Cow strings.
    ///
    /// The function takes an iterator over borrowed Cow strings and creates an instance of `EnvPath`.
    /// The raw value is automatically converted to represent the path structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use envpath::EnvPath;
    ///
    /// let arr = [
    ///     "$env: home",
    ///     ".local",
    ///     "share",
    ///     "$const: pkg",
    ///     "$const: ver",
    /// ];
    ///
    /// let path = EnvPath::new_cow(arr.map(Cow::Borrowed));
    /// dbg!(path.display(), path.exists());
    /// ```
    pub fn new_cow<V>(iter: V) -> Self
    where
        V: IntoIterator<Item = Cow<'r, str>>,
    {
        Self {
            raw: Raw::Cow(iter.into_iter().collect()),
            path: None,
        }
        .de()
    }

    /// Create a new instance of `Raw` from an iterator over borrowed strings.
    ///
    /// which is used internally by the other constructor methods to create `EnvPath` instances.
    pub(crate) fn create_ref_raw<V>(iter: V) -> Raw<'r>
    where
        V: IntoIterator<Item = &'r str>,
    {
        Raw::Ref(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn from_iter_ref() {
        let arr = vec!["$dir: data", "test"];
        let path = EnvPath::from(&arr);
        dbg!(arr);
        dbg!(path.de());
    }

    #[test]
    fn new_env_path() {
        let arr = ["$dir: cfg", "test2"].map(|x| x.to_string());
        let path = EnvPath::new_owned(&arr);
        dbg!(&path, &arr);

        let arr = [
            "$env: home",
            ".local",
            "share",
            "$const: pkg",
            "$const: ver",
        ];

        let path = EnvPath::new_cow(arr.map(std::borrow::Cow::Borrowed));
        dbg!(path.display(), path.exists());
    }
}
