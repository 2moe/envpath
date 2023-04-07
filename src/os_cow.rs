use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
};
pub const AND_SD: &str = "/storage/self/primary";

/// Type alias `OsCow` for handling OS Strings assigned to the heap or the stack.
pub type OsCow<'a> = Option<Cow<'a, OsStr>>;

// pub(crate) fn from_os_str(s: &OsStr) -> OsCow {
//     Some(Cow::from(s))
// }

/// Converts the given string into an `OsCow` object.
///
/// # Examples
///
///```no_run
/// use envpath::os_cow;
/// use std::{borrow::Cow, ffi::OsStr};
///
/// let str = "/usr/bin";
/// let os_cow = os_cow::from_str(str);
///
/// assert_eq!(os_cow, Some(Cow::from(OsStr::new(str))));
///```
pub(crate) fn from_str(s: &str) -> OsCow {
    Some(Cow::from(OsStr::new(s)))
}

/// Converts the given Path/OsStr into an `OsCow` object.
///
/// # Examples
///
///```no_run
///  use std::{
///      borrow::Cow,
///      path::{Path, PathBuf},
///  };
///  let path = Path::new("/usr/bin");
///  let os_cow = envpath::os_cow::into_os_cow(path);
///  let path_cow = Cow::from(path.as_os_str());
///  assert_eq!(os_cow, Some(path_cow));
///  let pathbuf = PathBuf::from("/usr/bin");
///  let cow_os_string = Cow::from(pathbuf.into_os_string());
///  assert_eq!(os_cow, Some(cow_os_string));
///```
pub(crate) fn into_os_cow<'a, I: Into<OsString>>(s: I) -> OsCow<'a> {
    Some(Cow::from(s.into())) // Converts the input into an OsString and wraps it in a Cow object
}

/// Join a specific string with the SD directory.
///
/// # Examples
///
/// ```no_run
/// use envpath::EnvPath;
///
/// let android_dir = EnvPath::os_cow::set_android_dir("Android/obb/com.x.y/");
///
/// assert_eq!(
///     android_dir,
///     Some(std::borrow::Cow::from(OsStr::new(
///         "/storage/self/primary/Android/obb/com.x.y/"
///     )))
/// );
/// ```
#[cfg(target_os = "android")]
pub(crate) fn set_android_dir(s: &str) -> OsCow {
    into_os_cow(std::path::Path::new(AND_SD).join(s))
}

#[cfg(test)]
mod tests {
    // use ron::from_str;

    #[test]
    fn into_os_cow_doc() {
        use std::{
            borrow::Cow,
            path::{Path, PathBuf},
        };

        let path = Path::new("/usr/bin");
        let os_cow = crate::os_cow::into_os_cow(path);

        let path_cow = Cow::from(path.as_os_str());
        assert_eq!(os_cow, Some(path_cow));

        let pathbuf = PathBuf::from("/usr/bin");
        let cow_os_string = Cow::from(pathbuf.into_os_string());
        assert_eq!(os_cow, Some(cow_os_string));
    }

    #[test]
    fn os_cow_doc() {
        use std::{borrow::Cow, ffi::OsStr};

        let str = "/usr/bin";
        let os_cow = crate::os_cow::from_str(str);

        assert_eq!(os_cow, Some(Cow::from(OsStr::new(str))));
    }

    #[cfg(target_os = "android")]
    #[test]
    fn set_android_dir_doc() {
        use std::ffi::OsStr;

        let android_dir = os_cow::set_android_dir("Android/obb/com.x.y/");

        assert_eq!(
            android_dir,
            Some(std::borrow::Cow::from(OsStr::new(
                "/storage/self/primary/Android/obb/com.x.y/"
            )))
        );
    }
}
