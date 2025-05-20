// use std::io;

// use envpath::EnvPath;

// #[test]
// fn into_envpath() {
//     let arr = ["$env:home", "dev"];
//     let path: EnvPath = arr.into();
//     dbg!(path.de().display());
// }

// #[test]
// fn from_vertor() {
//     use envpath::EnvPath;
//     let v: EnvPath = vec!["$env:home"].into();
//     assert_eq!(v.get_raw(), &["$env:home"]);
//     dbg!(v.de().display());
// }

// #[test]
// fn create_from_str_iter() {
//     use envpath::EnvPath;
//     let v1 = EnvPath::create_from_str_iter(["$env:home"]);
//     dbg!(v1.display(), v1.exists());
// }

// #[test]
// fn test_unsafe_u8() {
//     let mut casing = String::from("xdg-data-home");

//     if casing.contains('-') {
//         for i in unsafe { casing.as_bytes_mut() } {
//             if *i == b'-' {
//                 *i = b'_';
//             }
//         }
//     }
//     dbg!(&casing);
// }

// #[test]
// fn new_project_dir() {
//     let proj = get_proj().unwrap();
//     dbg!(proj);
// }

// fn get_proj() -> io::Result<envpath::ProjectDirs> {
//     EnvPath::new_project("com", "xy", "z")
// }
#[test]
fn readme_001() {
  use std::fs::File;

  use envpath::EnvPath;
  use serde::{Deserialize, Serialize};

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
}

#[test]
fn readme_ser() {
  use envpath::EnvPath;
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Default, Serialize, Deserialize)]
  #[serde(default)]
  struct Cfg<'a> {
    dir: Option<EnvPath<'a>>,
  }

  let dir = Some(EnvPath::from(["$env: user ?? userprofile ?? home"]));

  let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
  println!("{ron_str}");

  std::fs::write("test.ron", ron_str)
    .expect("Failed to write the ron cfg to test.ron");
}

#[test]
fn new_envpath() {
  let _arr = ["$dir: test ? env * HOME", "$val: rand-8"];
  //
}
