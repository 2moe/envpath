# EnvPath

[![envpath.crate](https://img.shields.io/crates/v/envpath.svg?logo=rust&logoColor=lightsalmon&label=envpath)](https://crates.io/crates/envpath)

[![Documentation](https://docs.rs/envpath/badge.svg)](https://docs.rs/envpath)
[![Apache-2 licensed](https://img.shields.io/crates/l/envpath.svg?logo=apache)](../License)

一個用於 **解析** 和 **反序列化** 具有特殊規則的路徑的 library。

格式類似於 `["$proj(com.xy.z): data ? cfg", "$const: os", "$val: rand-16"]`

<!-- Language -->
<details open>
<summary>
<img alt="Language/語言" src="./svg/language.svg" />
</summary>

- [zh-Hant: 繁體中文](Readme-zh-Hant.md)
- [en: English](Readme.md)
- [zh: 簡體中文](Readme-zh.md)

</details>

<!-- TOC -->
<details open>
<summary>
<img alt="目錄" src="./svg/toc/目錄.svg"/>
</summary>

- [preface](#preface)
  - [結構](#結構)
- [Quick Start](#quick-start)
  - [Basic guide](#basic-guide)
  - [serialisation \& deserialisation](#serialisation--deserialisation)
    - [serialisation](#serialisation)
    - [deserialisation](#deserialisation)
- [Features](#features)
  - [env](#env)
  - [const](#const)
    - [deb-arch](#deb-arch)
  - [val](#val)
  - [remix](#remix)
    - [example](#example)
  - [base](#base)
    - [Linux](#linux)
    - [Android](#android)
    - [Windows](#windows)
    - [macOS](#macos)
  - [project](#project)
    - [Linux](#linux-1)
    - [Android](#android-1)
    - [Windows](#windows-1)
    - [macOS](#macos-1)
    - [project 中的 "??"](#project-中的-)

</details>

## preface

在正式開始前，很抱歉打擾到您，能否請您解答我心中的一個小小的困惑呢？

我們一直以來是如何解決跨平臺路徑配置的問題？

假設有如下配置：

```toml
[dir]
data = "C:\\Users\\[username]\\AppData\\Roaming\\[dirname]"
```

也許我們會為配置檔案新建一個 Map (e.g. `HashMap<String, Dirs>`), 讓不同平臺使用不同配置。

```toml
[dir.linux]
data = "/home/[username]/.local/share/[appname]"
cache = "/home/[username]/.cache/[app]"

[dir.macos]
data = "/Users/[username]/Library/Application Support/x.y.z"

[dir.xxxbsd]

[dir.your-os-name]
```

這是一個好方法，但是有沒有更通用的方法呢？
於是，人們想到了使用環境變數。

我猜您想到了 XDG 規範，既然它這麼有用的話，那我們在所有平臺上都使用 `$XDG_DATA_HOME/[appname]/` 如何？
然而，不幸的是，並不是所有平臺都支援 XDG 規範，所以我們選擇更通用的 `$HOME`。

不幸的事情再次到來，在早期的 Windows 上，可能並沒有 `%HOME%`, 而是隻有 `%userprofile%`。

envpath 的設計初衷就是為了解決跨平臺目錄配置的問題。

```toml
[dir]
data = [
    "$proj(com.org-name.app-name): local-data",
    "dir"
]
# - Windows: C:\Users\[username]\AppData\Local\org-name\app-name\dir
# - Linux: /home/[username]/.local/share/app-name/dir
# - macOS: /Users/[username]/Library/Application Support/com.org-name.app-name/dir
```

### 結構

Raw 格式的 EnvPath 本質上是使用了特殊的規則的陣列結構。

它的缺點特別明顯，對於普通使用者來說，這種格式看起來可能會比字串更難看。

請注意，使用者配置檔案是用來給使用者看的，而不是隻是單純地用來反序列化，所以可讀性至關重要。

當不使用特殊規則時，它可以相容普通的路徑陣列。

比如 `["C:", "\\", "Users", "Public"]` 相當於 `C:\Users\Public`

> 這裡有個容易忽視的小細節，就是第二個元素是 "\\"。

Q: 特殊規則？

A: 當陣列的元素以 `$[關鍵詞]`開頭，並且整個元素的表示式都能正常解析時，它就是一條特殊規則。一個數組可以應用多條特殊規則。

例子：

- 環境變數（關鍵詞 env）： `["$env: XDG_CONFIG_HOME", "xx"]`，在某些系統上，它會被解析為 `/home/[user]/.config/xx`
- `?` 代表 fallback：
  - `["$dir: dl ? doc"]` 指定為 Downloads 目錄（不同的平臺的路徑不一樣）
  - 如果 Downloads 目錄的值不存在，就用 Documents 目錄。

注意：一個 `?` 與兩個 `?` 是有區別的, 之後我們會提到的。

## Quick Start

### Basic guide

首先，我們需要新增依賴:

```sh
cargo add envpath --no-default-features --features=dirs,consts,project
```

然後在我們的 `main()` 或者是測試函數里新增以下內容。

```rust
use envpath::EnvPath;

let v = EnvPath::from(["$dir: data", "$env: test_qwq", "app"]).de();
dbg!(v.display(), v.exists());
```

這是一個簡單的例子，還有更多的功能和概念，我沒有在這裡提到。
不要著急，一步一步慢慢來。

然後它會輸出類似於以下的內容

```js
[src/lib.rs:74] v.display() = "/home/m/.local/share/$env: test_qwq/app"
[src/lib.rs:74] v.exists() = false
```

我們可以看到 `$env: test_qwq` 並沒有被成功解析。

所以到底發生了什麼？是它出故障了嗎？

不，並不是，這是有意為之的設計。 EnvPath 在設計之初，就有意相容普通的路徑。

如果有一天， envpath 新增了一個功能，需要以 `$recycle` 為字首，加上 `bin` 關鍵詞就能解析到特定目錄。
而您的系統磁碟上，剛好有個 `$RECYCLE:BIN` 資料夾，不巧的是，那個磁碟的檔案系統剛好沒有開啟區分大小寫（Case-sensitive）的功能。
當存在同名路徑時，預設會先解析，解析失敗後，會假設當前存在同名路徑，然後直接返回。
同名路徑碰撞（解析的式子與檔案路徑同名）的機率是存在的，不過，只要稍微花一點技巧就能避開絕大多數的碰撞事件。

> 技巧：多用空白字元 (空格，換行符，製表符之類的)，以及使用 `?`(下文會介紹)
> 比如 `$env: test_qwq` 可以寫成 `$env    ：          test-QwQ`
> 儘管加了那麼多空格， 但如果成功的話，它們會被解析為同一個值。將上面的表示式用 unix 的 posix sh 來描述是： `$TEST_QWQ` (i.e. 所有小寫字母全部變為大寫，所有 `-` 全部變為 `_`)
> 儘管您覺得這種做法可能很難接受，但對於全域性系統環境變數來說，這樣做是慣例，我並沒有創造新的規則。

既然都解析失敗了，那為什麼不返回空目錄呢?

用 posix sh 的 env 舉個例子吧！

假設您要訪問的目錄是 `$XDG_DATA_HOME/app`, 如果相關的 env 是空的話，那麼您訪問的就是 /app ，這與預期結果不同。（~~我想要回家，但是卻買錯了車票 🎫~~
您可能會辯解道： 我可以用 `${ENV_NAME:-FALLBACK}` 來指定 fallback 啊！ 明明是你太笨了。

然而，有時候一不小心的疏忽可能會釀成大錯。我覺得少點抱怨，會讓生活變得更美好。

說到這裡，您可能已經忘記了前面出錯的地方： `$env: test_qwq`。
那麼要如何解決呢？您可以試試把它修改為 `$env: test_qwq ? user ? logname`， 或者是新增更多的問號與有效的環境變數名稱。

~~這裡就先不解釋`?` 的作用了，自己去探索，往往能發現更多的樂趣。（我寫完這句話後，才想起前面已經解釋過了 QuQ~~

---

回到我們剛開始提到的程式碼，然後稍微簡化一下。
`EnvPath::from(["$dir: data"]).de();`

As is well known, `[]` 是一個數組。但 `.de()` 究竟是什麼？
在中文裡，如果要用 `de` 來指代一個國家的話，那它是德國。如果用來形容人的話，可以說他有 “高尚品德”。
Ohhhh！I got it. 這個函式去了一趟德國（de），所以發生了改變，變成了有品德的函式。

總之，我覺得您很聰明，這個函式的確發生了變化。
不過它只是將類似於 `$env: QuQ ?? qwq-dir ? AwA-home` 的結構轉換成另一個值。

### serialisation & deserialisation

如果您想要序列化/反序列化配置檔案，需要啟用 envpath 的 `serde` 功能，並且還要新增 serde 依賴，以及與之有關的其他依賴。

下面我們將新增一個 `ron` 依賴（實際上您還可以用 yaml 或 json 等格式，不過相關依賴就不是 ron 了）

```sh
cargo add envpath --features=serde
cargo add serde --features=derive
cargo add ron
```

#### serialisation

接著讓我們一起寫程式碼吧！

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

我們首先定義了一個 Cfg 結構體，然後建立了一個新的 EnvPath instance, 接著把 dir 包裝進 Cfg 裡，用 ron 進行序列化，最後寫入到 `test.ron`。

輸出的結果是 : `(dir:Some(["$env: user ?? userprofile ?? home"]))`

除了多了個 `dir` 作為 key， 看起來它的結構與沒有序列化之前一樣啊！
Yes, you are right. 序列化後，看起來就是這樣。

這種格式的路徑適合跨平臺使用。
由於環境變數以及其他東西可能是動態改變的，因此序列化時保留 raw 格式，在反序列化時獲得它的真實路徑，這種做法是合理的。

#### deserialisation

接下來，讓我們試試反序列化吧！

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

上面的函式輸出的結果為

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

`?` 會判斷值是否存在，如果不存在，那就繼續判斷。如果存在，那就使用這個值。

而 `??` 指的是值和路徑都要存在。

比如說 `$env: user ? userprofile`, 這裡假設 user 的值為 m, userprofile 的值為空。
因為 user 的值存在，所以這條表示式的返回值為 m。

如果把它改成 `$env: user ?? userprofile ? home` 的話，
儘管 user 的值存在，但它的路徑不存在，所以繼續判斷。
然後，userprofile 的值不存在，所以繼續判斷，直到滿足條件為止。

`?` 和 `??` 有著不同的作用，並不是說有了 `??` 後就可以拋棄 `?`。
對於 `$const: os` 這種普通字串，而不是路徑的值來說，`?` 會比 `??` 更有用。
每個人都在扮演著重要的角色，各司其職。

Basic guide 到這裡就快要結束了。
上面所述的都是一些基本功能。

project_dirs 裡有更高階的功能，以下是一些簡單的介紹。
比如說，`$proj(com.macro-hard.app-name): data` 會為這個專案生成 data 目錄（不會自動建立，只是生成它的值）。

> com.macro-hard.app-name 這個名字有點不太妙啊！

All right，它現在是 `(com. x. y)`

- 在 android 上，它是 `/data/data/com.x.y`
- 在 macOS 上，它是 `/Users/[username]/Library/Application Support/com.x.y`

在瞭解完基本的用法後，我們將繼續介紹和補充更多內容。

- 最簡單的 ： consts
- 常用的基本標準目錄： dirs
- 高階的專案目錄： project

在下文中，我們會介紹到它們的用法，以及它們都有哪些值。

## Features

### env

在上文中，我們已經瞭解到了基本用法。
這裡還是再囉嗦幾句。
env 指的是環境變數，`$env: home` 指的是獲取 HOME 環境變數的值。
`$env:   xdg-data-home` 相當於 `$XDG_DATA_HOME`。
至於 '?' 的用法，您可以翻看前文，等到您瞭解 `$env: userprofile ??  QwQ-Dir ? LocalAppData ? home` 的作用的時候。
恭喜，您已經學會了 env 的用法了！

### const

使用 `$const: name` (e.g. `$const: arch`) 或者是 `$const: alias` (e.g. `$const: architecture`) 來獲取常量值。
這些值是在編譯時獲取的，而不是執行時。

| name          | alias        | From                    | example                 |
| ------------- | ------------ | ----------------------- | ----------------------- |
| arch          | architecture | `consts::ARCH`          | x86_64, aarch64         |
| deb-arch      | deb_arch     | `get_deb_arch()`        | amd64, arm64            |
| os            |              | `consts::OS`            | linux, windows, android |
| family        |              | `consts::FAMILY`        | unix, windows           |
| exe_suffix    |              | `consts::EXE_SUFFIX`    | `.exe`, `.nexe`         |
| exe_extension |              | `consts::EXE_EXTENSION` | exe                     |
| empty         |              |                         | ""                      |

#### deb-arch

下面的表格是 `$const: deb-arch` 可能會輸出的值。

比如說，您編譯了一個 `armv7` 的軟體包， 用 `$const:  arch` 得到的值是 arm, 而 `$const:  deb-arch` 可能是 armhf。

| Architecture                | deb_arch                                                                            |
| --------------------------- | ----------------------------------------------------------------------------------- |
| x86_64                      | amd64                                                                               |
| aarch64                     | arm64                                                                               |
| riscv64 (riscv64gc)         | riscv64                                                                             |
| arm (feature = `vfp3`)      | armhf                                                                               |
| arm                         | armel                                                                               |
| mips (endian = little)      | mipsel                                                                              |
| mips64 (endian = little)    | mips64el                                                                            |
| s390x                       | s390x                                                                               |
| powerpc64 (endian = little) | ppc64el                                                                             |
| x86 (i586/i686)             | i386                                                                                |
| other                       | [consts::ARCH](https://doc.rust-lang.org/nightly/std/env/consts/constant.ARCH.html) |

### val

使用 `$val:name` (e.g. `$val: rand-8`) 來獲取值。與 `$const:` 不同，大部分 `$val:` 的值都是在執行時獲取的，而不是編譯時。

| name           | expr            | example          |
| -------------- | --------------- | ---------------- |
| `rand-[usize]` | `$val: rand-16` | 90aU0QqYnx1gPEgN |
| empty          | `$val: empty`   | ""               |

rand 用於獲取 random(隨機) 內容，目前僅支援字串。

> rand 需要啟用 `rand` feature
>
> 碎碎念：咱感覺在寫這個功能的時候，有點走火入魔了，寫著寫著，甚至想要加上時間功能，類似於 `$val: time(rfc-3339, now)`
>
> 有時候，功能並非越多越好。
> EnvPath 的主要目標是簡單的跨平臺路徑，加太多功能有點違背初衷了。

### remix

| syntax                      | expr                            | example                                       |
| --------------------------- | ------------------------------- | --------------------------------------------- |
| `env * [env_name]`          | `env * HOME`                    | `C:\Users\[username]`                         |
| `const * [const]`           | `const * arch`                  | `x86_64`                                      |
| `dir * [dir]`               | `dir * dl`                      | `C:\Users\[username]\Downloads`               |
| `proj * (project): [ident]` | `proj * (com.xy.z): local-data` | `C:\Users\[username]\AppData\Local\xy\z\data` |
| `val * [val]`               | `val * rand-32`                 | o9kJjQqYc6lkznAPgaGnnY8dPYVzwawO              |

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

`env*` 可用於 fallback, 但與 `$env:` 不同，它不會自動將小寫字母全部轉換為大寫，也不會將 `-` 轉換為 `_`。

- `env * home` 獲取的是 `$home` , 而不是 `$HOME`。
- `$env: home` => `$HOME`
- `env * xdg-data-home` => `$xdg-data-home`, not `$XDG_DATA_HOME`
- `$env: xdg-data-home` => `$XDG_DATA_HOME`

> 注： 如果 `$env:` 式子中包含 `*`， 那麼自動轉換功能也會被停用。

目前支援的語法：

- `$const: exe_suffix ?   env * HOME ?   env * XDG_DATA_HOME ?   env * EXE_SUFFIX`
- `$env: home ? xdg-data-home ? exe_suffix ?    const * exe_suffix`

不支援:

- `$const: exe_suffix ? $env: home ? xdg-data-home ? exe_suffix`

如果要支援這種語法的話, 那麼解析會變得麻煩，並且 `$env: exe_suffix` 與 `$const: exe_suffix` 很容易搞混。

### base

這些是一些基本目錄，也可以說是標準目錄。
使用 `$dir: name` (e.g. `$dir: dl`) 或者是 `$dir: alias` (e.g. `$dir: download`) 來獲取 dir。
有不少內容都是透過 [dirs](https://docs.rs/dirs/latest/dirs/) 來獲取的，不過也有一些補充。

#### Linux

| name       | alias        | Linux `$dir`                             |
| ---------- | ------------ | ---------------------------------------- |
| home       |              | `$home`: (`/home/[username]`)            |
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
| tmp-rand   | tmp_random   | `$tmpdir/[random]`                       |
| temp       | temporary    | `env::temp_dir()`                        |
| cli-data   | cli_data     | `$xdg_data_home`                         |
| cli-cfg    | cli_config   | `$xdg_config_home`                       |
| cli-cache  | cli_cache    | `$xdg_cache_home`                        |
| empty      |              | ""                                       |

first_path 指的是第一個 `$PATH` 變數， last_path 則是最後一個。
若有 PATH 為 `/usr/local/bin:/usr/bin`，
則 `/usr/local/bin` 為 first_path, `/usr/bin` 為 last_path。

關於 tmp 與 temp

- tmp: 先獲取 `$env: tmpdir` 的值，若存在, 則使用該值。若不存在，使用 `env::temp_dir()` 獲取，判斷檔案路徑是否只讀，若是，則使用 `["$dir: cache", "tmp"]`
  - 有些平臺的 tmp 目錄對於普通使用者可能是隻讀的，沒錯，說的就是你： `/data/local/tmp`
- temp: 使用 `env::temp_dir()` 獲取, 不進行判斷
- tmp-rand: 生成隨機的臨時目錄，需要啟用 `rand` 功能

#### Android

- var:

  - sd = "/storage/self/primary"

對於沒有列出的內容，使用 linux 的資料

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
| tmp-rand   | tmp_random   | `$tmpdir/[random]`                    |
| temp       | temporary    | `env::temp_dir()`:(`/data/local/tmp`) |
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
| home                     |                          | `C:\Users\[username]`                                               |
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
| tmp-rand                 | tmp_random               | `$tmpdir\[random]`                                                  |
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
| home       |              | `/Users/[username]`                 |
| cache      |              | `$home/Library/Caches`              |
| cfg        | config       | `$home/Library/Application Support` |
| data       |              | `$home/Library/Application Support` |
| local-data | local_data   | `$home/Library/Application Support` |
| local-cfg  | local_config | `$home/Library/Application Support` |
| desktop    |              | `$home/Desktop`                     |
| doc        | document     | `$home/Documents`                   |
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
| tmp-rand   | tmp_random   | `$tmpdir/[random]`                  |
| temp       | temporary    | `env::temp_dir()`                   |
| cli-data   | cli_data     | `$home/Library/Application Support` |
| cli-cfg    | cli_config   | `$home/Library/Application Support` |
| cli-cache  | cli_cache    | `$home/Library/Caches`              |
| empty      |              | ""                                  |

### project

為專案生成指定目錄。
大部分資料從 [directories](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html) 獲取。

使用 `$proj(qualifier.  organization.   application): name` (e.g. `$proj(org. moz. ff): data`) 或者是 `$proj(com.company-name.app-name): alias` 來獲取 project dir。

接下來假設專案為 `(org. moz. ff)`

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

`$proj` 支援的 `?` 語法比其他的型別要更復雜一點，因為其他型別沒有 `()`,而它有。
別灰心，如果您已經掌握了核心語法，那麼相信您定能在幾分鐘快速掌握 `$proj` 的 `??`語法。

假設有三個專案：

- (org.moz.ff)
- (com. gg. cr)
- (com .ms .eg)

第一個例子為：

```rs
["
    $proj (org. moz. ff ): runtime ? data ?? state ?
    (com . gg . cr): cfg ?? cache ?
    (com .ms .eg): local-data ? data
"]
```

我們開始解析 ff 專案的 runtime, 很不幸，它不存在。
我們接著解析 data！太好了，我們發現它的值是存在的。
~~然而，好景不長，好不容易找到了一個存在的值，這時候天道的考驗來了，只有透過才能羽化飛昇。怎麼回事？我現在的修為連元嬰期都沒有，而且靈力和神識還在不斷潰散~~
（不好意思，拿錯劇本了...
由於有兩個 '?' ，所以還需要判斷檔案路徑是否存在。
~~（所以說 data 君 的身死道消也不是沒有理由的嗎~~
由於此專案的 data 的路徑不存在，因此下一個幸運兒是 state。
~~遠處，一聲充滿不甘，又直透人心的怒吼聲傳來：“我命由我不由天，我只是錯...”
突然間，聲音戛然而止，未幾，遠處傳來了嬰兒的啼哭聲，讓一切都充滿著陰森詭異的味道。~~
很遺憾，它也沒能透過，因為它的值不存在。
至此，ff 陣營的成員全軍覆沒，沒有一個解析成功。
於是，我們接著解析 cr 專案，幸運的是，第一次就成功了。
cr 的 cfg 不僅值存在，路徑也是存在的。
最終的勝者是 cr 家的 cfg，儘管如此，但她卻不怎麼開心的樣子。在離開前，她嘴角邊還嘟喃道：“明明只是個解析器，竟然敢這麼囂張，哼！”

第二個例子為：

```rs
["
    $proj (org . moz . ff )：runtime ？ data ？？ state ？
    (com . gg . cr)： cfg ？？ cache ？
    (com . ms . eg)： local-data ？ data
"]
```

Q: 咦？我怎麼沒看出來，這與第一個例子有何不同？
A: 屆時，汝自會知曉。
（此時一道亮光劃破天際，而眼前之人早已不見蹤影）
Q: 真是奇怪，我的腦袋裡怎麼會多出些奇奇怪怪的記憶？我怕不是睡迷糊了。對了，我剛剛在和誰說話來著？
