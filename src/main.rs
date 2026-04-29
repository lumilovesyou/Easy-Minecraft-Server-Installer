#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    fmt::{Debug, format}, fs::{
        create_dir, exists
    }, process::exit, vec
};

use::inquire::{
    Text,
    Select,
    Confirm,
    validator::Validation,
};
use reqwest::blocking::get;
use serde::Deserialize;
use serde_json::Value;
use url::{
    Url,
};

#[derive(Deserialize, Debug)]
struct FabricGameVersion {
    version: String,
    stable: bool,
}

fn optionInput(prompt: &str, options: Vec<String>) -> String {
    return match Select::new(prompt, options)
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                exit(0);
            }
        };
}

fn textInput<F>(prompt: &str, condition: F) -> String where F: Fn(&str) -> &str, {
    return match Text::new(prompt)
        .with_validator(|input: &str| { if !condition(input).is_empty() { Ok(Validation::Invalid(condition(input).into())) } else { Ok(Validation::Valid) }})
        .prompt() {
            Ok(val) => val,
            Err(_e) => {
                exit(0);
            }
        };
}

fn confirmationInput(prompt: &str) -> bool {
    return Confirm::new(prompt)
        .prompt()
        .unwrap();
}

fn stringList(list: Vec<&str>) -> Vec<String> {
    list.into_iter().map(|s| s.to_string()).collect()
}

fn isEmpty(text: &str) -> &str {
    if text.is_empty() {
        return "Cannot be empty";
    }
    return "";
}

fn modpackIsEmpty(text: &str, doModpack: bool) -> &str {
    if text.is_empty() && !doModpack {
        return "Cannot be empty";
    }
    return "";
}

fn isValidModpackLink(url: &str) -> &str {
    if url.is_empty() { return "" };
    if Url::parse(url).is_ok() {
        let parsedUrl = Url::parse(url).unwrap();
        match parsedUrl.host_str().unwrap() {
            "modrinth.com" => {
                let segments = parsedUrl.path_segments().map(|s| s.collect::<Vec<_>>()).unwrap();
                //https://modrinth.com/modpack/fabulously-optimized
                //https://api.modrinth.com/v2/project/1KVo5zza/version
                if segments[0] == "modpack" && segments.len() > 1 { return "" }
            },
            "curseforge.com" => {

            },
            _ => {}
        }
    }
    return "Invalid modpack URL";
}

////API Stuff
fn getMinecraftVersions(launcher: &String, filter: bool) -> Vec<String> {
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

fn getModpackMinecraftVersions(url: &String) -> Vec<String> {
    match getModpackHost(url).as_str() {
        "modrinth.com" => {
            let modpack: Value = getURL(format!("https://api.modrinth.com/v2/project/{}", getModpackName(url)));
            let mut modpackMinecraftVersions: Vec<String> = vec![];
            for i in modpack["game_versions"].as_array().unwrap() {
                modpackMinecraftVersions.push(i.as_str().unwrap().to_string());
            }
            modpackMinecraftVersions.reverse();
            return modpackMinecraftVersions;
        },
        _ => { vec![] }
    }
}

fn getModpackVersion(url: &str, minecraftVersion: String) -> (Vec<String>, Vec<String>) {
    match getModpackHost(url).as_str() {
        "modrinth.com" => {
            let modpack: Value = getURL(format!("https://api.modrinth.com/v2/project/{}/version?game_versions=[\"{}\"]", getModpackName(url), minecraftVersion));
            let mut modpackVersions: Vec<String> = vec![];
            let mut modpackVersionIDs: Vec<String> = vec![];
            for i in modpack.as_array().unwrap() {
                modpackVersions.push(i["name"].as_str().unwrap().to_string());
                modpackVersionIDs.push(i["id"].as_str().unwrap().to_string());
            }
            return (modpackVersions, modpackVersionIDs);
        },
        _ => { (vec![], vec![]) }
    }
}

fn getModpackHost(url: &str) -> String {
    let parsed = Url::parse(url).unwrap();
    parsed.host_str().unwrap().to_owned()
}

fn getModpackName(url: &str) -> String {
    let parsed = Url::parse(url).unwrap();
    parsed.path_segments().unwrap().nth(1).unwrap().to_owned()
}

fn getURL(url: String) -> Value {
    get(url).unwrap().json().unwrap()
}
////

fn main() {
    let modpackURL = textInput("Enter a modpack URL (optional)", isValidModpackLink);
    let doModpack = !modpackURL.is_empty();
    let mut modpackJSON: Value = Value::Null;

    if doModpack {
        modpackJSON = getURL(format!("https://api.modrinth.com/v2/project/{}", getModpackName(&modpackURL)));
    }

    let mut serverName = textInput("Enter the server's name", |input| modpackIsEmpty(input, doModpack));
    if serverName.is_empty() {
        serverName = modpackJSON["title"].as_str().unwrap().to_string();
    }

    if !doModpack {
        let selectedLauncher = optionInput("Select a launcher:", stringList(vec!["Fabric", "Neoforge", "Quilt", "Forge"]));

        let mut versions = getMinecraftVersions(&selectedLauncher, true);
        versions.insert(0, "Show experimental versions".to_string());
        let mut selectedVersion = optionInput("Select a version:", versions);
        if selectedVersion == "Show experimental versions" {
            selectedVersion = optionInput("Select a version:", getMinecraftVersions(&selectedLauncher, false));
        }
    } else {
        let selectedMinecraftVersion = optionInput("Selection a Minecraft version:", getModpackMinecraftVersions(&modpackURL));
        let (modpackVersions, modpackIDs) =  getModpackVersion(&modpackURL, selectedMinecraftVersion);
        let modpackVersions: Vec<String> = modpackVersions.into_iter().zip(modpackIDs).map(|(version, id)| format!("{} ({})", version, id)).collect();
        let selectedModpackVersion = optionInput("Selection a Modpack release:", modpackVersions);
    }

    let acceptsEULA = confirmationInput("Accept the Minecraft EULA?");
    let generateScripts = confirmationInput("Generate startup scripts?");
    
    let mut folderName = serverName.clone();
    if exists(&folderName).unwrap() {
        let mut i = 0;
        loop {
            folderName = format!("{} ({})", serverName, i);
            if !exists(&folderName).unwrap() {
                break;
            }
            i += 1;
        }
    }

    create_dir(folderName).unwrap();
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