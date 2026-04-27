#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    fmt::Debug,
    process::exit,
};

use::inquire::{
    Text,
    Select,
    validator::Validation,
};
use reqwest::blocking::get;
use serde::Deserialize;
use url::{
    Url
};

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
                exit(0);
            }
        };
}

fn enterChoice(prompt: &str, condition: fn(&str) -> bool) -> String {
    return match Text::new(prompt)
        .with_validator(|input: &str| { if condition(input) { Ok(Validation::Invalid("Field cannot be empty!".into())) } else { Ok(Validation::Valid) }})
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                exit(0);
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

fn stringList(list: Vec<&str>) -> Vec<String> {
    list.into_iter().map(|s| s.to_string()).collect()
}

fn isEmpty(text: &str) -> bool {
    text.is_empty()
}

fn isntValidUrl(url: &str) -> bool {
    !Url::parse(url).is_ok() || url.is_empty()
}

fn error(text: String) {
    println!("{}", text);
    exit(0);
}

fn isValidModpackLink(url: &str) -> bool {
    if Url::parse(url).is_ok() {
        let parsedUrl = Url::parse(url).unwrap();
        match parsedUrl.host_str() {
            Some("modrinth.com") => {
                let mut segments = parsedUrl.path_segments().map(|s| s.collect::<Vec<_>>()).unwrap();
                //https://modrinth.com/modpack/fabulously-optimized
                //https://api.modrinth.com/v2/project/1KVo5zza/version
                if segments[0] != "modpack" { error(format!("Invalid Modrinth URL \"{}\"!", url)); }
                segments.truncate(2);
            },
            Some("curseforge.com") => {

            },
            _ => {}
        }
    }
    return false
}

fn main() {
    let name = enterChoice("Enter server name:", isEmpty);

    let selectedLauncher = selectChoice("Select a launcher:", stringList(vec!["Fabric", "Neoforge", "Quilt", "Forge"]));

    let mut versions = getVersions(&selectedLauncher, true);
    versions.insert(0, "Show experimental versions".to_string());
    let mut selectedVersion = selectChoice("Select a version:", versions);
    if selectedVersion == "Show experimental versions" {
        selectedVersion = selectChoice("Select a version:", getVersions(&selectedLauncher, false));
    }

    let modpackLink = enterChoice("Enter a modpack url (empty for manual setup)", isntValidUrl);
    if !modpackLink.is_empty() {

    }

    println!("Output:\nName: {}\nLauncher: {}\nVersion: {}\nModpack: {}", name, selectedLauncher, selectedVersion, modpackLink);
}

////Process:
//Ask for Modpack URL
//Ask for Name (if empty & modpack URL use modpack name)
//(if no modpack) Ask for launcher
//(if modpack) Ask to select modpack version
//(if no modpack) Ask to select Minecraft version
//Ask whether user accepts Minecraft EULA
//Ask whether user wants scripts generated
////