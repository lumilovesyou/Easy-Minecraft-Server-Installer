#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    fmt::Debug, fs::{
        File, create_dir_all, exists, write
    }, io::{Cursor, Read, Write, copy}, path::Path, process::exit, vec
};
use::inquire::{
    Text,
    Select,
    Confirm,
    validator::Validation,
};
use zip::ZipArchive;
use reqwest::blocking::get;
use serde::Deserialize;
use serde_json::{Value, from_slice};
use indicatif::ProgressBar;
use url::Url;

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
        "Fabric" | "Quilt" => {
            let versions: Value =
                get(format!("https://meta.{}/versions/game", if launcher.to_lowercase() == "fabric" { "fabricmc.net/v2" } else { "quiltmc.org/v3" }))
                .unwrap()
                .json()
                .unwrap();
            
            let versionsArray = versions.as_array().unwrap();

            versionsArray.iter().filter_map(|v| {
                let stable = v["stable"].as_bool().unwrap();

                if filter && !stable {
                    return None;
                }

                v["version"].as_str().map(|s| s.to_string())
            })
            .collect()
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

fn getModpackLauncher(url: &str, modpackVersion: &str) -> String {
    match getModpackHost(url).as_str() {
        "modrinth.com" => {
            let modpack: Value = getURL(format!("https://api.modrinth.com/v2/project/{}/version/{}", getModpackName(url), modpackVersion));
            return modpack["loaders"].as_array().unwrap().first().unwrap().as_str().unwrap().to_owned();
        },
        _ => { String::new() }
    }
}

fn downloadMod(modURL: &str, modPath: &str, filePath: &str) {
    match getModpackHost(modURL).as_str() {
        "cdn.modrinth.com" => {
            let modBytes = get(modURL).unwrap().bytes().unwrap();
            let pathStr = format!("{}/{}", filePath, modPath);
            let path = Path::new(pathStr.as_str());
            if let Some(parent) = path.parent() {
                create_dir_all(parent).unwrap();
            }
            write(path, modBytes).unwrap();
        },
        _ => { }
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
    println!("Getting URL: {}", url);
    get(url).unwrap().json().unwrap()
}
////

////Launchers stuff
fn downloadLauncherAndSetup(launcher: &str, version: String, path: &str) -> anyhow::Result<()> { 
    match launcher {
        "fabric" => {
            let loaderVersionPart = getURL(format!("https://meta.fabricmc.net/v2/versions/loader/{}", version)).as_array().unwrap().first().unwrap()["loader"].clone();
            let loaderVersion = loaderVersionPart["version"].as_str().unwrap();
            let installerVersionPart = getURL("https://meta.fabricmc.net/v2/versions/installer".to_string()).as_array().unwrap().first().unwrap()["version"].clone();
            let installerVersion = installerVersionPart.as_str().unwrap();
            println!("Getting url: {}",format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar", version, loaderVersion, installerVersion) );
            let jarBytes = get(format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar", version, loaderVersion, installerVersion))?.bytes()?;
            write(format!("{}/server.jar", path), jarBytes)?;
            Ok(())
        },
        _ => {
            Ok(())
        }
    }
}

fn downloadModpackMods(url: &str, version: String, pathStr: &str) -> anyhow::Result<()> {
    match getModpackHost(url).as_str() {
        "modrinth.com" => {
            let name = getModpackName(url);
            let versionInfo: Value = getURL(format!("https://api.modrinth.com/v2/project/{}/version/{}", name, version));
            let mut fileURL = "";
            for i in versionInfo["files"].as_array().unwrap() {
                if i["primary"].as_bool().unwrap() {
                    fileURL = i["url"].as_str().unwrap();
                    break;
                }
            }
            let fileBytes = get(fileURL)?.bytes()?;
            let cursor = Cursor::new(fileBytes);
            let mut archive = ZipArchive::new(cursor)?;

            let path =  Path::new(pathStr);

            println!("Copying bundled files");

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let name = file.name();

                if !name.starts_with("overrides/") {
                    continue;
                }

                let relativePath = name.strip_prefix("overrides/").unwrap();

                if relativePath.is_empty() {
                    continue;
                }

                let outputPath = path.join(relativePath);

                if file.is_dir() {
                    create_dir_all(outputPath)?;
                } else {
                    if let Some(parent) = outputPath.parent() {
                        create_dir_all(parent)?;
                    }

                    let mut outputFile = File::create(outputPath)?;
                    copy(&mut file, &mut outputFile)?;
                }
            };

            println!("Finding mods");

            let index: Value = {
                let mut indexFile = archive.by_name("modrinth.index.json")?;
                let mut buf = vec![];
                indexFile.read_to_end(&mut buf)?;
                from_slice(&buf)?
            };
            let mut modPaths: Vec<&str> = vec![];
            let mut modURLs: Vec<&str> = vec![];
            let files = index["files"].as_array().unwrap();
            for file in files {
                modPaths.push(file["path"].as_str().unwrap());
                modURLs.push(file["downloads"].as_array().unwrap().first().unwrap().as_str().unwrap());
            }
            if modPaths.len() > 0 {
                println!("Downloading mods ({})", modPaths.len());
                let progressBar = ProgressBar::new(modPaths.len() as u64);
                for i in 0..modPaths.len() {
                    downloadMod(modURLs[i], modPaths[i], pathStr);
                    progressBar.inc(1);
                }
                progressBar.finish();
            }

            //Download zip file
            //Copy over bundled files
            //Download mods from the thingie
            //Remember to display progress bar
            Ok(())
        },
        _ => { 
            Ok(())
        }
    }

}
////

fn main() -> anyhow::Result<()> {
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
    let mut selectedModpackVersion = String::new();
    let mut selectedLauncher = String::new();
    if !doModpack {
        selectedLauncher = optionInput("Select a launcher:", stringList(vec!["Fabric", "Neoforge", "Quilt", "Forge"]));

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
        selectedModpackVersion = optionInput("Selection a Modpack release:", modpackVersions);
        selectedModpackVersion = selectedModpackVersion.split("(").last().unwrap().split(")").nth(0).unwrap().to_string(); //This is so gross but I genuinely don't know enough Rust tidbits to make it cleaner
        selectedLauncher = getModpackLauncher(&modpackURL, &selectedModpackVersion);
    }

    //Quick files
    let acceptsEULA = confirmationInput("Accept the Minecraft EULA?");
    let generateScripts = confirmationInput("Generate startup scripts?");
    
    println!("Generating valid directory name");

    //Generate valid folder name
    let mut folderName = serverName.clone();
    if exists(&folderName)? {
        let mut i = 0;
        loop {
            folderName = format!("{} ({})", serverName, i);
            if !exists(&folderName)? {
                break;
            }
            i += 1;
        }
    }

    //Create folder
    create_dir_all(format!("{}/mods", folderName))?;


    println!("Downloading launcher jar");

    //Download launcher
    downloadLauncherAndSetup(&selectedLauncher, selectedMinecraftVersion, &folderName).unwrap();


    if doModpack {
        println!("Downloading modpack");
        downloadModpackMods(&modpackURL, selectedModpackVersion, &folderName)?;
    }

    //Create quick files
    if acceptsEULA {
        println!("Accepting EULA");
        let mut minecraftEULA = File::create(format!("{}/eula.txt", folderName))?;
        minecraftEULA.write(b"#https://aka.ms/MinecraftEULA\neula=true")?;
    }
    if generateScripts {
        println!("Creating startup scripts");
        let mut shStart = File::create(format!("{}/start.sh", folderName))?;
        shStart.write(b"#!/usr/bin/env bash\njava -Xmx4G -jar server.jar nogui")?;
        let mut batStart = File::create(format!("{}/start.bat", folderName))?;
        batStart.write(b"java -Xmx4G -jar server.jar nogui\npause")?;
    }

    Ok(())
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