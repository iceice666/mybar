use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct IconEntry {
    #[serde(rename = "iconName")]
    icon_name: String,
    #[serde(rename = "appNames")]
    app_names: Vec<String>,
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest_dir).join("assets/icon_map.json");

    println!("cargo::rerun-if-changed={}", json_path.display());

    let json = fs::read_to_string(&json_path).expect("failed to read icon_map.json");
    let entries: Vec<IconEntry> = serde_json::from_str(&json).expect("failed to parse icon_map.json");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("icon_map.rs");
    let mut f = fs::File::create(&dest).unwrap();

    writeln!(f, "pub fn app_name_to_icon(app: &str) -> &'static str {{").unwrap();
    writeln!(f, "    match app {{").unwrap();

    for entry in &entries {
        for app in &entry.app_names {
            let escaped = app.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(f, "        \"{}\" => \"{}\",", escaped, entry.icon_name).unwrap();
        }
    }

    writeln!(f, "        _ => \":default:\",").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();
}
