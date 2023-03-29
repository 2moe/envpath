use crate::{envpath_core::EnvPath, OsCow};
use std::{borrow::Cow, env::var_os, ops::ControlFlow, path::Path};

/// fullwidth question mark
pub const FWQM: char = '\u{FF1F}';
/// halfwidth question mark
pub const HWQM: char = '\u{3F}';

impl EnvPath {
    pub(crate) const START_ARR: [&str; 5] = ["env", "dir", "const", "proj", "val"];
    /// It's a function for parsing rules(e.g. `$env: user ? userprofile ?? home`).
    /// The `s` parameter in this function refers to all strings in the closed interval from **user** to **home**. Does not contain the `$env:`.
    ///
    /// Design ideas explained:
    ///
    /// Split str first, terminator is `?` or `？`.(It depends on `separator`)
    /// Assume that the element of each iteration is x, and that the element of the previous iteration, after processing (e.g., passing in the value generated by env::var_os()) is acc.
    ///
    //// If acc is None, and x is empty. This means that the value of the previous environment variable does not exist, and this time a double check is required (not only must the previous env exist, but the path to it must also exist). Since the condition is not met, continue to the next iteration.
    ///
    /// If acc is None and x is not empty, then check the value of x's environment variable.
    ///
    /// If acc is Some, and x is not empty, exit the iterator. (If the value of the previous environment variable exists, we do not check the value of x this time, but return the value of the previous one)
    ///
    /// If acc is Some and x is empty, then determine if the file exists. If it does, we exit the iterator. If not, then acc is None.
    pub(crate) fn parse_dir_rules<F>(
        s: &str,
        f: F,
        separator: char, // Use a single char instead of pattern([char, char])
    ) -> ControlFlow<OsCow, OsCow>
    where
        F: Fn(&str) -> OsCow,
    {
        use ControlFlow::{Break, Continue};

        s.split_terminator(separator)
            .map(|x| x.trim())
            .try_fold(None, |acc: OsCow, x| match (acc, x.is_empty()) {
                (None, true) => Continue(None),
                (None, false) => Continue(f(x)),
                (p, false) => Break(p),
                (Some(p), true) => match Path::new(&p) {
                    x if x.exists() => Break(Some(p)),
                    _ => Continue(None),
                },
            })
    }

    /// The question mark is detected here for the same reason as the colon.
    ///
    /// If someone forgets to switch the Chinese input method to English, it is easy to type '?' as '？'.
    pub(crate) fn get_question_mark_separator(s: &str) -> char {
        let fq = FWQM;
        let hq = HWQM;
        match (s.find(hq), s.find(fq)) {
            (Some(h), Some(f)) if h < f => hq,
            (Some(h), Some(f)) if f < h => fq,
            (Some(_), None) => hq,
            (None, Some(_)) => fq,
            _ => ' ',
        }
    }

    /// This function is used to handle ident starting with `env *` or `env*`, and then resolve the environment variable to the right of `*`
    ///
    /// Assuming that the ident is `env * home`, it does not automatically convert `home` to `HOME`, but gets `$home` directly.
    pub(crate) fn handle_remix<'a>(s: &'a str, start: &str) -> OsCow<'a> {
        match s
            .trim_start_matches(start)
            .trim()
        {
            x if x.starts_with('*') => {
                let trimed = x.trim_start_matches('*').trim();
                match start {
                    "env" => Self::into_os_env(trimed),
                    #[cfg(feature = "base-dirs")]
                    "dir" => Self::match_base_dirs(trimed),
                    #[cfg(feature = "project-dirs")]
                    "proj" => match Self::get_chunks(trimed) {
                        c if matches!(c.len(), 0 | 1) => None,
                        c => match Self::set_proj_name_opt_tuple(c[0]) {
                            Some((name, proj)) => {
                                Self::match_proj_dirs(c[1], &name, proj.as_ref())
                            }
                            _ => None,
                        },
                    },
                    #[cfg(feature = "const-dirs")]
                    "const" => Self::match_const_dirs(trimed),
                    #[cfg(feature = "value")]
                    "val" => Self::match_values(trimed),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn starts_with_remix_expr(x: &str) -> bool {
        let mut iter = x.splitn(2, '*');

        match (iter.next(), iter.next()) {
            (Some(start), Some(_)) => Self::START_ARR
                .iter()
                .any(|s| start.starts_with(s)),
            _ => false,
        }
    }

    pub(crate) fn parse_remix_expr(x: &str) -> OsCow {
        Self::START_ARR
            .iter()
            // .inspect(|x| println!("in: {x}"))
            .filter(|&start| x.starts_with(start))
            // .inspect(|x| println!("out: {x}"))
            .find_map(|start| Self::handle_remix(x, start))
    }

    pub(crate) fn into_os_env(x: &str) -> OsCow {
        var_os(x).map(Cow::from)
    }

    fn match_os_env(ident: &str) -> OsCow {
        match ident {
            x if Self::starts_with_remix_expr(x) => {
                // dbg!("find start", x);
                Self::parse_remix_expr(x)
            }
            x => Self::into_os_env(x),
        }
    }

    /// For simple rules, get the environment variables directly.
    /// For complex rules, give them to `parse_dir_rules()`.
    pub(crate) fn handle_envs(ident: &str) -> OsCow {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(ident) {
            sep if sep == ' ' => var_os(ident).and_then(Self::into_os_cow),
            sep => match Self::parse_dir_rules(ident, Self::match_os_env, sep) {
                Break(x) | Continue(x) => x, // _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_complex_envs() {
        use crate::EnvPath;
        let s = EnvPath::create_from_str_iter([
            "$env       : user ?? aaa ? bbb ? ccc ? xdg_data_home ?? home",
            "$const   :    pkg",
            "$env   :      this_is-a-strange-env ?? to_avoid_conflicts_with_folder_names",
            "$env   ： xdg ？ uid ？ user",
        ]);
        dbg!(s.display());
    }

    #[test]
    fn find_the_first_colon() {
        let s = "$project(com.x)";
        dbg!(s.find('('));
    }

    #[test]
    fn split_to_two_chunks() {
        const CHUNK_NUM: usize = 2;
        // let s = " $env：xdg ： cfg";
        let s = " $env";
        let split_s = |c| {
            s.splitn(CHUNK_NUM, c)
                .map(|x| x.trim())
                .collect::<Vec<_>>()
        };
        dbg!(split_s('：'));
    }

    #[test]
    fn print_unicode_mark() {
        for i in ['?', '？', ' '].map(|x| x as u32) {
            println!(r#"\u{{{i:X}}}"#)
        }
    }
}
