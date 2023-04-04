/*!
# envpath

A library for parsing and deserializing paths with special rules.

## Features:
- A struct [EnvPath](crate::EnvPath) for representing system paths. `raw` is the special rule path(vector), while `path` is the normal path after parsing. Since `Deref` is implemented, you can use it just like [Path](::std::path::Path) of std.

The library also supports optional features for getting common system paths:
- `consts` - Gets the value of some specific constants built into crate.
- `project` - For generating project directories (user-specific data dir)
- `dirs` - Provides standard directories on different platforms.


## Serialization and deserialization

If you want to serialize/deserialize a configuration file, you need to enable the `serde` feature of envpath and add serde, as well as other related dependencies.

Next, we will add a `ron` dependency (You can actually use formats such as yaml or json, but you need to add the relevant dependencies instead of using ron.)

```sh
cargo add envpath --features=serde
cargo add serde --features=derive
cargo add ron
```

### Serialization

Now let's try serialization.

```rust
        use envpath::EnvPath;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg<'a> {
            dir: Option<EnvPath<'a>>,
        }

        let dir = Some(EnvPath::from([
            "$env: user ?? userprofile ?? home",
        ]));

        let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
        println!("{ron_str}");

        std::fs::write("test.ron", ron_str)
            .expect("Failed to write the ron cfg to test.ron");
```

The output result is: `(dir: Some(["$env: user ?? userprofile ?? home"]))`

It looks like the structure is the same as before serialization, except for the additional `dir` key.

Yes, after serialization, it looks like that.

This path format is suitable for cross-platform use.

Since environment variables and other things may be dynamically changed.

Keeping the raw format during serialization and obtaining its true path during deserialization is reasonable.

If you want to save performance overhead, you can change the value of `dir` to `None`, and serde should skip it during serialization.

### Deserialization

Next, let's try deserialization!

```rust
        use envpath::EnvPath;
        use serde::{Deserialize, Serialize};
        use std::fs::File;

        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg<'a> {
            dir: Option<EnvPath<'a>>,
        }

        let cfg: Cfg = ron::de::from_reader(
            File::open("test.ron").expect("Failed to open the file: text.ron"),
        )
        .expect("Failed to deser ron cfg");

        dbg!(&cfg);

        if let Some(x) = cfg.dir {
            if x.exists() {
                println!("{}", x.display())
            }
        }
```

The output result of the above function is:

```rs
[src/lib.rs:116] &cfg = Cfg {
    dir: Some(
        EnvPath {
            raw: [
                "$env: user ?? userprofile ?? home",
            ],
            path: Some(
                "/home/m",
            ),
        },
    ),
}
/home/m
```

The `?` operator checks if a value exists. If it doesn't exist, continue checking. If it exists, use that value.

On the other hand, the `??` operator requires both the value and the path to exist.

For example, consider `$env: user ? userprofile`. Let's assume that the value of `user` is `m`, and `userprofile` is empty. Since the value of `user` exists, the expression returns `m`.

If we change it to `$env: user ?? userprofile ? home`, even though the value of `user` exists, its path does not. So we continue checking. Then, since the value of `userprofile` does not exist, we continue checking until the condition is satisfied.

`?` and `??` have different functions, and adding `??` does not mean that you can discard `?`. For values that are normal strings, such as `$const: os`, rather than paths, `?` is more useful than `??`. Each one has an important role to play.

## const

Use `$const:name` (such as `$const:arch`) or `$const:alias` (e.g. `$const:architecture`) to obtain constant values. These values are obtained at compile time rather than runtime.

| name          | alias        | From                    | example                 |
| ------------- | ------------ | ----------------------- | ----------------------- |
| arch          | architecture | `consts::ARCH`          | x86_64, aarch64         |
| deb-arch      | deb_arch     | `get_deb_arch()`        | amd64, arm64            |
| os            |              | `consts::OS`            | linux, windows, android |
| family        |              | `consts::FAMILY`        | unix, windows           |
| exe_suffix    |              | `consts::EXE_SUFFIX`    | `.exe`, `.nexe`         |
| exe_extension |              | `consts::EXE_EXTENSION` | exe                     |
| empty         |              |                         | ""                      |

## val

> The `value` feature needs to be enabled.

Use `$val:name` (e.g. `$val: rand-16`) to obtain the values. Unlike `$const:`, most of the values here are obtained at runtime.

| name           | expr            | example          |
| -------------- | --------------- | ---------------- |
| `rand-[usize]` | `$val: rand-16` | 90aU0QqYnx1gPEgN |
| empty          | `$val: empty`   | ""               |

> `$val: rand-[usize]` syntax requires the `rand` feature to be enabled.

rand is used to obtain random content, and currently only supports strings.

## remix

| syntax                      | expr                            | example                              |
| --------------------------- | ------------------------------- | ------------------------------------ |
| `env * [env_name]`          | `env * HOME`                    | `C:\Users\m`                         |
| `const * [const]`           | `const * arch`                  | `x86_64`                             |
| `dir * [dir]`               | `dir * dl`                      | `C:\Users\m\Downloads`               |
| `proj * (project): [ident]` | `proj * (com.xy.z): local-data` | `C:\Users\m\AppData\Local\xy\z\data` |
| `val * [val]`               | `val * rand-32`                 | VcNCQy5IevkuoQKm70arpRpAC5QGtF9D     |

### example

```rs
["
    $const: empty ??
        env * home ?
        env * HOME
",
    "test"
]
```

`env*` can be used for fallback, but unlike `$env:`, it does not automatically convert lowercase letters to uppercase, and it does not automatically convert `-` to `_`.

- `env * home` retrieves `$home`, not `$HOME`.
- `$env: home` => `$HOME`
- `env * xdg-data-home` => `$xdg-data-home`, not `$XDG_DATA_HOME`
- `$env: xdg-data-home` => `$XDG_DATA_HOME`

> Note: If the `$env:` expression contains a `*`, the automatic conversion feature will also be disabled.

The following syntax is currently supported:

- `$const: exe_suffix ?   env * HOME ?   env * XDG_DATA_HOME ?   env * EXE_SUFFIX`
- `$env: home ? xdg-data-home ? exe_suffix ?    const * exe_suffix`

Not supported:

- `$const: exe_suffix ? $env: home ? xdg-data-home ? exe_suffix`

If it is supported, the parsing may become complicated and there could be confusion between `$env: exe_suffix` and `$const: exe_suffix`.

## dirs

Here is an example of `$dir` for windows, for a full table, see Readme.

These are some base-dirs, or you could say standard directories.
Use `$dir:name` (e.g. `$dir:dl`) or `$dir:alias` (e.g. `$dir:download`) to obtain the directory.
Many of these contents are obtained from [dirs](https://docs.rs/dirs/latest/dirs/), but there are also some additions.


| name                     | alias                    | Windows `$dir`                                                      |
| ------------------------ | ------------------------ | ------------------------------------------------------------------- |
| home                     |                          | `C:\Users\m`                                                        |
| cache                    |                          | `$localappdata`:(`$home\AppData\Local`)                             |
| cfg                      | config                   | `$appdata`: (`$home\AppData\Roaming`)                               |
| data                     |                          | `$home\AppData\Roaming`                                             |
| local-data               | local_data               | `$home\AppData\Local`                                               |
| local-cfg                | local_config             | `$home\AppData\Local`                                               |
| desktop                  |                          | `$home\Desktop`                                                     |
| doc                      | document                 | `$home\Documents`                                                   |
| dl                       | download                 | `$home\Downloads`                                                   |
| bin                      | exe                      | `$ms_dir\WindowsApps`                                               |
| first-path               | first_path               |                                                                     |
| last-path                | last_path                |                                                                     |
| font                     | typeface                 | `$ms_dir\Windows\Fonts`                                             |
| pic                      | picture                  | `$home\Pictures`                                                    |
| pref                     | preference               | `$home\AppData\Roaming`                                             |
| pub                      | public                   | `$home\Public`                                                      |
| runtime                  |                          | None                                                                |
| state                    |                          | None                                                                |
| video                    |                          | `$home\Videos`                                                      |
| music                    | audio                    | `$home\Music`                                                       |
| template                 |                          | `$ms_dir\Windows\Templates`                                         |
| tmp                      |                          | `$tmpdir`                                                           |
| tmp-rand                 | tmp_random               | `$tmpdir\[random]`                                        |
| temp                     | temporary                | `env::temp_dir()`                                                   |
| cli-data                 | cli_data                 | `$home\AppData\Local`                                               |
| cli-cfg                  | cli_config               | `$home\AppData\Local`                                               |
| cli-cache                | cli_cache                | `$home\AppData\Local`                                               |
| progam-files             | program_files            | `$ProgramFiles`: (`C:\Program Files`)                               |
| program-files-x86        | program_files_x86        | `$ProgramFiles(x86)`: (`C:\Program Files (x86)`)                    |
| common-program-files     | common_program_files     | `$CommonProgramFiles`: (`C:\Program Files\Common Files`)            |
| common-program-files-x86 | common_program_files_x86 | `$CommonProgramFiles(x86)`: (`C:\Program Files (x86)\Common Files`) |
| program-data             | program_data             | `$ProgramData`: (`C:\ProgramData`)                                  |
| microsoft                |                          | `$home\AppData\Roaming\Microsoft`                                   |
| local-low                | local_low                | `$home\AppData\LocalLow`                                            |
| empty                    |                          | ""                                                                  |


Note: `project_dirs` has more advanced features.
Here are some simple introductions.

For example, `$proj(com. x. y): data` will generate a `data` directory for this project (It does not create it automatically, just generates its value).

- On Android, it is `/data/data/com.x.y`
- On macOS, it is `/Users/[username]/Library/Application Support/com.x.y`
*/
use std::{self, path::PathBuf};

mod deref;
mod from;
mod os_cow;
mod os_env;
mod parser;
mod raw;

pub use os_cow::OsCow;
pub use raw::EnvPathRaw as Raw;

#[cfg(feature = "consts")]
pub mod consts;

#[cfg(feature = "project")]
mod project;

#[cfg(feature = "project")]
pub use directories::ProjectDirs;

#[cfg(feature = "dirs")]
pub mod dirs;

#[cfg(feature = "serde")]
mod serialisation;

#[cfg(feature = "value")]
mod value;

#[cfg(feature = "rand")]
pub mod random;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct EnvPath<'r> {
    pub(crate) raw: Raw<'r>,
    pub path: Option<PathBuf>,
}
