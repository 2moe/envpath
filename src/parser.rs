use crate::{os_cow, EnvPath, OsCow};
use std::path::PathBuf;

/// fullwidth colon
pub(crate) const FULL_COLON: char = '\u{FF1A}';
/// halfwidth colon
pub(crate) const HALF_COLON: char = '\u{3A}';

const CHUNK_NUM: usize = 2;

pub(crate) fn parse<S: AsRef<str>, I: IntoIterator<Item = S>>(
    iter: I,
) -> Option<PathBuf> {
    // Create a new string to store the casing for later use
    let mut casing = String::with_capacity(30);

    iter.into_iter()
        // Fold over the EnvPathRaw sequence, accumulating the PathBuf
        .fold(Some(PathBuf::with_capacity(16)), |acc, s| {
            acc.and_then(|acc_p| {
                let s = s.as_ref();
                // Split the string into chunks on colons.
                let chunks = get_chunks(s.trim());

                // Get the number of chunks
                let len = if chunks.is_empty() { 0 } else { CHUNK_NUM };

                // Define a fn to handle values. If val is None, then the default value is returned.
                // Note: crate::os_cow::from_str(s) is the default value. `s` is the raw str.
                fn or_default<'a>(val: OsCow<'a>, s: &'a str) -> OsCow<'a> {
                    val.or_else(|| os_cow::from_str(s))
                }

                // When calling this closure, make sure len >= 2
                let get_2nd_chunk = || unsafe { chunks.get_unchecked(1) };

                // Match on the number of chunks
                match len {
                    // If the length is 0 or 1, return the default value.
                    0 | 1 => or_default(None, s),
                    // If the first element is $env, get the value of the environment variable with the second element as the key
                    _ => match chunks[0] {
                        "$env" => {
                            match get_2nd_chunk() {
                                x if x.contains('*') => {
                                    casing = x.to_string();
                                }
                                x => {
                                    casing = x.to_ascii_uppercase();
                                    // Warning: The unsafe function is used here!
                                    if casing.contains('-') {
                                        for i in unsafe { casing.as_bytes_mut() } {
                                            // Replace all '-' with '_'
                                            if *i == b'-' {
                                                *i = b'_';
                                            }
                                        }
                                    }
                                }
                            }

                            // handle_env: Parsing environment variables (e.g.: `$env: home` or `$env: userprofile ?? home`)
                            or_default(EnvPath::handle_envs(&casing), s)
                        }
                        // If the first element is $const and the consts feature is enabled, get the value of the directory with the second element as the key
                        #[cfg(feature = "consts")]
                        "$const" => {
                            or_default(EnvPath::handle_consts(get_2nd_chunk()), s)
                        }
                        #[cfg(feature = "value")]
                        "$val" => {
                            or_default(EnvPath::handle_values(get_2nd_chunk()), s)
                        }
                        // If the first element is $dir and the base-dirs feature is enabled, get the value of the base directory with the second element as the key
                        #[cfg(feature = "dirs")]
                        "$dir" => {
                            or_default(EnvPath::handle_dirs(get_2nd_chunk()), s)
                        }
                        // If the first element starts with `$proj` and the `project` feature is enabled, get the value of the project directory with the second element as the key
                        #[cfg(feature = "project")]
                        x if x.starts_with("$proj") => or_default(
                            EnvPath::handle_project_dirs(x, get_2nd_chunk()),
                            s,
                        ),
                        // If none of the above conditions are met, return the default value.
                        _ => or_default(None, s),
                    },
                }
                // Join the path of the accumulator with the parsed path.
                .map(|p| acc_p.join(p))
            })
        })
}

impl EnvPath<'_> {
    /// This function is used for deserialization.
    /// Although EnvPath implements Deserialize Trait with `deserialize()`, it essentially calls this `de()` function.
    ///
    /// In addition to deserializing the configuration file, you can parse the vector or array.
    ///
    /// # Examples
    ///
    /// ```
    /// use envpath::EnvPath;
    ///
    /// let path = EnvPath::from(["$dir: data ?? cfg", "$const: deb-arch"]).de();
    /// dbg!(path.display(), path.exists());
    /// ```
    pub fn de(self) -> Self {
        let ref_raw = self.get_raw();

        if ref_raw.is_empty() {
            // The function returns early to avoid additional overhead.
            return EnvPath {
                raw: self.raw,
                path: None,
            };
        }

        let path = ref_raw.parse();

        Self {
            raw: self.raw,
            path,
        }
    }
}

/// Split the string into chunks on colons.
/// Half and full colons are matched here.
/// If someone forgets to switch the Chinese input method to English, it is easy to type ':' as '：', the two characters are particularly similar. To solve the confusion problem, it supports both.
pub(crate) fn get_chunks(s: &str) -> Vec<&str> {
    let hc = HALF_COLON;
    let fc = FULL_COLON;
    match (s.find(hc), s.find(fc)) {
        (Some(h), Some(f)) if h < f => split_n(s, hc),
        (Some(h), Some(f)) if f < h => split_n(s, fc),
        (Some(_), None) => split_n(s, hc),
        (None, Some(_)) => split_n(s, fc),
        _ => Vec::new(),
    }
}

fn split_n(s: &str, c: char) -> Vec<&str> {
    s.splitn(CHUNK_NUM, c)
        .map(|x| x.trim())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn deser_vec() {
        let v = EnvPath::from(["  $env  :  home  "]).de();
        dbg!(v);
    }

    #[test]
    fn deser_doc() {
        // use envpath::EnvPath;
        let path = EnvPath::from(["$env: home"]).de();
        dbg!(path.display(), path.exists());
    }
}
