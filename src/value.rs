use crate::{EnvPath, OsCow};
use std::ops::ControlFlow;

impl EnvPath<'_> {
    /// This function is used to resolve ident in `$val: ident`.
    /// Unlike `$const:`, most of the values here are obtained at runtime.
    pub(crate) fn match_values(ident: &str) -> OsCow {
        match ident {
            "empty" => crate::os_cow::from_str(""),
            #[cfg(feature = "rand")]
            x if x.starts_with("rand-") => {
                let u = x
                    .split_once('-')
                    .map(|x| x.1)
                    .and_then(|x| x.parse::<usize>().ok());
                crate::os_cow::into_os_cow(crate::random::get_random_value(u))
            }
            x if Self::starts_with_remix_expr(x) => Self::parse_remix_expr(x),
            _ => None,
        }
    }

    pub(crate) fn handle_values(ident: &str) -> OsCow {
        use ControlFlow::{Break, Continue};

        match Self::get_question_mark_separator(ident) {
            sep if sep == ' ' => Self::match_values(ident),
            sep => match Self::parse_dir_rules(ident, Self::match_values, sep) {
                Break(x) | Continue(x) => x,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(feature = "consts")]
    fn test_value() {
        let v = EnvPath::from(["$val: empty ??  rand-2"]);
        dbg!(v.de().display());

        let p = EnvPath::new(["$const: empty ?? val * rand-33"]);
        dbg!(p.display());
    }
}
