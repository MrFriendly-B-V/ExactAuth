use std::path::{Path, PathBuf};
use std::{fs, io};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=./protos");
    let proto_files = get_proto_files(&PathBuf::from("./protos"))?;
    compile_protos(&proto_files, "./protos")?;

    Ok(())
}

fn compile_protos(protos: &[String], path: &str) -> io::Result<()> {
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");
    config.type_attribute(".", r#"#[derive(serde::Serialize, serde::Deserialize)]"#);
    config.type_attribute(".", r#"#[typeshare::typeshare]"#);

    config.compile_protos(protos, &[path])?;
    Ok(())
}

fn get_proto_files(path: &Path) -> io::Result<Vec<String>> {
    let mut buf = Vec::new();

    let dirs = fs::read_dir(path)?;
    for dir in dirs {
        let dir = dir?;
        if dir.path().is_dir() {
            buf.extend_from_slice(&get_proto_files(&dir.path())?);
        } else if let Some(ext) = dir.path().extension() {
            if ext.eq("proto") {
                buf.push(dir.path().to_string_lossy().to_string());
            }
        }
    }

    Ok(buf)
}
