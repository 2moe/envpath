# EnvPath

一个用于 **解析** 和 **反序列化** 具有特殊规则的路径的 library。

格式类似于 `["$proj(com.xy.z): data ? cfg", "$const: pkg", "$const: deb-arch"]`

## preface

注意: 本文包含大量非技术内容。

在正式开始前，很抱歉打扰到您，能否请您解答我心中的一个小小的困惑呢？

我们一直以来是如何解决跨平台路径配置的问题？

假设有如下配置：

```toml
[dir]
data = "C:\\Users\\[username]\\AppData\\Roaming\\[dirname]"
```

也许我们会为配置文件新建一个 Map (e.g. `HashMap<String, Dirs>`), 让不同平台使用不同配置。

```toml
[dir.linux]
data = "/home/[username]/.local/share/[appname]"
cache = "/home/[username]/.cache/[app]"

[dir.macos]
data = "/Users/[username]/Library/Application Support/x.y.z"

[dir.xxxbsd]

[dir.your-os-name]
```

这是一个好方法，但是有没有更通用的方法呢？  
于是，人们想到了使用环境变量。

我猜您想到了 XDG 规范，既然它这么有用的话，那我们在所有平台上都使用 `$XDG_DATA_HOME/[appname]/` 如何？
然而，不幸的是，并不是所有平台都支持，所以我们选择更通用的 `$HOME`。

不幸的事情再次到来，在早期的 Windows 上，可能并没有 `%HOME%`, 而是只有 `%userprofile%`。  
So, what should we do? And how do we do it?

我们可以使用一些外部 crates, 自动获取不同平台的路径。  
或者是手动为不同平台编写不同的目录映射关系。

Nice！我们好像已经解决了跨平台问题，但可能忘记了一件事。  
那就是不同平台的路径分隔符可能是不同的。  
我们可以为不同平台生成路径，但是生成的格式可能不太通用。

- Windows 用 `C:\path\to\xxx`
- Unix-like 用 `/path/to/xxx`

### Path Separator

以下是一些补充内容：

> Macintosh 操作系统中的路径分隔符经历了多次发展和变化。早期版本的 Macintosh 操作系统使用斜杠字符 "/" 作为路径分隔符。  
> 1985 年，随着 Hierarchical File System（HFS）的引入，路径分隔符被切换为冒号 ":"。这种改变是由于 HFS 的设计需要在文件名中包含冒号。
>
> 2001 年，苹果公司推出了 Mac OS X 操作系统(现代 macOS 的前身)，并引入了一种新的文件系统 HFS+。该文件系统尽管仍在使用冒号 ":" 作为路径分隔符，但支持区分大小写，并添加了对 Unicode 字符集的支持。  
> 根据苹果技术笔记 TN1150: HFS Plus Volume Format 所述，将冒号用作 HFS 文件系统中的路径分隔符，旨在允许用户轻松浏览目录, 从而让操作系统变得更加 friendly。  
> 2017 年，苹果公司推出了新的文件系统 APFS，它成为最新的 macOS 默认文件系统。APFS 具有许多优点，例如更快的速度、更高的安全性和更好的数据备份等。  
> APFS 中使用斜杠字符 "/" 作为路径分隔符。  
> 这是因为斜杠在 “类 UNIX 系统” 中已被广泛采用，苹果公司希望 macOS 能够提高与其他平台的兼容性。
>
> Source: [Apple Technical Note TN1150](https://developer.apple.com/library/archive/technotes/tn/tn1150.html)

~~macOS 操作系统中的路径分隔符发展历史反映了苹果公司对操作系统设计的持续优化。这种演变是为了改进用户体验和完善磁盘文件系统功能，以满足不断变化的使用需求和技术要求。
苹果公司一直以来都敢于抛弃旧事物，不断推陈出新，这是其取得成功的重要原因之一。在传统技术逐渐失去优势的今天，我们应该勇于接受挑战和改变，不断追求卓越和创新，为推动时代发展和人类福祉作出积极贡献。~~ (不好意思，拿错台词了)

下面是一份有关分隔符的表格。  
有些系统可能会更改其文件路径分隔符，如果有误，还望各位积极反馈 issue。

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

---

既然不同平台使用不同路径分隔符，那我们要怎么样让它看起来一样呢？  
答案是使用数组 (或者是 Vector)。

但是它的缺点特别明显，就是对于普通用户来说，这种格式看起来可能会比字符串更难看。 （尽管我们可能更喜欢前者）  
请注意，用户配置文件是用来给用户看的，而不是只是单纯地用来反序列化，所以可读性至关重要。

比如 `["C:", "\\", "Users", "Public"]` 相当于 `C:\Users\Public`  
这里有个容易忽视的小细节，就是第二个元素是 "\\"。

EnvPath(raw) 用的也是数组结构（其实是 Vector），但是使用了特殊的规则。

比如 `["$dir: dl ? doc"]` 指定为 Downloads 目录（不同的平台的路径不一样），如果 Downloads 目录的值不存在，就用 Documents 目录。
注意：一个 `?` 与两个 `?` 是有区别的, 之后我们会提到的。

说完了一大堆无关紧要的话后，让我们正式开始吧！

## Quick Start

在开始前，请先确保您的 rust 版本不要太旧了。  
因为这个 library 里面用到了一些比较新的语法，比如 let-else (需要 1.65+)。

### Basic guide

首先，我们需要添加依赖:

```sh
cargo add envpath
```

或者是根据需求添加 features, 而不是使用默认的功能。

```sh
cargo add envpath --no-default-features --features=base-dirs,const-dirs,project-dirs
```

然后在我们的 `main()` 或者是测试函数里添加以下内容。

```rust
use envpath::EnvPath;

let v = EnvPath::from(["$dir: data", "$const: pkg", "$env: test_qwq", "app"]).de();
dbg!(v.display(), v.exists());
```

这是一个简单的例子，还有更多的功能和概念，我没有在这里提到。  
不要着急，一步一步慢慢来。

然后它会输出类似于以下的内容

```js
[src/lib.rs:74] v.display() = "/home/m/.local/share/envpath/$env: test_qwq/app"
[src/lib.rs:74] v.exists() = false
```

我们可以看到 `$env: test_qwq` 并没有被成功解析。

所以到底发生了什么？是它出故障了吗？

不，并不是，这是有意为之的设计。 EnvPath 在设计之初，就有意兼容普通的路径。

如果有一天， envpath 新增了一个功能，需要以 `$recycle` 为前缀，加上 `bin` 关键词就能解析到特定目录。  
而您的系统磁盘上，刚好有个 `$RECYCLE:BIN` 文件夹，不巧的是，那个磁盘的文件系统刚好没有开启区分大小写（Case-sensitive）的功能。  
当存在同名路径时，默认会先解析，解析失败后，会假设当前存在同名路径，然后直接返回。  
同名路径碰撞（解析的式子与文件路径同名）的概率是存在的，不过，只要稍微花一点技巧就能避开绝大多数的碰撞事件。

> 技巧：多用空白字符 (空格，换行符，制表符之类的)，以及使用 `?`(下文会介绍)  
> 比如 `$env: test_qwq` 可以写成 `$env    ：          test-QwQ`  
> 尽管加了那么多空格， 但如果成功的话，它们会被解析为同一个值。将上面的表达式用 unix 的 posix sh 来描述是： `$TEST_QWQ` (i.e. 所有小写字母全部变为大写，所有 `-` 全部变为 `_`)  
> 尽管您觉得这种做法可能很难接受，但对于全局系统环境变量来说，这样做是惯例，我并没有创建新的规则。

既然都解析失败了，那为什么不返回空目录呢?

用 posix sh 的 env 举个例子吧！

假设您要访问的目录是 `$XDG_DATA_HOME/app`, 如果相关的 env 是空的话，那么您访问的就是 /app ，这与预期结果不同。（~~我想要回家，但是却买错了车票 🎫~~  
您可能会辩解道： 我可以用 `${ENV_NAME:-FALLBACK}` 来指定 fallback 啊！ 明明是你太笨了。

然而，有时候一不小心的疏忽可能会酿成大错。我觉得少点抱怨，会让生活变得更美好。

说到这里，您可能已经忘记了前面出错的地方： `$env: test_qwq`。  
那么要如何解决呢？您可以试试把它修改为 `$env: test_qwq ? user ? logname`， 或者是添加更多的问号与有效的环境变量名称。

~~这里就先不解释`?` 的作用了，自己去探索，往往能发现更多的乐趣。（我写完这句话后，才想起前面已经解释过了 QuQ~~

---

回到我们刚开始提到的代码，然后稍微简化一下。  
`EnvPath::from(["$dir: data"]).de();`

As is well known, `[]` 是一个数组。但 `.de()` 究竟是什么？
在中文里，如果要用 `de` 来指代一个国家的话，那它是德国。如果用来形容人的话，可以说他有 “高尚品德”。  
Ohhhh！I got it. 这个函数去了一趟德国（de），所以发生了改变，变成了有品德的函数。

总之，我觉得您很聪明，这个函数的确发生了变化。  
不过它只是将类似于 `$env: QuQ ?? qwq-dir ? AwA-home` 的结构转换成另一个值。

### serialisation & deserialisation

如果您想要序列化/反序列化配置文件，需要启用 envpath 的 `serde` 功能，并且还要添加 serde 依赖，以及与之有关的其他依赖。

下面我们将添加一个 `ron` 依赖（实际上您还可以用 toml, yaml 或 json 等格式，不过相关依赖就不是 ron 了）

```sh
cargo add envpath --features=serde
cargo add serde --features=derive
cargo add ron
```

接着让我们一起写代码吧！

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

我们首先定义了一个 Cfg 结构体，然后创建了一个新的 EnvPath instance, 接着把 dir 包装进 Cfg 里，用 ron 进行序列化，最后写入到 `test.ron`。

输出的结果是 : `(dir:Some(["$env: user ?? userprofile ?? home"]))`

除了多了个 `dir` 作为 key， 看起来它的结构与没有序列化之前一样啊！  
Yes, you are right. 序列化后，看起来就是这样。

这种格式的路径适合跨平台使用。  
由于环境变量以及其他东西可能是动态改变的，因此序列化时保留 raw 格式，在反序列化时获得它的真实路径，这种做法是合理的。  
您如果想要节省性能开销，那可以把 dir 的值改成 None, serde 在序列化时应该会跳过它。

接下来，让我们试试反序列化吧！

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

对于支持 toml, yaml, json 反序列化的 crates， 如果没有 `from_reader()`, 那么可以用 `from_str()`，这里就不多介绍了。

上面的函数输出的结果为

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

`?` 会判断值是否存在，如果不存在，那就继续判断。如果存在，那就使用这个值。

而 `??` 指的是值和路径都要存在。

比如说 `$env: user ? userprofile`, 这里假设 user 的值为 m, userprofile 的值为空。
因为 user 的值存在，所以这条表达式的返回值为 m。

如果把它改成 `$env: user ?? userprofile ? home` 的话，
尽管 user 的值存在，但它的路径不存在，所以继续判断。
然后，userprofile 的值不存在，所以继续判断，直到满足条件为止。

`?` 和 `??` 有着不同的作用，并不是说有了 `??` 后就可以抛弃 `?`。
对于 `$const: os` 这种普通字符串，而不是路径的值来说，`?` 会比 `??` 更有用。
每个人都在扮演着重要的角色，各司其职。

Basic guide 到这里就快要结束了。
上面所述的都是一些基本功能。

project_dirs 里有更高级的功能，以下是一些简单的介绍。
比如说，`$proj(com.macro-hard.app-name): data` 会为这个项目生成 data 目录（不会自动创建，只是生成它的值）。

> `M$` : ~~律师函警告~~
>
> Sorry，Please forgive me.

All right，它现在是 `(com. x. y)`

- 在 android 上，它是 `/data/data/com.x.y`
- 在 macOS 上，它是 `/Users/[username]/Library/Application Support/com.x.y`

在了解完基本的用法后，我们将继续介绍和补充更多内容。

- 最简单的 ： const-dirs
- 常用的基本标准目录： base-dirs
- 高级的项目目录： project-dirs

在下文中，我们会介绍到它们的用法，以及它们都有哪些值。

## Features

### env

在上文中，我们已经了解到了基本用法。  
这里还是再啰嗦几句。  
env 指的是环境变量，`$env: home` 指的是获取 HOME 环境变量的值。  
`$env:   xdg-data-home` 相当于 `$XDG_DATA_HOME`。  
至于 '?' 的用法，您可以翻看前文，等到您了解 `$env: userprofile ??  QwQ-Dir ? LocalAppData ? home` 的作用的时候。  
恭喜，您已经学会了 env 的用法了！

### const

使用 `$const: name` (例如 `$const: arch`) 或者是 `$const: alias` (e.g. `$const: architecture`) 来获取常量值。
这些值是在编译时获取的，而不是运行时。

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

下面的表格是 `$const: deb-arch` 可能会输出的值。

比如说，您编译了一个 `armv7` 的软件包， 用 `$const:  arch` 得到的值是 arm, 而 `$const:  deb-arch` 可能是 armhf。

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

### remix

| name                      | alias       | example                          |
| ------------------------- | ----------- | -------------------------------- |
| `env * [env_name]`        | `env*[env]` | `env * HOME`                     |
| `const * [const]`         |             | `const * arch`                   |
| `dir * [dir]`             |             | `dir * download`                 |
| `proj * (project): ident` |             | `proj * (com. xy.z): local-data` |

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

`env*` 可用于 fallback, 但与 `$env:` 不同，它不会自动将小写字母全部转换为大写，也不会将 `-` 转换为 `_`。

- `env * home` 获取的是 `$home` , 而不是 `$HOME`。
- `$env: home` => `$HOME`
- `env * xdg-data-home` => `$xdg-data-home`, not `$XDG_DATA_HOME`
- `$env: xdg-data-home` => `$XDG_DATA_HOME`

> 注： 如果 `$env:` 式子中包含 `*`， 那么自动转换功能也会被禁用。

目前支持的语法：

- `$const: exe_suffix ?   env * HOME ?   env * XDG_DATA_HOME ?   env * EXE_SUFFIX`
- `$env: home ? xdg-data-home ? exe_suffix ?    const * exe_suffix`

不支持:

- `$const: exe_suffix ? $env: home ? xdg-data-home ? exe_suffix`

如果要支持这种语法的话, 那么解析会变得麻烦，并且 `$env: exe_suffix` 与 `$const: exe_suffix` 很容易搞混。

### base

这些是一些基本目录，也可以说是标准目录。  
使用 `$dir: name` (e.g. `$dir: dl`) 或者是 `$dir: alias` (e.g. `$dir: download`) 来获取 dir。
有不少内容都是通过 [dirs](https://docs.rs/dirs/latest/dirs/) 来获取的，不过也有一些补充。

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
| tmp        |              |                                          |
| temp       | temporary    |                                          |
| cli-data   | cli_data     | `$xdg_data_home`                         |
| cli-cfg    | cli_config   | `$xdg_config_home`                       |
| cli-cache  | cli_cache    | `$xdg_cache_home`                        |
| empty      |              | ""                                       |

first_path 指的是第一个 `$PATH` 变量， last_path 则是最后一个。
若有 PATH 为 `/usr/local/bin:/usr/bin`，
则 `/usr/local/bin` 为 first_path, `/usr/bin` 为 last_path。

关于 tmp 与 temp

- tmp: 先获取 `$env: tmpdir` 的值，若存在, 则使用该值。若不存在，使用 `env::temp_dir()` 获取，判断文件路径是否只读，若是，则使用 `["$dir: cache", "tmp"]`
  - 有些平台的 tmp 目录对于普通用户可能是只读的，没错，说的就是你： `/data/local/tmp`
- temp: 使用 `env::temp_dir()` 获取, 不进行判断

#### Android

- var:

  - sd = "/storage/self/primary"

对于没有列出的内容，使用 linux 的数据

| name       | alias        | Android `$dir`        |
| ---------- | ------------ | --------------------- |
| home       |              |                       |
| cache      |              |                       |
| cfg        | config       |                       |
| data       |              |                       |
| local-data | local_data   | `$sd/Android/data`    |
| local-cfg  | local_config |                       |
| desktop    |              |                       |
| doc        | document     | `$sd/Documents`       |
| dl         | download     | `$sd/Download`        |
| bin        | exe          |                       |
| first-path | first_path   |                       |
| last-path  | last_path    |                       |
| font       | typeface     |                       |
| pic        | picture      | `$sd/Pictures`        |
| pref       | preference   |                       |
| pub        | public       |                       |
| runtime    |              |                       |
| state      |              |                       |
| video      |              | `$sd/Movies`          |
| music      | audio        | `$sd/Music`           |
| template   |              |                       |
| tmp        |              |                       |
| temp       | temporary    |                       |
| cli-data   | cli_data     | `$xdg_data_home`      |
| cli-cfg    | cli_config   | `$xdg_config_home`    |
| cli-cache  | cli_cache    | `$xdg_cache_home`     |
| sd         |              | /storage/self/primary |
| empty      |              | ""                    |

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
| tmp                      |                          |                                                                     |
| temp                     | temporary                |                                                                     |
| cli-data                 | cli_data                 | `$home\AppData\Local`                                               |
| cli-cfg                  | cli_config               | `$home\AppData\Local`                                               |
| cli-cache                | cli_cache                | `$home\AppData\Local`                                               |
| progam-files             | program_files            | `$ProgramFiles`: (`C:\Program Files`)                               |
| program-files-x86        | program_files_x86        | `$ProgramFiles(x86)`: (`C:\Program Files (x86)`)                    |
| common-program-files     | common_program_files     | `$CommonProgramFiles`: (`C:\Program Files\Common Files`)            |
| common-program-files-x86 | common_program_files_x86 | `$CommonProgramFiles(x86)`: (`C:\Program Files (x86)\Common Files`) |
| program-data             | program_data             | `$ProgramData`: (`C:\ProgramData`)                                  |
| microsoft                |                          | `$home\AppData\Local\Roaming\Microsoft`                             |
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
| tmp        |              |                                     |
| temp       | temporary    |                                     |
| cli-data   | cli_data     | `$home/Library/Application Support` |
| cli-cfg    | cli_config   | `$home/Library/Application Support` |
| cli-cache  | cli_cache    | `$home/Library/Caches`              |
| empty      |              | ""                                  |

### project

为项目生成指定目录。  
大部分数据从 [directories](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html) 获取。

使用 `$proj(qualifier.  organization.   application): name` (e.g. `$proj(org. moz. ff): data`) 或者是 `$proj(com.company-name.app-name): alias` 来获取 project dir。

接下来假设项目为 `(org. moz. ff)`

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

#### project 中的 "??"

`$proj` 支持的 `?` 语法比其他的类型要更复杂一点，因为其他类型没有 `()`,而它有。  
别灰心，如果您已经掌握了核心语法，那么相信您定能在几分钟快速掌握 `$proj` 的 `??`语法。

假设有三个项目：

- (org. moz. ff)
- (com. gg. cr)
- (com . ms . eg)

第一个例子为：

```rs
["
    $proj (org . moz . ff ): runtime ? data ?? state ?
    (com . gg . cr): cfg ?? cache ?
    (com . ms . eg): local-data ? data
"]
```

我们开始解析 ff 项目的 runtime, 很不幸，它不存在。  
我们接着解析 data！太好了，我们发现它的值是存在的。  
~~然而，好景不长，好不容易找到了一个存在的值，这时候天道的考验来了，只有通过才能羽化飞升。怎么回事？我现在的修为连元婴期都没有，而且灵力和神识还在不断溃散~~
（不好意思，拿错剧本了...  
由于有两个 '?' ，所以还需要判断文件路径是否存在。  
~~（所以说 data 君 的身死道消也不是没有理由的吗~~  
下一个幸运儿是 state。  
~~远处，一声充满不甘，又直透人心的怒吼声传来：“我命由我不由天，我只是错...”
突然间，声音戛然而止，未几，远处传来了婴儿的啼哭声，让一切都充满着阴森诡异的味道。~~  
很遗憾，它也没能通过，因为它的值不存在。  
至此，ff 阵营的成员全军覆没，没有一个解析成功。  
于是，我们接着解析 cr 项目，幸运的是，第一次就成功了。  
cr 的 cfg 不仅值存在，路径也是存在的。  
最终的胜者是 cr 家的 cfg，尽管如此，但她却不怎么开心的样子。在离开前，她嘴角边还嘟喃道：“明明只是个解析器，竟然敢这么嚣张，哼！”

第二个例子为：

```rs
["
    $proj (org . moz . ff )：runtime ？ data ？？ state ？
    (com . gg . cr)： cfg ？？ cache ？
    (com . ms . eg)： local-data ？ data
"]
```

Q: 咦？我怎么没看出来，这与第一个例子有何不同？  
A: 届时，汝自会知晓。  
（此时一道亮光划破天际，而眼前之人早已不见踪影）  
Q: 真是奇怪，我的脑袋里怎么会多出些奇奇怪怪的记忆？我怕不是睡迷糊了。对了，我刚刚在和谁说话来着？
