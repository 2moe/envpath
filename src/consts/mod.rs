use crate::{EnvPath, OsCow};
use std::{env::consts, ops::ControlFlow};

mod arch;
pub use arch::get_deb_arch;

pub const fn get_architecture() -> &'static str {
    consts::ARCH
}

#[macro_export]
macro_rules! get_pkg_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

#[macro_export]
macro_rules! get_pkg_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub const fn get_os_name() -> &'static str {
    consts::OS
}

pub const fn get_os_family() -> &'static str {
    consts::FAMILY
}

impl EnvPath<'_> {
    /// This function is used to resolve ident in `$const: ident`.
    /// Although the relevant content is obtained at compile time, but wrapping it in `OsCow` is not.
    pub(crate) fn match_consts(ident: &str) -> OsCow {
        // Create a cow wrapper for the OS Str.
        // In fact, this is only the alias equivalent of the `os_cow()` function.
        let as_cow = crate::os_cow::from_str;

        match ident {
            // "pkg" | "pkg_name" | "pkg-name" => as_cow(get_pkg_name!()),
            // "bin" | "bin_name" => as_cow(env!("CARGO_BIN_NAME")),
            // "pkg_version" | "pkg-version" | "ver" => as_cow(get_pkg_version!()),
            "arch" | "architecture" => as_cow(get_architecture()),
            "deb_arch" | "deb-arch" => as_cow(get_deb_arch()),
            "os" => as_cow(get_os_name()),
            "family" => as_cow(get_os_family()),
            "exe_suffix" => as_cow(consts::EXE_SUFFIX),
            "exe_extension" => as_cow(consts::EXE_EXTENSION),
            "empty" => as_cow(""),
            x if Self::starts_with_remix_expr(x) => Self::parse_remix_expr(x),
            _ => None,
        }
    }

    pub(crate) fn handle_consts(ident: &str) -> OsCow {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(ident) {
            sep if sep == ' ' => Self::match_consts(ident),
            sep => match Self::parse_dir_rules(ident, Self::match_consts, sep) {
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

        let p = EnvPath::new(["$const: empty ?? dir * config"]);
        dbg!(p.display());
    }
}
