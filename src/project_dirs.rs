use crate::{
    envpath_core::EnvPath,
    parser::{FULL_COLON, HALF_COLON},
    OsCow,
};
use directories::ProjectDirs;
use std::{borrow::Cow, ops::ControlFlow, path::Path};

/// Implement additional methods for EnvPath when the "project-dirs" feature is enabled
///
/// If you see a method(function) with a parameter name containing **_** prefix (e.g. **_name**) in some methods, do not delete it.
/// This may be a platform-specific parameter, so to avoid the "unused variable" warning, I've added the "_" prefix.
#[cfg(feature = "project-dirs")]
impl EnvPath {
    // Method to extract project name information from a string
    pub(crate) fn get_project_name(c0: &str) -> Option<(&str, &str, Cow<str>)> {
        // Find the first and last occurrence of parentheses in the string
        let (Some(start), Some(end)) = (c0.find('('), c0.rfind(')')) else {
            return None
        };

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

    // Method to set the project cache directory
    pub(crate) fn set_proj_cache<'a>(
        _name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        match () {
            #[cfg(target_os = "android")]
            () => Self::into_os_cow(PathBuf::from_iter([
                "/", "data", "data", _name, "cache",
            ])), // If the target OS is Android, use a specific path for the cache directory
            #[allow(unreachable_patterns)]
            () => proj.and_then(|s| Self::into_os_cow(s.cache_dir())), // Otherwise, use the cache directory provided by the ProjectDirs object
        }
    }

    // Method to set the project configuration directory
    pub(crate) fn set_proj_cfg<'a>(
        _name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        match () {
            #[cfg(target_os = "android")]
            () => Self::into_os_cow(PathBuf::from_iter([
                "/", "data", "data", _name, "files",
            ])), // If the target OS is Android, use a specific path for the configuration directory
            #[allow(unreachable_patterns)]
            () => proj.and_then(|s| Self::into_os_cow(s.config_dir())), // Otherwise, use the configuration directory provided by the ProjectDirs object
        }
    }

    // Method to set the project data directory
    pub(crate) fn set_proj_data<'a>(
        _name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        match () {
            #[cfg(target_os = "android")]
            () => {
                Self::into_os_cow(PathBuf::from_iter(["/", "data", "data", _name]))
            } // If the target OS is Android, use a specific path for the data directory
            #[allow(unreachable_patterns)]
            () => proj.and_then(|s| Self::into_os_cow(s.data_dir())), // Otherwise, use the data directory provided by the ProjectDirs object
        }
    }

    // Method to set the project local data directory
    pub(crate) fn set_proj_local_data<'a>(
        _name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        match () {
            #[cfg(target_os = "android")]
            () => Self::set_android_dir(format!("Android/data/{_name}")), // If the target OS is Android, use a specific path for the local data directory
            #[allow(unreachable_patterns)]
            () => proj.and_then(|s| Self::into_os_cow(s.data_local_dir())), // Otherwise, use the local data directory provided by the ProjectDirs object
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
        remain: &str,
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
                let Some((name, proj)) = Self::set_proj_name_opt_tuple(first_chunk) else {
                    return None
                };
                Self::match_proj_dirs(remain, &name, proj.as_ref())
            }
            sep => match Self::parse_proj_dir_rules(first_chunk, remain, sep) {
                Break(x) | Continue(x) => x,
            },
        }
    }

    fn set_proj_name_opt_tuple(
        chunk: &str,
    ) -> Option<(String, Option<ProjectDirs>)> {
        // Extract the project name information from the first chunk
        let Some((qual, org, app)) = Self::get_project_name(chunk) else {
            return None // If the project name information cannot be extracted, return None
        };

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
        ident: &str,
        name: &str,
        proj: Option<&ProjectDirs>,
    ) -> OsCow<'a> {
        // Define a closure to convert an Option<Path> to an OsCow
        let and_then_cow = |s: Option<&Path>| s.and_then(Self::into_os_cow);

        // Determine which project directory is being requested and set the corresponding path
        match ident {
            "path" => Self::set_proj_path(name, proj), // Set the project path
            "cache" => Self::set_proj_cache(name, proj), // Set the cache directory
            "cfg" | "config" => Self::set_proj_cfg(name, proj), // Set the configuration directory
            "data" => Self::set_proj_data(name, proj), // Set the data directory
            "local-data" | "local_data" => Self::set_proj_local_data(name, proj), // Set the local data directory
            "local-cfg" | "local_cfg" | "local_config" => {
                proj.and_then(|x| Self::into_os_cow(x.config_local_dir()))
            }
            "pref" | "preference" => {
                proj.and_then(|x| Self::into_os_cow(x.preference_dir())) // Set the preference directory
            }
            "runtime" => proj.and_then(|x| and_then_cow(x.runtime_dir())), // Set the runtime directory
            "state" => proj.and_then(|x| and_then_cow(x.state_dir())), // Set the state directory
            _ => None, // If an unknown directory is requested, return None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn test_proj_dir() {
        let path = EnvPath::from([
            "$project(me. tmm. store-demo): cfg  runtime ??  (    me. tmm. wasm-module  )： data ?? state ? cfg ?",
        ])
        .de();
        dbg!(path.display());
    }

    #[test]
    fn test_proj_dir_question_mark() {
        let path = EnvPath::from([
            "$project(me. tmm. store-demo): cfg  runtime ？？  (    me. tmm. wasm-module  )： data ？？ state ？？ cfg",
        ])
        .de();
        dbg!(path.display());

        let path2 = EnvPath::from(["$proj (com . moz . ff )：runtimes ？ data ？？ state ？？ (com . gg . cr)： cfg ？？ cache ？ (com . ms . eg)： local-data ？ data"]).de();
        dbg!(path2.display());
    }
}
