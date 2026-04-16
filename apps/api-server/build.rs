use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../proto/exchange.proto");

    let mut config = prost_build::Config::new();
    
    // 1. Add Serde traits to all types
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    // 2. Add CamelCase renaming globally 
    // This allows the JSON to use "userId" instead of "user_id"
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    // 3. Flatten the 'action' field in ExchangeRequest
    // This removes the need for the "action": { ... } wrapper in your JSON
    config.field_attribute("exchange.ExchangeRequest.action", "#[serde(flatten)]");

    config.compile_protos(
        &["../../proto/exchange.proto"],
        &["../../proto/"]
    )?;

    Ok(())
}