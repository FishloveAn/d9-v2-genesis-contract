use d9_genesis_contract::{
    inventory, parse, schema, validate, validate_inventory, ArtifactBinding,
};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = match args.as_slice() {
        [command] if command == "schema" => serde_json::to_value(schema()),
        [command] if command == "binding-schema" => {
            serde_json::to_value(schemars::schema_for!(ArtifactBinding))
        }
        [command] if command == "inventory" => {
            validate_inventory()?;
            serde_json::to_value(inventory())
        }
        [command, path] if command == "check" => {
            let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let input = parse(&bytes).map_err(|e| format!("contract_decode: {e}"))?;
            match validate(&input) {
                Ok(report) => serde_json::to_value(report),
                Err(error) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&error).map_err(|e| e.to_string())?
                    );
                    return Err("input contract rejected (not a release-gate result)".into());
                }
            }
        }
        _ => return Err(
            "usage: d9-genesis-contract schema | binding-schema | inventory | check <input.json>"
                .into(),
        ),
    }
    .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
