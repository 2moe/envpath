use crate::envpath_core::EnvPath;
use std::path::PathBuf;

/// fullwidth colon
pub const FULL_COLON: char = '\u{FF1A}';
/// halfwidth colon
pub const HALF_COLON: char = '\u{3A}';

impl EnvPath {
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
        let raw = self.get_raw();

        if raw.is_empty() {
            // The function returns early to avoid additional overhead.
            return EnvPath {
                raw: self.raw,
                path: None,
            };
        }

        // Create a new string to store the casing for later use
        let mut casing = String::with_capacity(30);

        const CHUNK_NUM: usize = 2;

        // Fold over the EnvPathRaw sequence, accumulating the PathBuf
        let path = self
            .raw
            .iter()
            .map(|x| x.trim()) // Trim the string
            .fold(Some(PathBuf::with_capacity(16)), |acc, s| {
                acc.and_then(|acc_p| {
                    let split_s = |c| {
                        s.splitn(CHUNK_NUM, c)
                            .map(|x| x.trim())
                            .collect()
                    };

                    // Split the string into chunks on colons.
                    // Half and full colons are matched here.
                    // If someone forgets to switch the Chinese input method to English, it is easy to type ':' as '：', the two characters are particularly similar. To solve the confusion problem, it supports both.
                    let chunks = {
                        let hc = HALF_COLON;
                        let fc = FULL_COLON;
                        match (s.find(hc), s.find(fc)) {
                            (Some(h), Some(f)) if h < f => split_s(hc),
                            (Some(h), Some(f)) if f < h => split_s(fc),
                            (Some(_), None) => split_s(hc),
                            (None, Some(_)) => split_s(fc),
                            _ => Vec::new(),
                        }
                    };

                    // Get the number of chunks
                    let len = if chunks.is_empty() { 0 } else { CHUNK_NUM };

                    // Define closures to convert the second chunk to upper or lower case
                    let ucase = || chunks[1].to_ascii_uppercase();
                    let lcase = || chunks[1].to_ascii_lowercase();

                    // Define a closure to handle matching values
                    // Note: Self::os_cow(s) is the default value. `s` is an entire (unsplit), but trimmed str.
                    let match_val = |t| match t {
                        None => Self::os_cow(s),
                        v => v,
                    };

                    // Match on the number of chunks
                    match len {
                        // If the length is 0 or 1, return the default value.
                        0 | 1 => match_val(None),
                        // If the first element is $env, get the value of the environment variable with the second element as the key
                        _ => match chunks[0] {
                            "$env" => {
                                casing = ucase();

                                // Warning: The unsafe function is used here!
                                if casing.contains('-') {
                                    for i in unsafe { casing.as_bytes_mut() } {
                                        // Replace all '-' with '_'
                                        if *i == b'-' {
                                            *i = b'_';
                                        }
                                    }
                                }

                                // handle_env: Parsing environment variables (e.g.: `$env: home` or `$env: userprofile ?? home`)
                                match_val(Self::handle_envs(&casing))
                            }
                            // If the first element is $const and the const-dirs feature is enabled, get the value of the directory with the second element as the key
                            #[cfg(feature = "const-dirs")]
                            "$const" => {
                                casing = lcase();
                                match_val(Self::handle_const_dirs(&casing))
                            }
                            // If the first element is $dir and the base-dirs feature is enabled, get the value of the base directory with the second element as the key
                            #[cfg(feature = "base-dirs")]
                            "$dir" => {
                                casing = lcase();
                                match_val(Self::handle_dirs(&casing))
                            }
                            // If the first element starts with $project and the project-dirs feature is enabled, get the value of the project directory with the second element as the key
                            #[cfg(feature = "project-dirs")]
                            x if x.starts_with("$proj") => {
                                casing = lcase();
                                match_val(Self::handle_project_dirs(x, &casing))
                            }
                            // If none of the above conditions are met, return the default value.
                            _ => match_val(None),
                        },
                    }
                    // Join the path of the accumulator with the parsed path.
                    .map(|p| acc_p.join(p))
                })
            });

        Self {
            raw: self.raw,
            path,
        }
    }
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
