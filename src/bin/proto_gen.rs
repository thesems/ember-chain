extern crate prost_build;
extern crate tonic;

use std::env;
use std::io::Result;

fn main() -> Result<()> {
    env::set_var("OUT_DIR", "src/proto/");
    prost_build::compile_protos(&["src/proto/types.proto"], &["src/proto/"])?;
    tonic_build::compile_protos("src/proto/types.proto");
    Ok(())
}
