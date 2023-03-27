use crate::{EnvPath, OsCow};
use std::ops::ControlFlow;

impl EnvPath {
    /// This function is used to resolve ident in `$const: ident`.
    /// Although the relevant content is obtained at compile time, but wrapping it in `OsCow` is not.
    fn match_const_dirs(ident: &str) -> OsCow {
        // Imports for using all env::consts
        use std::env::consts::*;
        // Create a cow wrapper for the OS Str.
        // In fact, this is only the alias equivalent of the `os_cow()` function.
        let as_cow = Self::os_cow;

        // #[cfg(debug_assertions)]
        // dbg!(&ident);

        match ident {
            "pkg" | "pkg_name" | "pkg-name" => as_cow(env!("CARGO_PKG_NAME")),
            // "bin" | "bin_name" => as_cow(env!("CARGO_BIN_NAME")),
            "pkg_version" | "pkg-version" | "ver" => {
                as_cow(env!("CARGO_PKG_VERSION"))
            }
            "arch" | "architecture" => as_cow(ARCH),
            "deb_arch" | "deb-arch" => as_cow(crate::arch::get_deb_arch()),
            "os" => as_cow(OS),
            "family" => as_cow(FAMILY),
            "exe_suffix" => as_cow(EXE_SUFFIX),
            "exe_extension" => as_cow(EXE_EXTENSION),
            _ => None,
        }
    }

    pub(crate) fn handle_const_dirs(ident: &str) -> OsCow {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(ident) {
            sep if sep == ' ' => Self::match_const_dirs(ident),
            sep => match Self::parse_dir_rules(ident, Self::match_const_dirs, sep) {
                Break(x) | Continue(x) => x,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::EnvPath;

    #[test]
    fn test_const_dir() {
        let v = EnvPath::from(["$const: family ?? os"]);
        dbg!(v.de().display());
    }
}
