use crate::{
    envpath_core::EnvPath,
    parser::{FULL_COLON, HALF_COLON},
    OsCow, ProjectDirs,
};

#[cfg(windows)]
use directories::BaseDirs;

#[cfg(target_os = "android")]
use std::path::PathBuf;

use std::{borrow::Cow, io, ops::ControlFlow, path::Path};

/// Implement additional methods for EnvPath when the "project-dirs" feature is enabled
///
/// If you see a method(function) with a parameter name containing **_** prefix (e.g. **_name**) in some methods, do not delete it.
/// This may be a platform-specific parameter, so to avoid the "unused variable" warning, I've added the "_" prefix.
impl EnvPath {
    // Method to extract project name information from a string
    pub(crate) fn get_project_name(c0: &str) -> Option<(&str, &str, Cow<str>)> {
        // Find the first and last occurrence of parentheses in the string
        let (start, end) = (c0.find('(')?, c0.rfind(')')?);

        // Extract the content within the parentheses
        let content = &c0[start + 1..end];

        // Split the content by periods and trim each part
        let parts = content
            .split('.')
            .map(|x| x.trim())
            .collect::<Vec<_>>();

        // Extract the qualifier, organization, and application from the parts
        let (qualifier, organization, application) = match parts.len() {
            0 => return None, // If there are no parts, return None
            1 => (parts[0], "", Cow::from(parts[0])), // If there is only one part, use it as the application name
            2 => (parts[0], "", Cow::from(parts[1])), // If there are two parts, use the first as the qualifier and the second as the application name
            3 => (parts[0], parts[1], Cow::from(parts[2])), // If there are three parts, use the first as the qualifier, the second as the organization, and the third as the application name
            _ => (parts[0], parts[1], Cow::from(parts[2..].concat())), // If there are more than three parts, use the first as the qualifier, the second as the organization, and the rest as the application name
        };

        Some((qualifier, organization, application))
    }

    // Method to set the project path
    pub(crate) fn set_proj_path<'a>(
        name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        match () {
            #[cfg(target_os = "android")]
            () => Self::into_os_cow(name), // If the target OS is Android, use the input name as the path
            #[allow(unreachable_patterns)]
            () => match proj {
                Some(s) => Self::into_os_cow(s.project_path()), // If a ProjectDirs object is provided, use its project path
                _ => Self::into_os_cow(name), // Otherwise, use the input name as the path
            },
        }
    }

    /// The function returns an `io::Result<ProjectDirs>`, which is created using the [ProjectDirs::from()](directories::ProjectDirs::from) method.
    ///
    /// Note: `directories::ProjectDirs` is different from `$proj` of EnvPath!
    ///
    /// | Parameter | Description                                                                           |
    /// | --------- | ------------------------------------------------------------------------------------- |
    /// | qual      | The qualifier of the project, which is a string reference.                            |
    /// | org       | The organization responsible for the project, also as a string reference.             |
    /// | app       | The name of the application associated with the project, again as a string reference. |
    ///
    /// Here's a table explaining the parts of the string "org.moz.ff" and what they represent:
    ///
    /// | Part | Abbreviation | Meaning     |
    /// |------|-------------|--------------|
    /// | org  | qual        | qualifier    |
    /// | moz  | org         | organization |
    /// | ff   | app         | application  |
    ///
    pub fn new_project<Q, O, A>(qual: Q, org: O, app: A) -> io::Result<ProjectDirs>
    where
        Q: AsRef<str>,
        O: AsRef<str>,
        A: AsRef<str>,
    {
        ProjectDirs::from(qual.as_ref(), org.as_ref(), app.as_ref()).ok_or_else(
            || {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Cannot generate ProjectDirs for your platform.",
                )
            },
        )
    }

    // Method to set the project directory
    pub(crate) fn set_proj_dir<'a, F>(
        proj: Option<&ProjectDirs>,
        f: F,
        _android_iter: &[&str],
    ) -> OsCow<'a>
    where
        F: Fn(&ProjectDirs) -> &Path,
    {
        match () {
            #[cfg(target_os = "android")]
            () => Self::into_os_cow(PathBuf::from_iter(_android_iter)),
            #[allow(unreachable_patterns)]
            () => proj.and_then(|s| Self::into_os_cow(f(s))), // Otherwise, use the configuration directory provided by the ProjectDirs object
        }
    }

    /// This contains more complex parsing rules than `parse_dir_rules()`.
    /// For example:
    /// `$proj(com.a.b): state   ? cfg ?? data ? local-data ? (com. x. y. z): data ?? local-data ? cfg`
    ///
    /// If the value of the state path for the project (com.a.b) exists, then break, otherwise continue parsing.
    /// Next is cfg, which, because it is followed by `?`, so not only must the value exist, but the path must exist as well.
    /// If one of the two does not exist, the parsing continues. If neither `data` nor `local-data` has a value, then we're at the (com.x.y.z) project and continue parsing.
    fn parse_proj_dir_rules<'a>(
        first: &str,
        remain: &'a str,
        separator: char,
    ) -> ControlFlow<OsCow<'a>, OsCow<'a>> {
        use ControlFlow::{Break, Continue};

        let mut last_chunk = String::with_capacity(20);

        remain
            .split_terminator(separator)
            .enumerate()
            .map(|(u, x)| (u, x.trim()))
            .try_fold(None, |acc: OsCow, (u, x)| match (acc, x.is_empty()) {
                (None, true) => Continue(None),
                (None, false) => {
                    let c0 = match (u, x.find('(')) {
                        (0, None) => {
                            // dbg!(&chunk, &x);
                            last_chunk = first.to_owned();
                            first
                        }
                        (_, Some(_)) => {
                            last_chunk = x.to_owned();
                            x
                        }
                        _ => &last_chunk,
                    };

                    // dbg!(&c0);
                    let Some((name, proj)) = Self::set_proj_name_opt_tuple(c0) else {
                        return Break(None)
                    };

                    let ident = match x
                        .rsplit([FULL_COLON, HALF_COLON])
                        .map(|x| x.trim())
                        .next()
                    {
                        Some(x) => x,
                        _ => return Break(None),
                    };
                    // dbg!(&name, &proj, &ident);
                    // dbg!(&ident);

                    Continue(Self::match_proj_dirs(ident, &name, proj.as_ref()))
                }
                (p, false) => Break(p),
                (Some(p), true) => match Path::new(&p) {
                    x if x.exists() => Break(Some(p)),
                    _ => Continue(None),
                },
            })
    }

    pub(crate) fn handle_project_dirs<'a>(
        first_chunk: &'a str,
        remain: &'a str,
    ) -> OsCow<'a> {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(remain) {
            sep if sep == ' ' => {
                let (name, proj) = Self::set_proj_name_opt_tuple(first_chunk)?;

                Self::match_proj_dirs(remain, &name, proj.as_ref())
            }
            sep => match Self::parse_proj_dir_rules(first_chunk, remain, sep) {
                Break(x) | Continue(x) => x,
            },
        }
    }

    pub(crate) fn set_proj_name_opt_tuple(
        chunk: &str,
    ) -> Option<(String, Option<ProjectDirs>)> {
        // Extract the project name information from the first chunk
        // If the project name information cannot be extracted, return None
        let (qual, org, app) = Self::get_project_name(chunk)?;

        // Create a ProjectDirs object using the project name information
        let proj = ProjectDirs::from(qual, org, &app);

        // Construct the project name by joining the qualifier, organization, and application
        Some((
            [qual, org, &app]
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
                .join("."),
            proj,
        ))
    }

    // Method to handle a project directory request
    pub(crate) fn match_proj_dirs<'a>(
        ident: &'a str,
        name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        // Define a closure to convert an Option<Path> to an OsCow
        let and_then_cow = |s: Option<&Path>| s.and_then(Self::into_os_cow);

        // Determine which project directory is being requested and set the corresponding path
        let proj_path = || Self::set_proj_path(name, proj);

        match ident {
            "path" => proj_path(), // Set the project path
            "cache" => Self::set_proj_dir(
                proj,
                ProjectDirs::cache_dir,
                &["/", "data", "data", name, "cache"],
            ),
            "cfg" | "config" => Self::set_proj_dir(
                proj,
                ProjectDirs::config_dir,
                &["/", "data", "data", name, "files"],
            ),
            "data" => Self::set_proj_dir(
                proj,
                ProjectDirs::data_dir,
                &["/", "data", "data", name],
            ),
            "local-data" | "local_data" => Self::set_proj_dir(
                proj,
                ProjectDirs::data_local_dir,
                &[Self::AND_SD, "Android", "data", name],
            ),
            "local-cfg" | "local_cfg" | "local_config" => Self::set_proj_dir(
                proj,
                ProjectDirs::config_local_dir,
                &[Self::AND_SD, "Android", "data", name, "files"],
            ),
            "pref" | "preference" => Self::set_proj_dir(
                proj,
                ProjectDirs::preference_dir,
                &["/", "data", "data", name, "files"],
            ),
            "runtime" => proj.and_then(|x| and_then_cow(x.runtime_dir())),
            "state" => proj.and_then(|x| and_then_cow(x.state_dir())),
            "cli-data" | "cli_data" => {
                proj.and_then(|p| Self::into_os_cow(p.data_local_dir()))
            }
            "cli-cfg" | "cli_cfg" | "cli_config" => {
                proj.and_then(|p| Self::into_os_cow(p.config_local_dir()))
            }
            "cli-cache" | "cli_cache" => {
                proj.and_then(|p| Self::into_os_cow(p.cache_dir()))
            }
            #[cfg(windows)]
            "local-low" | "local_low" => {
                let opt = BaseDirs::new().and_then(|p| {
                    proj_path().map(|x| {
                        p.data_local_dir()
                            .join("LocalLow")
                            .join(x)
                    })
                });
                opt.and_then(Self::into_os_cow)
            }
            "empty" => Self::os_cow(""),
            x if Self::starts_with_remix_expr(x) => Self::parse_remix_expr(x),
            _ => None,
            // If an unknown directory is requested, return None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn test_proj_dir() {
        let path = EnvPath::from([
            "$project(me. tmm. store-demo): cfg ? runtime ??  (    me. tmm. wasm-module  )： data ?? state ? cfg ?",
        ])
        .de();
        dbg!(path.display());
    }

    #[test]
    fn test_proj_dir_question_mark() {
        let path = EnvPath::from([
            "$project(me. tmm. store-demo): local-cfg ? runtime ？？  (    me. tmm. wasm-module  )： data ？？ state ？？ cfg",
        ])
        .de();
        dbg!(path.display());

        let path2 = EnvPath::from(["
            $proj (com . moz . ff )：runtimes ？ data ？？ state ？？ 
            (com . gg . cr)： cfg ？？ cache ？ 
            (com . ms . eg)： local-data ？ data
            "])
        .de();
        dbg!(path2.display());
    }

    #[test]
    fn proj_env() {
        let p = EnvPath::new(["$proj (org.x) : data ?? env * HOME"]);
        dbg!(p.display());
    }

    #[test]
    fn remix_proj_and_env() {
        let p = EnvPath::new(["$env: user ?? proj * (com.xy.z):cfg ? HOME"]);
        dbg!(p);

        let p2 = EnvPath::new(["$proj * (org. a . b ): runtimes ? env * HOME"]);
        dbg!(p2);
    }
}
