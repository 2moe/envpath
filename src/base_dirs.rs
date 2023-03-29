use crate::{envpath_core::EnvPath, OsCow};
use std::{borrow::Cow, env, ops::ControlFlow, path::PathBuf};
impl EnvPath {
    /// Returns the path to the `Microsoft` directory in the local data folder on Windows, if available.
    ///
    /// | Platform | Example                                       |
    /// | -------- | --------------------------------------------- |
    /// | windows  | `C:\Users\[username]\AppData\Local\Microsoft` |
    ///
    /// An `Option<PathBuf>` object containing the path to the `Microsoft` directory, or `None` if it is unavailable.
    #[cfg(windows)] // This function is only available on Windows
    pub(crate) fn get_microsoft_windows_data_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|x| x.join("Microsoft")) // Gets the path to the local data directory and appends "Microsoft" to it
    }

    /// Returns the path to the executable directory or the `WindowsApps` directory on Windows.
    /// On Unix-like systems, returns the path to the `.local/bin` directory in the user's home directory.
    ///
    /// | Platform            | Example                                                   |
    /// | ------------------- | --------------------------------------------------------- |
    /// | windows             | `C:\Users\[username]\AppData\Local\Microsoft\WindowsApps` |
    /// | unix (if available) | `$env: xdg_bin_home`                                      |
    /// | unix                | `/home/[username]/.local/bin`                             |
    ///
    pub(crate) fn set_bin_dir<'a>() -> OsCow<'a> {
        let bin_dir =
            || dirs::data_local_dir().and_then(|p| Self::into_os_cow(p.join("bin"))); // Gets the path to the local data directory and appends "bin" to it, wrapped in an OsCow object

        match dirs::executable_dir() {
            // Checks if there is an executable directory
            Some(s) => Self::into_os_cow(s), // If there is, return it wrapped in an OsCow object
            #[cfg(windows)]
            _ => match Self::get_microsoft_windows_data_dir() {
                // If on Windows, check if the Microsoft directory is Some(x).
                Some(x) => Self::into_os_cow(x.join("WindowsApps")), // If it is, return the path to the WindowsApps directory wrapped in an OsCow object
                _ => bin_dir(), // Otherwise, return the bin directory wrapped in an OsCow object
            },
            #[cfg(unix)]
            _ => match dirs::home_dir() {
                // If on Unix, get the path to the home directory
                Some(x) => Self::into_os_cow(x.join(".local/bin")), // Append ".local/bin" to it and return it wrapped in an OsCow object
                _ => bin_dir(), // If the home directory is unavailable, return the bin directory wrapped in an OsCow object
            },
            #[cfg(not(any(unix, windows)))]
            _ => bin_dir(), // If not on Unix or Windows, return the bin directory wrapped in an OsCow object
        }
    }

    /// Returns the path to the system fonts directory on Windows, or the `fonts` directory in the system data directory on Unix-like systems.
    pub(crate) fn set_font_dir<'a>() -> OsCow<'a> {
        match dirs::font_dir() {
            // Checks if there is a font directory
            Some(s) => Self::into_os_cow(s), // If there is, return it wrapped in an OsCow object
            #[cfg(windows)]
            _ => match Self::get_microsoft_windows_data_dir() {
                // If on Windows, check if the Microsoft directory is available
                Some(x) => Self::into_os_cow(x.join(r#"Windows\Fonts"#)), // If it is, return the path to the Windows fonts directory wrapped in an OsCow object
                _ => Self::os_cow(r#"C:\Windows\Fonts"#), // Otherwise, return the path to the Windows fonts directory wrapped in an OsCow object
            },
            #[cfg(unix)]
            _ => dirs::data_dir().and_then(|p| Self::into_os_cow(p.join("fonts"))), // If on Unix, get the path to the system data directory and append "fonts" to it, then return it wrapped in an OsCow object
            #[cfg(not(any(unix, windows)))]
            _ => None, // If not on Unix or Windows, return None
        }
    }

    /// Returns either the first or last path in the `PATH` environment variable.
    pub(crate) fn set_double_ended_path(s: &str) -> OsCow {
        let Some(path) = env::var_os("PATH") else { // Gets the value of the PATH environment variable, or returns None if it is unavailable
            return None // If PATH is unavailable, return None
        };
        let path_iter = || env::split_paths(&path); // Splits the PATH variable into multiple paths
        let into_os_cow = |x: PathBuf| Cow::from(x.into_os_string()); // Wraps a PathBuf object in a Cow object

        match s {
            "first" => path_iter()
                .next()
                .map(into_os_cow), // If "first" is provided, return the first path in the PATH variable wrapped in an OsCow object
            "last" => path_iter()
                .last()
                .map(into_os_cow), // If "last" is provided, return the last path in the PATH variable wrapped in an OsCow object
            _ => None, // Otherwise, return None
        }
    }

    /// Returns the path to the temporary directory, either specified by the `TMPDIR` environment variable or the system temporary directory.
    pub(crate) fn set_tmp_dir<'a>() -> OsCow<'a> {
        match env::var_os("TMPDIR") {
            // Checks if the TMPDIR environment variable is set
            Some(s) => Self::into_os_cow(s), // If it is, return its value wrapped in an OsCow object
            None => match env::temp_dir() {
                // If it is not set, get the system temporary directory
                p if p
                    .metadata()
                    .map_or(true, |x| x.permissions().readonly()) =>
                // If the system temporary directory is read-only
                {
                    dirs::cache_dir().and_then(|x| Self::into_os_cow(x.join("tmp")))
                    // Get the path to the cache directory and append "tmp" to it, then return it wrapped in an OsCow object
                }
                p => Self::into_os_cow(p), // Otherwise, return the system temporary directory wrapped in an OsCow object
            },
        }
    }

    /// Returns the path to the directory specified by the given function, or the Android-specific directory if running on Android.
    ///
    /// # Parameters
    ///
    /// - `p` - A function that returns an `Option<PathBuf>` object.
    /// - `_android_dir` - A string representing the Android-specific directory to use.
    ///     For non-android platforms, to avoid the "unused variable" warning, I added the `_` prefix to the variable name
    ///
    pub(crate) fn set_dir<F>(p: F, _android_dir: &str) -> OsCow
    where
        F: FnOnce() -> Option<PathBuf>,
    {
        match () {
            #[cfg(target_os = "android")]
            () => Self::set_android_dir(_android_dir), // If running on Android, return the Android-specific directory wrapped in an OsCow object
            #[allow(unreachable_patterns)]
            () => p().and_then(Self::into_os_cow), // Otherwise, call the provided function and return its result wrapped in an OsCow object
        }
    }

    pub(crate) fn handle_dirs(ident: &str) -> OsCow {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(ident) {
            sep if sep == ' ' => Self::match_base_dirs(ident),
            sep => match Self::parse_dir_rules(ident, Self::match_base_dirs, sep) {
                Break(x) | Continue(x) => x,
            },
        }
    }

    /// Use `match` to match **ident** in `$dir: ident` and get different Paths depending on the platform.
    /// This is the core function of this module.
    pub(crate) fn match_base_dirs(ident: &str) -> OsCow {
        use dirs::*;
        let into_cow = |p: Option<PathBuf>| p.and_then(Self::into_os_cow);

        match ident {
            "music" | "audio" => Self::set_dir(audio_dir, "Music"),
            "cache" => into_cow(cache_dir()),
            "cfg" | "config" => into_cow(config_dir()),
            "data" => into_cow(data_dir()),
            "local_data" | "local-data" => {
                Self::set_dir(data_local_dir, "Android/data")
            }
            "local-cfg" | "local_cfg" | "local_config" => {
                into_cow(config_local_dir())
            }
            "desktop" => into_cow(desktop_dir()),
            "doc" | "document" | "documentation" => {
                Self::set_dir(data_local_dir, "Documents")
            }
            "dl" | "download" => Self::set_dir(data_local_dir, "Download"),
            "bin" | "exe" | "executable" => Self::set_bin_dir(),
            "path" | "first-path" | "first_path" => {
                Self::set_double_ended_path("first")
            }
            "last_path" | "last-path" => Self::set_double_ended_path("last"),
            "font" | "typeface" => Self::set_font_dir(),
            "home" => into_cow(home_dir()),
            "pic" | "picture" => Self::set_dir(audio_dir, "Pictures"),
            "pref" | "preference" => into_cow(preference_dir()),
            "pub" | "public" => into_cow(public_dir()),
            "runtime" => into_cow(runtime_dir()),
            "state" => into_cow(state_dir()),
            "template" => into_cow(template_dir()),
            "video" | "movie" => Self::set_dir(video_dir, "Movies"),
            "tmp" => Self::set_tmp_dir(),
            "temp" | "temporary" => Self::into_os_cow(env::temp_dir()),
            #[cfg(target_os = "android")]
            "sd" => Self::os_cow(Self::AND_SD),
            #[cfg(windows)]
            "local-low" | "local_low" => into_cow(data_local_dir().and_then(|p| {
                p.parent()
                    .map(|x| x.join("LocalLow"))
            })),
            "cli-data" | "cli_data" => into_cow(data_local_dir()),
            "cli-cfg" | "cli_cfg" | "cli_config" => into_cow(config_local_dir()),
            "cli-cache" | "cli_cache" => into_cow(cache_dir()),
            #[cfg(windows)]
            "progam-files" | "program_files" => Self::into_os_env("ProgramFiles")
                .or_else(|| Self::os_cow(r#"C:\Program Files"#)),
            #[cfg(windows)]
            "program-files-x86" | "program_files_x86" => {
                Self::into_os_env("ProgramFiles(x86)")
                    .or_else(|| Self::os_cow(r#"=C:\Program Files (x86)"#))
            }
            #[cfg(windows)]
            "common-program-files" | "common_program_files" => {
                Self::into_os_env("CommonProgramFiles")
                    .or_else(|| Self::os_cow(r#"C:\Program Files\Common Files"#))
            }
            #[cfg(windows)]
            "common-program-files-x86" | "common_program_files_x86" => {
                Self::into_os_env("CommonProgramFiles(x86)").or_else(|| {
                    Self::os_cow(r#"C:\Program Files (x86)\Common Files"#)
                })
            }
            #[cfg(windows)]
            "program-data" | "program_data" => Self::into_os_env("ProgramData")
                .or_else(|| Self::os_cow(r#"C:\ProgramData"#)),
            #[cfg(windows)]
            "microsoft" => into_cow(data_dir().map(|x| x.join("Microsoft"))),
            "empty" => Self::os_cow(""),
            x if Self::starts_with_star_arr(x) => Self::find_map_single_star(x),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn strange_dir() {
        let mut path =
            EnvPath::from(["$dir: states ?? template ?? video", " $const: pkg  "])
                .de();
        dbg!(path.display());

        path.set_raw(vec![" $dir:  bin ?? first_path  "]);
        dbg!(path.de().display());
    }

    #[test]
    fn remix_dir() {
        let p = EnvPath::new(["$env: user ?? dir * cfg ? empty"]);
        dbg!(p);

        let p2 = EnvPath::new(["$dir: runtimes ?? test ? env * HOME"]);
        dbg!(p2);
    }
}
