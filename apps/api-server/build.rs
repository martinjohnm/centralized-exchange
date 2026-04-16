use std::io::Result;

fn main() -> Result<()> {
    // Tell Cargo to rerun if the proto file changes
    println!("cargo:rerun-if-changed=../../proto/exchange.proto");

    let mut config = prost_build::Config::new();
    
    // 1. Add standard Serde traits and CamelCase renaming
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    // 2. Enable 'serde_as' to allow the String <-> Number mapping
    config.type_attribute(".", "#[serde_with::serde_as]");

    // 3. Flatten 'action' so the JSON doesn't need an extra nesting layer
    config.field_attribute("exchange.ExchangeRequest.action", "#[serde(flatten)]");

    // 4. Target your IDs and Timestamps to be Strings in the JSON
    let fields_to_string = [
        "exchange.ExchangeRequest.user_id",
        "exchange.ExchangeRequest.timestamp",
        "exchange.CreateOrder.client_id",
        "exchange.CancelOrder.client_id",
        "exchange.CancelOrder.engine_id",
        "exchange.Trade.maker_id",
        "exchange.Trade.taker_id",
        "exchange.ExecutionReport.user_id",
        "exchange.ExecutionReport.client_id",
        "exchange.Candle.timestamp",
    ];

    for field in fields_to_string {
        // This is the specific line that solves the JS precision problem
        config.field_attribute(field, "#[serde_as(as = \"serde_with::DisplayFromStr\")]");
    }

    config.compile_protos(
        &["../../proto/exchange.proto"],
        &["../../proto/"]
    )?;

    Ok(())
}