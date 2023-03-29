//! # envpath
//!
//! A library for parsing and deserializing paths with special rules.
//!
//! ## Features:
//! - A struct [EnvPath](crate::EnvPath) for representing system paths. `raw` is the special rule path(vector), while `path` is the normal path after parsing. Since `Deref` is implemented, you can use it just like [Path](::std::path::Path) of std.
//!
//! The library also supports optional features for getting common system paths:
//! - `const-dirs` - Gets the value of some specific constants built into crate.
//! - `project-dirs` - For generating project directories (user-specific data dir)
//! - `base-dirs` - Provides standard directories on different platforms.
//!
//!
//! ## Serialization and deserialization
//!
//! If you want to serialize/deserialize a configuration file, you need to enable the `serde` feature of envpath and add serde, as well as other related dependencies.
//!
//! Next, we will add a `ron` dependency (You can actually use formats such as toml, yaml or json, but you need to add the relevant dependencies instead of using ron.)
//!
//! ```sh
//! cargo add envpath --features=serde
//! cargo add serde --features=derive
//! cargo add ron
//! ```
//!
//! ### Serialization
//!
//! Now let's try serialization.
//!
//! ```rust
//!         use serde::{Deserialize, Serialize};
//!         use envpath::EnvPath;
//!
//!         #[derive(Debug, Default, Serialize, Deserialize)]
//!         #[serde(default)]
//!         struct Cfg {
//!             dir: Option<EnvPath>,
//!         }
//!
//!         // If you want to skip the serialization, change the value of dir to None
//!         let dir = Some(EnvPath::from([
//!             "$env: user ?? userprofile ?? home",
//!         ]));
//!
//!         let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
//!         println!("{ron_str}");
//!
//!         std::fs::write("test.ron", ron_str)
//!             .expect("Failed to write the ron cfg to test.ron");
//! ```
//!
//! The output result is: `(dir: Some(["$env: user ?? userprofile ?? home"]))`
//!
//! It looks like the structure is the same as before serialization, except for the additional `dir` key.
//!
//! Yes, after serialization, it looks like that.
//!
//! This path format is suitable for cross-platform use.
//!
//! Since environment variables and other things may be dynamically changed.
//!
//! Keeping the raw format during serialization and obtaining its true path during deserialization is reasonable.
//!
//! ### Deserialization
//!
//! Next, let's try deserialization!
//!
//! ```rust
//!         use serde::{Deserialize, Serialize};
//!         use std::fs::File;
//!
//!         #[derive(Debug, Default, Serialize, Deserialize)]
//!         #[serde(default)]
//!         struct Cfg {
//!             dir: Option<EnvPath>,
//!         }
//!
//!         let cfg: Cfg = ron::de::from_reader(
//!             File::open("test.ron").expect("Failed to open the file: text.ron"),
//!         )
//!         .expect("Failed to deser ron cfg");
//!
//!         dbg!(&cfg);
//!
//!         if let Some(x) = cfg.dir {
//!             if x.exists() {
//!                 println!("{}", x.display())
//!             }
//!         }
//! ```
//!
//! For crates that support deserialization of toml, yaml, json and do not have a `from_reader()` method, you can use `from_str()`. I won't go into detail about it here.
//!
//! The output result of the above function is:
//!
//! ```rs
//! [src/lib.rs:116] &cfg = Cfg {
//!     dir: Some(
//!         EnvPath {
//!             raw: [
//!                 "$env: user ?? userprofile ?? home",
//!             ],
//!             path: Some(
//!                 "/home/m",
//!             ),
//!         },
//!     ),
//! }
//! /home/m
//! ```
//!
//! The `?` operator checks if a value exists. If it doesn't exist, continue checking. If it exists, use that value.
//!
//! On the other hand, the `??` operator requires both the value and the path to exist.
//!
//! For example, consider `$env: user ? userprofile`. Let's assume that the value of `user` is `m`, and `userprofile` is empty. Since the value of `user` exists, the expression returns `m`.
//!
//! If we change it to `$env: user ?? userprofile ? home`, even though the value of `user` exists, its path does not. So we continue checking. Then, since the value of `userprofile` does not exist, we continue checking until the condition is satisfied.
//!
//! `?` and `??` have different functions, and adding `??` does not mean that you can discard `?`. For values that are normal strings, such as `$const: os`, rather than paths, `?` is more useful than `??`. Each one has an important role to play.
//!
//! ## const
//!
//! Use `$const:name` (such as `$const:arch`) or `$const:alias` (e.g. `$const:architecture`) to obtain constant values. These values are obtained at compile time rather than runtime.
//!
//! | name          | alias        | From                    | example                 |
//! | ------------- | ------------ | ----------------------- | ----------------------- |
//! | pkg           | pkg-name     | `CARGO_PKG_NAME`        | envpath                 |
//! | ver           | pkg-version  | `CARGO_PKG_VERSION`     | `0.0.1-alpha.1`         |
//! | arch          | architecture | `consts::ARCH`          | x86_64, aarch64         |
//! | deb-arch      | deb_arch     | `get_deb_arch()`        | amd64, arm64            |
//! | os            |              | `consts::OS`            | linux, windows, android |
//! | family        |              | `consts::FAMILY`        | unix, windows           |
//! | exe_suffix    |              | `consts::EXE_SUFFIX`    | `.exe`, `.nexe`         |
//! | exe_extension |              | `consts::EXE_EXTENSION` | exe                     |
//! | empty         |              |                         | ""                      |
//!
//! ## remix
//!
//! | expr                        | example                          |
//! | --------------------------- | -------------------------------- |
//! | `env * [env_name]`          | `env * HOME`                     |
//! | `const * [const]`           | `const * arch`                   |
//! | `dir * [dir]`               | `dir * download`                 |
//! | `proj * (project): [ident]` | `proj * (com. xy.z): local-data` |
//!
//! ### example
//!
//! ```rs
//! ["
//!     $const: empty ??
//!         env * home ?
//!         env * HOME
//! ",
//!     "test"
//! ]
//! ```
//!
//! `env*` can be used for fallback, but unlike `$env:`, it does not automatically convert lowercase letters to uppercase, and it does not automatically convert `-` to `_`.
//!
//! - `env * home` retrieves `$home`, not `$HOME`.
//! - `$env: home` => `$HOME`
//! - `env * xdg-data-home` => `$xdg-data-home`, not `$XDG_DATA_HOME`
//! - `$env: xdg-data-home` => `$XDG_DATA_HOME`
//!
//! > Note: If the `$env:` expression contains a `*`, the automatic conversion feature will also be disabled.
//!
//! The following syntax is currently supported:
//!
//! - `$const: exe_suffix ?   env * HOME ?   env * XDG_DATA_HOME ?   env * EXE_SUFFIX`
//! - `$env: home ? xdg-data-home ? exe_suffix ?    const * exe_suffix`
//!
//! Not supported:
//!
//! - `$const: exe_suffix ? $env: home ? xdg-data-home ? exe_suffix`
//!
//! If it is supported, the parsing may become complicated and there could be confusion between `$env: exe_suffix` and `$const: exe_suffix`.
//!
//! ## dirs
//!
//! Here is an example of `$dir` for windows, for a full table, see Readme.
//!
//! These are some base-dirs, or you could say standard directories.
//! Use `$dir:name` (e.g. `$dir:dl`) or `$dir:alias` (e.g. `$dir:download`) to obtain the directory.
//! Many of these contents are obtained from [dirs](https://docs.rs/dirs/latest/dirs/), but there are also some additions.
//!
//!
//! | name                     | alias                    | Windows `$dir`                                                      |
//! | ------------------------ | ------------------------ | ------------------------------------------------------------------- |
//! | home                     |                          | `C:\Users\m`                                                        |
//! | cache                    |                          | `$localappdata`:(`$home\AppData\Local`)                             |
//! | cfg                      | config                   | `$appdata`: (`$home\AppData\Roaming`)                               |
//! | data                     |                          | `$home\AppData\Roaming`                                             |
//! | local-data               | local_data               | `$home\AppData\Local`                                               |
//! | local-cfg                | local_config             | `$home\AppData\Local`                                               |
//! | desktop                  |                          | `$home\Desktop`                                                     |
//! | doc                      | document                 | `$home\Documents`                                                   |
//! | dl                       | download                 | `$home\Downloads`                                                   |
//! | bin                      | exe                      | `$ms_dir\WindowsApps`                                               |
//! | first-path               | first_path               |                                                                     |
//! | last-path                | last_path                |                                                                     |
//! | font                     | typeface                 | `$ms_dir\Windows\Fonts`                                             |
//! | pic                      | picture                  | `$home\Pictures`                                                    |
//! | pref                     | preference               | `$home\AppData\Roaming`                                             |
//! | pub                      | public                   | `$home\Public`                                                      |
//! | runtime                  |                          | None                                                                |
//! | state                    |                          | None                                                                |
//! | video                    |                          | `$home\Videos`                                                      |
//! | music                    | audio                    | `$home\music`                                                       |
//! | template                 |                          | `$ms_dir\Windows\Templates`                                         |
//! | tmp                      |                          | `$tmpdir`                                                           |
//! | tmp-rand                 | tmp_random               | `$tmpdir\[pkg-name]_$random`                                        |
//! | temp                     | temporary                | `env::temp_dir()`                                                   |
//! | cli-data                 | cli_data                 | `$home\AppData\Local`                                               |
//! | cli-cfg                  | cli_config               | `$home\AppData\Local`                                               |
//! | cli-cache                | cli_cache                | `$home\AppData\Local`                                               |
//! | progam-files             | program_files            | `$ProgramFiles`: (`C:\Program Files`)                               |
//! | program-files-x86        | program_files_x86        | `$ProgramFiles(x86)`: (`C:\Program Files (x86)`)                    |
//! | common-program-files     | common_program_files     | `$CommonProgramFiles`: (`C:\Program Files\Common Files`)            |
//! | common-program-files-x86 | common_program_files_x86 | `$CommonProgramFiles(x86)`: (`C:\Program Files (x86)\Common Files`) |
//! | program-data             | program_data             | `$ProgramData`: (`C:\ProgramData`)                                  |
//! | microsoft                |                          | `$home\AppData\Roaming\Microsoft`                                   |
//! | local-low                | local_low                | `$home\AppData\LocalLow`                                            |
//! | empty                    |                          | ""                                                                  |
//!
//!
//! Note: `project_dirs` has more advanced features.
//! Here are some simple introductions.
//!
//! For example, `$proj(com. x. y): data` will generate a `data` directory for this project (It does not create it automatically, just generates its value).
//!
//! - On Android, it is `/data/data/com.x.y`
//! - On macOS, it is `/Users/[username]/Library/Application Support/com.x.y`
use std::{self, borrow::Cow, ffi::OsStr};

/// Type alias `OsCow` for handling OS Strings assigned to the heap or the stack.
pub type OsCow<'a> = Option<Cow<'a, OsStr>>;

pub use envpath_core::EnvPath;

mod deref;
pub mod envpath_core;
mod os_env;
mod parser;

#[cfg(feature = "const-dirs")]
pub mod arch;

#[cfg(feature = "const-dirs")]
mod const_dirs;

#[cfg(feature = "project-dirs")]
mod project_dirs;

#[cfg(feature = "base-dirs")]
mod base_dirs;

#[cfg(feature = "serde")]
mod serialisation;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    /// Testing deserialization with RON format
    fn deser_ron() {
        // use crate::envpath_core::EnvPath;
        use serde::{Deserialize, Serialize};

        // Struct with data_dir field, of Option EnvPath type
        #[derive(Serialize, Debug, Deserialize)]
        struct Cfg {
            data_dir: Option<EnvPath>,
        }

        // Sample string to be deserialized
        let str = r#"["$env: xdg_data_home", "$const: os", "$const: pkg"]"#;

        // Convert the string to EnvPath struct using RON format
        let path = ron::from_str::<EnvPath>(str).unwrap();

        // Print the path in display format
        println!("{}", path.display());

        // If the path does not exist, error message will be printed
        if !path.exists() {
            eprintln!("Err: File does not exist")
        }
    }

    #[test]
    fn readme_doc_quick_start_0() {
        let v =
            EnvPath::from(["$dir: data", "$const: pkg", "$env: test_qwq", "app"])
                .de();
        dbg!(v.display(), v.exists());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn readme_doc_quick_start_serialisation() {
        use serde::{Deserialize, Serialize};

        // Struct with dir field, of Option EnvPath type
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg {
            dir: Option<EnvPath>,
        }

        let dir = Some(EnvPath::from(&["$env: user ?? userprofile ?? home"]));

        let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
        std::fs::write("test.ron", ron_str)
            .expect("Failed to write the ron cfg to test.ron");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn readme_doc_quick_start_serialisation_deser() {
        use serde::{Deserialize, Serialize};
        use std::fs::File;

        // Struct with dir field, of Option EnvPath type
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg {
            dir: Option<EnvPath>,
        }

        let cfg: Cfg = ron::de::from_reader(
            File::open("test.ron").expect("Failed to open the file: text.ron"),
        )
        .expect("Failed to deser ron cfg");

        if let Some(x) = &cfg.dir {
            if x.exists() {
                println!("{}", x.display())
            }
        }
        dbg!(&cfg);
    }
}
