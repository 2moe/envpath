# EnvPath

[![Documentation](https://docs.rs/envpath/badge.svg)](https://docs.rs/envpath)

[![Apache-2 licensed](https://img.shields.io/crates/l/envpath.svg)](./License)

A library for **parsing** and **deserialising** paths with special rules.

The format is similar to `["$proj(com.xy.z): data ? cfg", "$const: pkg", "$const: deb-arch"]`

> Maybe I should change it to **deserializing**.  
> Never mind all the details, let's get started!

[中文](Readme-zh.md)

## preface

Note: This readme contains a lot of non-technical content.

Before the official start, if it's not too much trouble, could you provide me with an answer to my query?

How have we been solving the problem of cross-platform path configuration?

Assume the following configuration.

```toml
[dir]
data = "C:\\Users\\[username]\\AppData\\Roaming\\[dirname]"
```

Perhaps we would create a new Map (e.g. `HashMap<String, Dirs>`) for the configuration file, allowing different platforms to use different configurations.

```toml
[dir.linux]
data = "/home/[username]/.local/share/[appname]"
cache = "/home/[username]/.cache/[app]"

[dir.macos]
data = "/Users/[username]/Library/Application Support/x.y.z"

[dir.xxxbsd]

[dir.your-os-name]
```

This is a good approach, but is there a more universal method?

So people thought of using environment variables.

I guess you're thinking of the XDG specification. Since it's so useful, why don't we use `$XDG_DATA_HOME/[appname]/` on all platforms?

Unfortunately, not all platforms support it, so we choose the more universal `$HOME`.  
Sadly, on early versions of Windows, there might not be `%HOME%`, but only `%userprofile%`.  
So, what should we do? And how do we do it?  
We can use external crates to automatically retrieve paths for different platforms or manually write different directory mapping relationships for different platforms.  
Great, it seems like we have solved the cross-platform issue, but we may have forgotten one thing.
That is, the path separators on different platforms may be different.  
We can generate paths for different platforms, but the format generated may not be very universal.

- Windows: `C:\path\to\xxx`
- Unix-like: `/path/to/xxx`

### Path Separator

The following are some additions.

> The path separator in the Macintosh operating system has undergone several changes throughout its history. In the early versions of the Macintosh operating system, the path separator was a forward slash (/). However, with the introduction of the Hierarchical File System (HFS) in 1985, the path separator was switched to a colon (:).  
> With the release of the macOS operating system in 2001, the HFS+ file system was introduced, and the path separator remained a colon (:). However, as of macOS Catalina (10.15), Apple has introduced a new read-only file system called APFS, which uses a forward slash (/) as the path separator.  
> According to the Apple Technical Note TN1150: HFS Plus Volume Format, the use of the colon as the path separator in the HFS file system was intended to make the Macintosh operating system more user-friendly by allowing users to easily navigate through directories. The switch to the forward slash in APFS is likely due to its compatibility with other Unix-based systems.  
> Source: [Apple Technical Note TN1150](https://developer.apple.com/library/archive/technotes/tn/tn1150.html)

The following is a table of separators.

| Operating System               | Company                       | Path Separator          |
| ------------------------------ | ----------------------------- | ----------------------- |
| Windows                        | Microsoft                     | Backslash (`\`)         |
| Some Unix-like(e.g. GNU/Linux) | N/A (open source)             | Forward Slash (/)       |
| mac (early early)              | Apple                         | Forward Slash (/)       |
| mac (early)                    | Apple                         | Colon (:)               |
| mac (current)                  | Apple                         | Forward Slash (/)       |
| MS-DOS                         | Microsoft                     | Backslash (`\`)         |
| CP/M                           | Digital Research              | Forward Slash (/)       |
| VMS                            | Digital Equipment Corporation | Brackets ([ ])          |
| IBM OS/2                       | IBM                           | Backslash (`\`)         |
| PrimeOS                        | Prime Computer                | Caret (^)               |
| Virtuozzo                      | Virtuozzo International GmbH  | Double colon (::)       |
| VOS                            | Stratus Technologies          | Right Angle Bracket (>) |
| RISC OS                        | Acorn Computers               | Full Stop (.)           |
| AmigaOS                        | Commodore International       | Colon (:)               |
| TOPS-20                        | Digital Equipment Corporation | Forward Slash (/)       |
| Plan 9                         | Bell Labs                     | Forward Slash (/)       |
| Inferno                        | Bell Labs                     | Forward Slash (/)       |
| ZX Spectrum                    | Sinclair Research             | Backslash (`\`)         |

Note: some operating systems may change their file path separators between versions.
I can't guarantee that the table above is exactly correct, so if something goes wrong, report an issue and let me change it.

---

Since different platforms use different path separators, how can we make them look the same?

The answer is to use an array (or vector).

However, its disadvantages are quite obvious. For ordinary users, this format may be harder to read than a string (although developers may prefer the former). Please note that user configuration files are for users to see, not just for deserialization, so readability is crucial.

For example, `["C:", "\\", "Users", "Public"]` is equivalent to `C:\Users\Public`. There's a small detail that's easy to overlook, which is that the second element is "\\".

EnvPath (raw) also uses an array structure (actually a vector), but with special rules.

For example, `["$dir: dl ? doc"]` specifies the Downloads directory (which has different paths on different platforms), and if the Downloads directory does not exist, it will use the Documents directory instead.
Note: A single "?" and a double "?" are different, as we will mention later.

After saying a lot of irrelevant things, let's get started!

## Quick Start

Before we start, please make sure that the Rust version is not too old.
Because this library uses some relatively new syntax, such as let-else (which requires 1.65+).

### Basic guide

First, we need to add the dependency.

```sh
cargo add envpath --no-default-features --features=base-dirs,const-dirs,project-dirs
```

Then add the following content to our `main()` or test function.

```rust
use envpath::EnvPath;

let v = EnvPath::from(["$dir: data", "$const: pkg", "$env: test_qwq", "app"]).de();
dbg!(v.display(), v.exists());
```

This is a simple example, and there are more features and concepts that we haven't mentioned here.

Don't worry, take it step by step.

It will then output something like the following.

```js
[src/lib.rs:74] v.display() = "/home/m/.local/share/envpath/$env: test_qwq/app"
[src/lib.rs:74] v.exists() = false
```

We can see that `$env: test_qwq` was not resolved successfully.

So what happened? Is it malfunctioning?

No, it's not. This is a deliberate design decision. When EnvPath was designed, it was intentionally made compatible with regular paths.

If one day, EnvPath adds a feature that requires the prefix `$recycle` and includes the keyword `bin` to resolve to a specific directory. And on your system disk, there happens to be a `$RECYCLE:BIN` folder, and unfortunately, the file system on that disk has not enabled case sensitivity. When there is a collision with a same-named path, it will first try to resolve it, and if it fails, it will assume that the same-named path exists and return it.

The probability of collision with the same-named path exists, but with a little bit of skill, most collision events can be avoided.

> Trick: Use whitespace characters (spaces, line breaks, tabs, etc.) and use `?`(will be introduced below)
>
> For example, `$env: test_qwq` can be written as `$env       ：          test-QwQ`
>
> Although many spaces have been added, if successful, they will be resolved to the same value. Using the posix sh on unix to describe the above expression is: `$TEST_QWQ` (i.e. all lowercase letters are changed to uppercase, and all `-` are changed to `_`)
>
> Although you may find this approach difficult to accept, it is customary for global system environment variables, and I have not created any new rules.

Since the resolution failed, why not return an empty directory?

Let's take an example with the env command in posix sh!

Assuming the directory you want to access is `$XDG_DATA_HOME/app`, if the relevant env is empty, then what you are accessing is /app, which is different from the expected result. (~~I want to go home, but I bought the wrong train ticket 🎫~~

You may argue: I can use `${ENV_NAME:-FALLBACK}` to specify the fallback! It's clearly because you're just not smart enough.

However, sometimes a careless mistake can lead to a big problem. I think complaining less will make life more beautiful.

At this point, you may have forgotten where the error occurred earlier: `$env: test_qwq`.  
So how do we solve it? You can try changing it to `$env: test_qwq ? user ? logname`, or add more question marks and valid environment variable names.

~~I won't explain the function of `?` here, go explore it yourself, often you will discover more fun.~~

---

Going back to the code we mentioned earlier, let's simplify it a bit.
`EnvPath::from(["$dir: data"]).de();`

As is well known, `[]` is an array. But what exactly is `.de()`?

In Chinese, if we use "de" to refer to a country, it means Germany. Written in Chinese characters, it is "德". If used to describe a person, it can mean that he has "noble character".

Ohhhh! I got it. This function took a trip to Germany (de), so it changed and became a function with noble character.

Anyway, I think you're very smart, and this function did indeed change.  
But it only converts a structure like `$env: QuQ? ?? qwq-dir? AwA-home` into another value.

### Serialization and deserialization

If you want to serialize/deserialize a configuration file, you need to enable the `serde` feature of envpath and add serde, as well as other related dependencies.

Next, we will add a `ron` dependency (You can actually use formats such as toml, yaml or json, but you need to add the relevant dependencies instead of using ron.)

```sh
cargo add envpath --features=serde
cargo add serde --features=derive
cargo add ron
```

Now let's try serialization.

```rust
        use serde::{Deserialize, Serialize};
        use envpath::EnvPath;

        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg {
            dir: Option<EnvPath>,
        }

        let dir = Some(EnvPath::from(&[
            "$env: user ?? userprofile ?? home",
        ]));

        let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
        println!("{ron_str}");

        std::fs::write("test.ron", ron_str)
            .expect("Failed to write the ron cfg to test.ron");
```

We first defined a `Cfg` struct, created a new `EnvPath` instance, wrapped `dir` in `Cfg`, serialized it with `ron`, and finally wrote it to `test.ron`.

The output result is: `(dir: Some(["$env: user ?? userprofile ?? home"]))`

It looks like the structure is the same as before serialization, except for the additional `dir` key.

Yes, after serialization, it looks like that.

This path format is suitable for cross-platform use.

Since environment variables and other things may be dynamically changed.

Keeping the raw format during serialization and obtaining its true path during deserialization is reasonable.

If you want to save performance overhead, you can change the value of `dir` to `None`, and serde should skip it during serialization.

Next, let's try deserialization!

```rust
        use serde::{Deserialize, Serialize};
        use std::fs::File;

        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg {
            dir: Option<EnvPath>,
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

For crates that support deserialization of toml, yaml, json and do not have a `from_reader()` method, you can use `from_str()`. I won't go into detail about it here.

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

That concludes the basic guide.
The above describes some basic features.

`project_dirs` has more advanced features. Here are some simple introductions. For example, `$proj(com.macro-hard.app-name): data` will generate a `data` directory for this project (It does not create it automatically, just generates its value).

> `M$`, his smile was as wide as the Grand Canyon, but behind it lurked a simmering rage that could rival a volcano as he approached me and asked a question as sweet as honey on a summer day.
>
> Sorry, please forgive me.

Now, it is `$proj(com. x. y): data`.

- On Android, it is `/data/data/com.x.y`
- On macOS, it is `/Users/[username]/Library/Application Support/com.x.y`

After learning the basic usage, we will continue to introduce and supplement more content.

- The simplest: const-dirs
- Common standard directories: base-dirs
- Advanced project directories: project-dirs

In the following text, we will introduce what their functions are and what values they all have.

## Features

### env

In the previous text, we have learned about the basic usage. Here are a few more things to explain.

"env" refers to environment variables. `$env:home` is used to obtain the value of the HOME environment variable. `$env:xdg-data-home` is equivalent to `$XDG_DATA_HOME`.

As for the use of "?", you can refer to the previous text.  
When you understand the purpose of `$env:userprofile ?? QwQ-Dir ? LocalAppData ? home`, then congratulations, you have learned how to use env!

### const

Use `$const:name` (such as `$const:arch`) or `$const:alias` (e.g. `$const:architecture`) to obtain constant values. These values are obtained at compile time rather than runtime.

| name          | alias        | From                    | example                 |
| ------------- | ------------ | ----------------------- | ----------------------- |
| pkg           | pkg-name     | `CARGO_PKG_NAME`        | envpath                 |
| ver           | pkg-version  | `CARGO_PKG_VERSION`     | `0.0.1-alpha.1`         |
| arch          | architecture | `consts::ARCH`          | x86_64, aarch64         |
| deb-arch      | deb_arch     | `get_deb_arch()`        | amd64, arm64            |
| os            |              | `consts::OS`            | linux, windows, android |
| family        |              | `consts::FAMILY`        | unix, windows           |
| exe_suffix    |              | `consts::EXE_SUFFIX`    | `.exe`, `.nexe`         |
| exe_extension |              | `consts::EXE_EXTENSION` | exe                     |
| empty         |              |                         | ""                      |

#### deb-arch

The following table shows the possible output values for `$const:deb-arch`:

| Architecture                | deb_arch                                                                            |
| --------------------------- | ----------------------------------------------------------------------------------- |
| x86_64                      | amd64                                                                               |
| aarch64                     | arm64                                                                               |
| riscv64 (riscv64gc)         | riscv64                                                                             |
| arm (feature = `+vfpv3`)    | armhf                                                                               |
| arm                         | armel                                                                               |
| mips (endian = little)      | mipsel                                                                              |
| mips64 (endian = little)    | mips64el                                                                            |
| s390x                       | s390x                                                                               |
| powerpc64 (endian = little) | ppc64el                                                                             |
| x86 (i586/i686)             | i386                                                                                |
| other                       | [consts::ARCH](https://doc.rust-lang.org/nightly/std/env/consts/constant.ARCH.html) |

For example, if you compile a package for `armv7`, the value obtained by `$const:arch` would be `arm`, while `$const:deb-arch` could be `armhf`.

### remix

| expr                        | example                          |
| --------------------------- | -------------------------------- |
| `env * [env_name]`          | `env * HOME`                     |
| `const * [const]`           | `const * arch`                   |
| `dir * [dir]`               | `dir * download`                 |
| `proj * (project): [ident]` | `proj * (com. xy.z): local-data` |

#### example

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

### base

These are some base-dirs, or you could say standard directories.  
Use `$dir:name` (e.g. `$dir:dl`) or `$dir:alias` (e.g. `$dir:download`) to obtain the directory.  
Many of these contents are obtained from [dirs](https://docs.rs/dirs/latest/dirs/), but there are also some additions.

#### Linux

| name       | alias        | Linux `$dir`                             |
| ---------- | ------------ | ---------------------------------------- |
| home       |              | `$home`: (/home/m)                       |
| cache      |              | `$xdg_cache_home`:(`$home/.cache`)       |
| cfg        | config       | `$xdg_config_home`:(`$home/.config`)     |
| data       |              | `$xdg_data_home`:(`$home/.local/share`)  |
| local-data | local_data   | `$xdg_data_home`                         |
| local-cfg  | local_config | `$xdg_config_home`                       |
| desktop    |              | `$xdg_desktop_dir`:(`$home/Desktop`)     |
| doc        | document     | `$xdg_documents_dir`:(`$home/Documents`) |
| dl         | download     | `$xdg_download_dir`:(`$home/Downloads`)  |
| bin        | exe          | `$xdg_bin_home`:(`$home/.local/bin`)     |
| first-path | first_path   |                                          |
| last-path  | last_path    |                                          |
| font       | typeface     | `$xdg_data_home/fonts`                   |
| pic        | picture      | `$xdg_pictures_dir`:(`$home/Pictures`)   |
| pref       | preference   | `$xdg_config_home`                       |
| pub        | public       | `$xdg_publicshare_dir`:(`$home/Public`)  |
| runtime    |              | `$xdg_runtime_dir`:(`/run/user/[uid]/`)  |
| state      |              | `$xdg_state_home`:(`$home/.local/state`) |
| video      |              | `$xdg_video_dir`:(`$home/Videos`)        |
| music      | audio        | `$xdg_music_dir`:(`$home/Music`)         |
| template   |              | `$xdg_templates_dir`:(`$home/Templates`) |
| tmp        |              | `$tmpdir`:(`/tmp`)                       |
| tmp-rand   | tmp_random   | `$tmpdir/[pkg-name]_$random`             |
| temp       | temporary    | `env::temp_dir()`                        |
| var_tmp    | var-tmp      | `/var/tmp/[pkg-name]`                    |
| cli-data   | cli_data     | `$xdg_data_home`                         |
| cli-cfg    | cli_config   | `$xdg_config_home`                       |
| cli-cache  | cli_cache    | `$xdg_cache_home`                        |
| empty      |              | ""                                       |

`first_path` refers to the first `$PATH` variable, while `last_path` refers to the last one. If PATH is `/usr/local/bin:/usr/bin`, then `/usr/local/bin` is the first_path, and `/usr/bin` is the last_path.

Regarding `tmp` and `temp`:

- `tmp`: First, get the value of `$env:tmpdir`. If it exists, use that value. If not, use `env::temp_dir()` to obtain the directory path and check if it is read-only. If it is, use `["$dir:cache", "tmp"]`.
  - On some platforms, the tmp directory may be read-only for regular users, such as `/data/local/tmp`.
- `temp`: Use `env::temp_dir()` to obtain the directory path, without performing any checks.
- `tmp-rand`: Generate a random temporary directory, `rand` feature needs to be enabled.

#### Android

- var:

  - sd = "/storage/self/primary"

For items not listed, use Linux data.

| name       | alias        | Android `$dir`                        |
| ---------- | ------------ | ------------------------------------- |
| home       |              |                                       |
| cache      |              |                                       |
| cfg        | config       |                                       |
| data       |              |                                       |
| local-data | local_data   | `$sd/Android/data`                    |
| local-cfg  | local_config | `$sd/Android/data`                    |
| desktop    |              |                                       |
| doc        | document     | `$sd/Documents`                       |
| dl         | download     | `$sd/Download`                        |
| bin        | exe          |                                       |
| first-path | first_path   |                                       |
| last-path  | last_path    |                                       |
| font       | typeface     |                                       |
| pic        | picture      | `$sd/Pictures`                        |
| pref       | preference   |                                       |
| pub        | public       |                                       |
| runtime    |              |                                       |
| state      |              |                                       |
| video      |              | `$sd/Movies`                          |
| music      | audio        | `$sd/Music`                           |
| template   |              |                                       |
| tmp        |              | `$tmpdir`                             |
| tmp-rand   | tmp_random   | `$tmpdir/[pkg-name]_$random`          |
| temp       | temporary    | `env::temp_dir()`:(`/data/local/tmp`) |
| var_tmp    | var-tmp      |                                       |
| cli-data   | cli_data     | `$xdg_data_home`                      |
| cli-cfg    | cli_config   | `$xdg_config_home`                    |
| cli-cache  | cli_cache    | `$xdg_cache_home`                     |
| sd         |              | /storage/self/primary                 |
| empty      |              | ""                                    |

#### Windows

- var:
  - ms_dir = `$home\AppData\Roaming\Microsoft`

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
| music                    | audio                    | `$home\music`                                                       |
| template                 |                          | `$ms_dir\Windows\Templates`                                         |
| tmp                      |                          | `$tmpdir`                                                           |
| tmp-rand                 | tmp_random               | `$tmpdir\[pkg-name]_$random`                                        |
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

#### macOS

| name       | alias        | macOS `$dir`                        |
| ---------- | ------------ | ----------------------------------- |
| home       |              | /Users/m                            |
| cache      |              | `$home/Library/Caches`              |
| cfg        | config       | `$home/Library/Application Support` |
| data       |              | `$home/Library/Application Support` |
| local-data | local_data   | `$home/Library/Application Support` |
| local-cfg  | local_config | `$home/Library/Application Support` |
| desktop    |              | `$home/Desktop`                     |
| doc        | document     | `$hom/Documents`                    |
| dl         | download     | `$home/Downloads`                   |
| bin        | exe          |                                     |
| first-path | first_path   |                                     |
| last-path  | last_path    |                                     |
| font       | typeface     | `$home/Library/Fonts`               |
| pic        | picture      | `$home/Pictures`                    |
| pref       | preference   | `$home/Library/Preferences`         |
| pub        | public       | `$home/Public`                      |
| runtime    |              | None                                |
| state      |              | None                                |
| video      |              | `$home/Movies`                      |
| music      | audio        | `$home/music`                       |
| template   |              | None                                |
| tmp        |              | `$tmpdir`                           |
| tmp-rand   | tmp_random   | `$tmpdir/[pkg-name]_$random`        |
| temp       | temporary    | `env::temp_dir()`                   |
| var_tmp    | var-tmp      | `/var/tmp/[pkg-name]`               |
| cli-data   | cli_data     | `$home/Library/Application Support` |
| cli-cfg    | cli_config   | `$home/Library/Application Support` |
| cli-cache  | cli_cache    | `$home/Library/Caches`              |
| empty      |              | ""                                  |

### project

Most of the data is obtained from [directories](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html).

Use `$proj(qualifier.organization.application):name` (e.g. `$proj(org.moz.ff):data`) or `$proj(com.company-name.app-name):alias` to obtain the project directory.

These directories will vary depending on the operating system and the specific configuration.

Assuming the project is `(org.moz.ff)`, here's an example:

#### Linux

| name       | alias        | Linux `$proj`                                          |
| ---------- | ------------ | ------------------------------------------------------ |
| path       |              | (the project path fragment): ff                        |
| cache      |              | `$xdg_cache_home/$proj_path`:(`$home/.cache/ff`)       |
| cfg        | config       | `$xdg_config_home/$proj_path`:(`$home/.config/ff`)     |
| data       |              | `$xdg_data_home/$proj_path`:(`$home/.local/share/ff`)  |
| local-data | local_data   | `$xdg_data_home/$proj_path`                            |
| local-cfg  | local_config | `$xdg_config_home/$proj_path`                          |
| pref       | preference   | `$xdg_config_home/$proj_path`                          |
| runtime    |              | `$xdg_runtime_dir/$proj_path`:(`/run/user/[uid]/ff`)   |
| state      |              | `$xdg_state_home/$proj_path`:(`$home/.local/state/ff`) |
| cli-data   | cli_data     | `$xdg_data_home/$proj_path`                            |
| cli-cfg    | cli_config   | `$xdg_config_home/$proj_path`                          |
| cli-cache  | cli_cache    | `$xdg_cache_home/$proj_path`                           |
| empty      |              | ""                                                     |

#### Android

- var:

  - sd = "/storage/self/primary"

| name       | alias        | Android `$proj`                     |
| ---------- | ------------ | ----------------------------------- |
| path       |              | org.moz.ff                          |
| cache      |              | /data/data/org.moz.ff/cache         |
| cfg        | config       | /data/data/org.moz.ff/files         |
| data       |              | /data/data/org.moz.ff               |
| local-data | local_data   | `$sd/Android/data/org.moz.ff`       |
| local-cfg  | local_config | `$sd/Android/data/org.moz.ff/files` |
| pref       | preference   | /data/data/org.moz.ff/files         |
| runtime    |              | `$xdg_runtime_dir/ff`               |
| state      |              | `$xdg_state_home/ff`                |
| cli-data   | cli_data     | `$xdg_data_home/ff`                 |
| cli-cfg    | cli_config   | `$xdg_config_home/ff`               |
| cli-cache  | cli_cache    | `$xdg_cache_home/ff`                |
| empty      |              | ""                                  |

#### Windows

| name       | alias        | Windows `$proj`                       |
| ---------- | ------------ | ------------------------------------- |
| path       |              | `moz\ff`                              |
| cache      |              | `$home\AppData\Local\moz\ff\cache`    |
| cfg        | config       | `$home\AppData\Roaming\moz\ff\config` |
| data       |              | `$home\AppData\Roaming\moz\ff\data`   |
| local-data | local_data   | `$home\AppData\Local\moz\ff\data`     |
| local-cfg  | local_config | `$home\AppData\Local\moz\ff\config`   |
| pref       | preference   | `$home\AppData\Roaming\moz\ff\config` |
| cli-data   | cli_data     | `$home\AppData\Local\moz\ff\data`     |
| cli-cfg    | cli_config   | `$home\AppData\Local\moz\ff\config`   |
| cli-cache  | cli_cache    | `$home\AppData\Local\moz\ff\cache`    |
| local-low  | local_low    | `$home\AppData\LocalLow\moz\ff`       |
| empty      |              | ""                                    |

#### macOS

| name       | alias        | macOS `$proj`                                  |
| ---------- | ------------ | ---------------------------------------------- |
| path       |              | org.moz.ff                                     |
| cache      |              | `$home/Library/Caches/org.moz.ff`              |
| cfg        | config       | `$home/Library/Application Support/org.moz.ff` |
| data       |              | `$home/Library/Application Support/org.moz.ff` |
| local-data | local_data   | `$home/Library/Application Support/org.moz.ff` |
| local-cfg  | local_config | `$home/Library/Application Support/org.moz.ff` |
| pref       | preference   | `$home/Library/Preferences/org.moz.ff`         |
| cli-data   | cli_data     | `$home/Library/Application Support/org.moz.ff` |
| cli-cfg    | cli_config   | `$home/Library/Application Support/org.moz.ff` |
| cli-cache  | cli_cache    | `$home/Library/Caches/org.moz.ff`              |
| empty      |              | ""                                             |

#### "??" in project

The `?` syntax supported by `$proj` is slightly more complex than other types, because it has `()` while others don't.

Don't worry, if you have already mastered the core syntax, then you can quickly master the `??` syntax of `$proj` in a few minutes.

Assuming there are three projects:

- (org. moz. ff)
- (com. gg. cr)
- (com. ms. eg)

---

The first example is:

```rs
["
    $proj (org . moz . ff ): runtime ? data ?? state ?
    (com . gg . cr): cfg ?? cache ?
    (com . ms . eg): local-data ? data
"]
```

Let's start parsing the runtime of the ff project, unfortunately, it does not exist.

Next, we parse the data! Great, we find that its value exists.

Because there are double `?`, we also need to check if the file path exists.

Unfortunately, the data directory does not exist.

The next one is state.

Unfortunately, it did not pass either because its value does not exist.

By now, all members of the ff camp have been defeated, and none have been successfully parsed.

So, we continue to parse the cr project, luckily, it succeeded on the first try.

The value of cr's cfg not only exists, but the path also exists.

The final return value is the cfg directory of the cr project!

---

The second example is:

```rs
["
    $proj (org . moz . ff )：runtime ？ data ？？ state ？
    (com . gg . cr)： cfg ？？ cache ？
    (com . ms . eg)： local-data ？ data
"]
```

Q: Why don't I see any difference from the first example?

A: It allows you to use full-width symbols (colon and question mark) as separators, but this is limited.

It depends on the first symbol that appears. That is to say, if the first separator is a half-width "?"(`\u{3F}`) instead of a full-width "？"(`\u{FF1F}`), then the rest should also be expressed in half-width.
