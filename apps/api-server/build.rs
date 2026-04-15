use std::io::Result;

fn main() -> Result<()> {
    // 1. Tell Cargo to re-run if the proto file changes
    println!("cargo:rerun-if-changed=../../proto/exchange.proto");

    // 2. Configure prost to add Serde traits
    let mut config = prost_build::Config::new();
    
    // This adds #[derive(serde::Serialize, serde::Deserialize)] to all generated types
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    // 3. Compile the proto file using the config
    config.compile_protos(
        &["../../proto/exchange.proto"], // Your proto file
        &["../../proto/"]                // The include directory
    )?;

    Ok(())
}