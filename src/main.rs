#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    fmt::{Debug, format}, fs::{
        create_dir, exists
    }, process::exit
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

fn modpackIsEmpty(text: &str, modpackURL: String) -> &str {
    if text.is_empty() && modpackURL.is_empty() {
        return "Cannot be empty";
    }
    return "";
}

fn isValidModpackLink(url: &str) -> &str {
    if url.is_empty() { return "" };
    if Url::parse(url).is_ok() {
        let parsedUrl = Url::parse(url).unwrap();
        match parsedUrl.host_str() {
            Some("modrinth.com") => {
                let segments = parsedUrl.path_segments().map(|s| s.collect::<Vec<_>>()).unwrap();
                println!("{:?}", segments);
                //https://modrinth.com/modpack/fabulously-optimized
                //https://api.modrinth.com/v2/project/1KVo5zza/version
                if segments[0] == "modpack" && segments.len() > 1 { return "" }
            },
            Some("curseforge.com") => {

            },
            _ => {}
        }
    }
    return "Invalid modpack URL";
}

////API Stuff
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

fn getURL(url: String) -> Value {
    return get(url).unwrap().json().unwrap();
}
////

fn main() {
    let modpackURL = textInput("Enter a modpack URL (optional)", isValidModpackLink);
    let mut modpackJSON: Value = Value::Null;
    if !modpackURL.is_empty() {
        let parsed = Url::parse(&modpackURL).unwrap();
        let modpackTitle = parsed.path_segments().unwrap().nth(1).unwrap();
        modpackJSON = getURL(format!("https://api.modrinth.com/v2/project/{}", modpackTitle));
    }
    let mut serverName = textInput("Enter the server's name", |input| modpackIsEmpty(input, modpackURL.clone()));
    if serverName.is_empty() {
        serverName = modpackJSON["title"].as_str().unwrap().to_string();
    }
    println!("Modpack URL: {}\nServer Name: {}", modpackURL, serverName);

    if modpackURL.is_empty() {
        //Ask for launcher
        //Ask for Minecraft version
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

    /*
    let name = textInput("Enter server name:", isEmpty);

    let selectedLauncher = optionInput("Select a launcher:", stringList(vec!["Fabric", "Neoforge", "Quilt", "Forge"]));

    let mut versions = getVersions(&selectedLauncher, true);
    versions.insert(0, "Show experimental versions".to_string());
    let mut selectedVersion = optionInput("Select a version:", versions);
    if selectedVersion == "Show experimental versions" {
        selectedVersion = optionInput("Select a version:", getVersions(&selectedLauncher, false));
    }

    let modpackLink = textInput("Enter a modpack url (empty for manual setup)", isntValidUrl);
    if !modpackLink.is_empty() {

    }

    println!("Output:\nName: {}\nLauncher: {}\nVersion: {}\nModpack: {}", name, selectedLauncher, selectedVersion, modpackLink);
    */
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