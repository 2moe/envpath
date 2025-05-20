use std::io;

use envpath::EnvPath;

#[ignore]
#[test]
fn test_proj_dir() -> io::Result<()> {
  let new_project = EnvPath::new_project("me", "tmoe", "glossa-cli")?;
  let dir = new_project.data_dir();
  println!("{dir:?}");
  Ok(())
}
