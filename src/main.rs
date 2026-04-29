#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    fmt::Debug,
    fs::{
        File,
        create_dir,
        create_dir_all,
        exists,
        write,
    },
    io::Write,
    process::exit,
    vec,
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
use url::Url;

#[derive(Deserialize, Debug)]
struct FabricGameVersion {
    version: String,
    stable: bool,
}

fn stringList(list: Vec<&str>) -> Vec<String> {
    list.into_iter().map(|s| s.to_string()).collect()
}

////Input methods
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
////

////Validators
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
////

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

fn getModpackVersion(url: &str, minecraftVersion: &str) -> (Vec<String>, Vec<String>) {
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
////

////URL Stuff
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

////Launchers stuff
fn downloadLauncher(launcher: &str, version: String, path: &str) {
    match launcher {
        "fabric" => {
            let loaderVersionPart = getURL(format!("https://meta.fabricmc.net/v2/versions/loader/{}", version)).as_array().unwrap().first().unwrap()["loader"].clone();
            let loaderVersion = loaderVersionPart["version"].as_str().unwrap();
            let installerVersionPart = getURL("https://meta.fabricmc.net/v2/versions/installer".to_string()).as_array().unwrap().first().unwrap()["version"].clone();
            let installerVersion = installerVersionPart.as_str().unwrap();
            //https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{loader_version}/{installer_version}/server/jar
            let jarBytes = get(format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar", version, loaderVersion, installerVersion)).unwrap().bytes().unwrap();
            write(format!("{}/server.jar", path), jarBytes).unwrap();
        },
        _ => {}
    }
}
////

fn main() {
    //Modpack
    let modpackURL = textInput("Enter a modpack URL (optional)", isValidModpackLink);
    let doModpack = !modpackURL.is_empty();
    let mut modpackJSON: Value = Value::Null;

    if doModpack {
        modpackJSON = getURL(format!("https://api.modrinth.com/v2/project/{}", getModpackName(&modpackURL)));
    }


    //Name
    let mut serverName = textInput("Enter the server's name", |input| modpackIsEmpty(input, doModpack));
    if serverName.is_empty() {
        serverName = modpackJSON["title"].as_str().unwrap().to_string();
    }

    //Selection launcher/versions
    let mut selectedMinecraftVersion = String::new();
    if !doModpack {
        let selectedLauncher = optionInput("Select a launcher:", stringList(vec!["Fabric", "Neoforge", "Quilt", "Forge"]));

        let mut versions = getMinecraftVersions(&selectedLauncher, true);
        versions.insert(0, "Show experimental versions".to_string());
        let mut selectedMinecraftVersion = optionInput("Select a version:", versions);
        if selectedMinecraftVersion == "Show experimental versions" {
            selectedMinecraftVersion = optionInput("Select a version:", getMinecraftVersions(&selectedLauncher, false));
        }
    } else {
        selectedMinecraftVersion = optionInput("Selection a Minecraft version:", getModpackMinecraftVersions(&modpackURL));
        let (modpackVersions, modpackIDs) =  getModpackVersion(&modpackURL, &selectedMinecraftVersion);
        let modpackVersions: Vec<String> = modpackVersions.into_iter().zip(modpackIDs).map(|(version, id)| format!("{} ({})", version, id)).collect();
        let selectedModpackVersion = optionInput("Selection a Modpack release:", modpackVersions);
    }

    //Quick files
    let acceptsEULA = confirmationInput("Accept the Minecraft EULA?");
    let generateScripts = confirmationInput("Generate startup scripts?");
    
    //Generate valid folder name
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

    //Create folder
    create_dir_all(format!("{}/mods", folderName)).unwrap();

    //Download launcher
    downloadLauncher("fabric", selectedMinecraftVersion, &folderName);

    //Create quick files
    if acceptsEULA {
        let mut minecraftEULA = File::create(format!("{}/eula.txt", folderName)).unwrap();
        minecraftEULA.write(b"#https://aka.ms/MinecraftEULA\neula=true").unwrap();
    }

}

////To-do:
//Download launcher
//(If modpack) Download modpack file
//(If modpack) Download modpack mods
//Generate launch scripts
//Add NeoForge, Forge, and Quilt support
//Add CurseForge support
////

////Process:
//Ask for Modpack URL
//Ask for Name (if empty & modpack URL use modpack name)
//(if no modpack) Ask for launcher
//(if modpack) Ask to select modpack version
//(if no modpack) Ask to select Minecraft version
//Ask whether user accepts Minecraft EULA
//Ask whether user wants scripts generated
////