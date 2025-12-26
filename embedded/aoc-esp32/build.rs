use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo::rerun-if-env-changed=ESP_LOG");

    fs::write(
        out.join("log_filter.rs"),
        format!(
            "const FILTER_MAX: log::LevelFilter = log::LevelFilter::{};",
            parse_esp_log()
        ),
    )
		.unwrap();

    println!("cargo::rustc-link-arg-bins=-Tlinkall.x");

	#[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))]
	println!("cargo::rustc-link-arg=-nostartfiles");

    println!("cargo::rerun-if-changed=build.rs");	
}

fn parse_esp_log() -> &'static str {
    let level = env::var("ESP_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    match level.as_str() {
        "trace" => "Trace",
        "debug" => "Debug",
        "info" => "Info",
        "warn" => "Warn",
        "error" => "Error",
        _ => panic!("invalid value for ESP_LOG env: '{level}'"),
    }
}
