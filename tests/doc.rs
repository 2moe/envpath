use std::io;

use testutils::os_cmd::{RunnableCommand, presets::CargoDoc};

#[ignore]
#[test]
fn build_and_open_rust_doc() -> io::Result<()> {
  let build = |pkg| {
    CargoDoc::default()
      .with_pkg(pkg)
      .with_enable_private_items(false)
      // .with_open(false)
      .run()
  };

  build("envpath")
}
