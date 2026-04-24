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

fn selectChoice<'a>(prompt: &'a str, options: Vec<&'a str>) -> &'a str {
    return match Select::new(prompt, options)
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                return "";
            }
        };
}

fn getVersions(launcher: &str, filter: bool) -> Vec<&str> {
    match launcher {
        "Fabric" => {
            let mut versions: Vec<FabricGameVersion> =
                get("https://meta.fabricmc.net/v2/versions/game")
                .unwrap()
                .json()
                .unwrap();

            if filter {
                versions.retain(|v| v.stable);
            }
            
            let mut versionsOnly: Vec<&str> = vec![];
            versions.iter().for_each(|v| versionsOnly.push(v.version.as_str()));
            return versionsOnly;
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
    
    let givenLaunchers: Vec<&str> = vec!["Fabric", "Neoforge", "Quilt", "Forge"];
    let selectedLauncher = selectChoice("Select a launcher:", givenLaunchers);

    let mut versions = getVersions(selectedLauncher, true);
    versions.insert(0, "Show experimental versions".to_string());
    let selectedVersion = selectChoice("Select a version:", versions);

    println!("Output:\nName: {}\nLauncher: {}\nVersion: {}", name, selectedLauncher, selectedVersion);
}