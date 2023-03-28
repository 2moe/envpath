//! A library for parsing and deserializing paths with special rules.
//!
//! The features:
//! - A struct [EnvPath](crate::EnvPath) for representing system paths. `raw` is the special rule path(vector), while `path` is the normal path after parsing. Since `Deref` is implemented, you can use it just like [Path](::std::path::Path) of std.
//!
//! The library also supports optional features for getting common system paths:
//! - `const-dirs` - Gets the value of some specific constants built into crate.
//! - `project-dirs` - For generating project directories (user-specific data dir)
//! - `base-dirs` - Provides standard directories on different platforms.
//!
//! Here is an example of `$dir` for windows, for a full table, see Readme.
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
//! | tmp                      |                          |                                                                     |
//! | temp                     | temporary                |                                                                     |
//! | cli-data                 | cli_data                 | `$home\AppData\Local`                                               |
//! | cli-cfg                  | cli_config               | `$home\AppData\Local`                                               |
//! | cli-cache                | cli_cache                | `$home\AppData\Local`                                               |
//! | progam-files             | program_files            | `$ProgramFiles`: (`C:\Program Files`)                               |
//! | program-files-x86        | program_files_x86        | `$ProgramFiles(x86)`: (`C:\Program Files (x86)`)                    |
//! | common-program-files     | common_program_files     | `$CommonProgramFiles`: (`C:\Program Files\Common Files`)            |
//! | common-program-files-x86 | common_program_files_x86 | `$CommonProgramFiles(x86)`: (`C:\Program Files (x86)\Common Files`) |
//! | program-data             | program_data             | `$ProgramData`: (`C:\ProgramData`)                                  |
//! | microsoft                |                          | `$home\AppData\Local\Roaming\Microsoft`                             |
//! | local-low                | local_low                | `$home\AppData\LocalLow`                                            |
//!
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
