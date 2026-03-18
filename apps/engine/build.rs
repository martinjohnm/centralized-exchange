


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Tell Cargo to re-run if the proto file changes
    println!("cargo:rerun-if-changed=../../proto/order.proto");

    // 2. Compile the proto file
    // First arg: path to the .proto file
    // Second arg: the directory to search for imports
    prost_build::compile_protos(&["../../proto/order.proto"], &["../../proto/"])?;
    
    Ok(())
}