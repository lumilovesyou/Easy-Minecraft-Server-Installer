#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::fmt::Debug;

use::inquire::{
    Text,
    Select,
    validator::Validation,
};
use reqwest::blocking::get;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FabricGameVersion {
    version: String,
    stable: bool,
}

fn selectChoice(prompt: &str, options: Vec<String>) -> String {
    return match Select::new(prompt, options)
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                return String::new();
            }
        };
}

fn getVersions(launcher: &String, filter: bool) -> Vec<String> {
    match launcher.as_str() {
        "Fabric" => {
            let mut versions: Vec<FabricGameVersion> =
                get("https://meta.fabricmc.net/v2/versions/game")
                .unwrap()
                .json()
                .unwrap();

            if filter {
                versions.retain(|v| v.stable);
            }
            
            versions.into_iter().map(|v| v.version).collect()
        },
        _ => vec![],
    }
}

fn main() {
    let name = match Text::new("Enter server name:")
        .with_validator(|input: &str| { if input.trim().is_empty() { Ok(Validation::Invalid("Name cannot be empty!".into())) } else { Ok(Validation::Valid) }})
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                return;
            }
        };
    
    let givenLaunchers: Vec<String> = vec!["Fabric", "Neoforge", "Quilt", "Forge"].into_iter().map(|s| s.to_string()).collect();
    let selectedLauncher = selectChoice("Select a launcher:", givenLaunchers);

    let mut versions = getVersions(&selectedLauncher, true);
    versions.insert(0, "Show experimental versions".to_string());
    let selectedVersion = selectChoice("Select a version:", versions);

    println!("Output:\nName: {}\nLauncher: {}\nVersion: {}", name, selectedLauncher, selectedVersion);
}