/*
 * Magical Launcher Core
 * Copyright (C) 2023 Broken-Deer <old_driver__@outlook.com> and contributors
 *
 * This program is free software, you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::folder::MinecraftLocation;

use super::PlatformInfo;

static DEFAULT_GAME_ARGS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "--username".to_string(),
        "${auth_player_name}".to_string(),
        "--version".to_string(),
        "${version_name}".to_string(),
        "--gameDir".to_string(),
        "${game_directory}".to_string(),
        "--assetsDir".to_string(),
        "${assets_root}".to_string(),
        "--assetIndex".to_string(),
        "${asset_index}".to_string(),
        "--uuid".to_string(),
        "${auth_uuid}".to_string(),
        "--accessToken".to_string(),
        "${auth_access_token}".to_string(),
        "--clientId".to_string(),
        "${clientid}".to_string(),
        "--xuid".to_string(),
        "${auth_xuid}".to_string(),
        "--userType".to_string(),
        "${user_type}".to_string(),
        "--versionType".to_string(),
        "${version_type}".to_string(),
        "--width".to_string(),
        "${resolution_width}".to_string(),
        "--height".to_string(),
        "${resolution_height}".to_string(),
    ]
});

static DEFAULT_JVM_ARGS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "\"-Djava.library.path=${natives_directory}\"".to_string(),
        // "\"-Djna.tmpdir=${natives_directory}\"".to_string(),
        // "\"-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}\"".to_string(),
        // "\"-Dio.netty.native.workdir=${natives_directory}\"".to_string(),
        "\"-Dminecraft.launcher.brand=${launcher_name}\"".to_string(),
        "\"-Dminecraft.launcher.version=${launcher_version}\"".to_string(),
        "\"-Dfile.encoding=UTF-8\"".to_string(),
        "\"-Dsun.stdout.encoding=UTF-8\"".to_string(),
        "\"-Dsun.stderr.encoding=UTF-8\"".to_string(),
        "\"-Djava.rmi.server.useCodebaseOnly=true\"".to_string(),
        "\"-XX:MaxInlineSize=420\"".to_string(),
        "\"-XX:-UseAdaptiveSizePolicy\"".to_string(),
        "\"-XX:-OmitStackTraceInFastThrow\"".to_string(),
        "\"-XX:-DontCompileHugeMethods\"".to_string(),
        "\"-Dcom.sun.jndi.rmi.object.trustURLCodebase=false\"".to_string(),
        "\"-Dcom.sun.jndi.cosnaming.object.trustURLCodebase=false\"".to_string(),
        "\"-Dlog4j2.formatMsgNoLookups=true\"".to_string(),
        "-cp".to_string(),
        "${classpath}".to_string(),
    ]
});

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LatestVersion {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u8,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VersionManifest {
    pub latest: LatestVersion,
    pub versions: Vec<VersionInfo>,
}

impl VersionManifest {
    pub async fn new() -> Result<VersionManifest> {
        let response =
            reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").await?;
        Ok(response.json::<VersionManifest>().await?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    // pub sha1: String,
    pub size: u64,
    pub url: String,
    pub id: String,
    pub total_size: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AssetIndexObjectInfo {
    pub hash: String,
    pub size: u32,
}

// #[derive(Debug, Clone, Deserialize, PartialEq)]
pub type AssetIndexObject = HashMap<String, AssetIndexObjectInfo>;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LibraryDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoggingFile {
    pub size: u64,
    pub url: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NormalLibrary {
    pub name: String,
    pub downloads: HashMap<String, LibraryDownload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Rule {
    pub action: String,
    pub os: Option<Platform>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NativeLibrary {
    pub name: String,
    pub downloads: HashMap<String, LibraryDownload>,
    pub classifiers: HashMap<String, LibraryDownload>,
    pub rules: Vec<Rule>,
    pub extract: Extract,
    pub natives: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlatformSpecificLibrary {
    pub name: String,
    pub downloads: HashMap<String, LibraryDownload>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LegacyLibrary {
    pub name: String,
    pub url: Option<String>,
    pub clientreq: Option<bool>,
    pub serverreq: Option<bool>,
    pub checksums: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Library {
    Normal(NormalLibrary),
    Native(NativeLibrary),
    PlatformSpecific(PlatformSpecificLibrary),
    Legacy(LegacyLibrary),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum LaunchArgument {
    String(String),
    Object(serde_json::map::Map<String, Value>),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Platform {
    pub name: String,
    pub version: Option<String>,
    // Add other platform properties if needed
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Arguments {
    pub game: Option<Vec<Value>>,
    pub jvm: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Logging {
    pub file: LoggingFileDownload,
    pub argument: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoggingFileDownload {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i32,
}

/// Resolved version.json
///
/// Use `new` to parse a Minecraft version json, and see the detail info of the version,
/// equivalent to `crate::core::version::Version::parse`.
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    /// The id of the version, should be identical to the version folder.
    pub id: String,
    pub arguments: Option<ResolvedArguments>,

    /// The main class full qualified name.
    pub main_class: String,
    pub asset_index: Option<AssetIndex>,

    /// The asset index id of this version. Should be something like `1.14`, `1.12`.
    pub assets: String,
    pub downloads: Option<HashMap<String, Download>>,
    pub libraries: Vec<ResolvedLibrary>,
    pub minimum_launcher_version: i32,
    pub release_time: String,
    pub time: String,
    pub version_type: String,
    pub logging: Option<HashMap<String, Logging>>,

    /// Recommended java version.
    pub java_version: JavaVersion,

    /// The minecraft version of this version.
    pub minecraft_version: String,

    /// The version inheritances of this whole resolved version.
    ///
    /// The first element is this version, and the last element is the root Minecraft version.
    /// The dependencies of \[\<a\>, \<b\>, \<c\>\] should be \<a\> -> \<b\> -> \<c\>, where c is a Minecraft version.
    pub inheritances: Vec<String>,

    /// All array of json file paths.
    ///
    /// It's the chain of inherits json path. The root json will be the last element of the array.
    /// The first element is the user provided version.
    pub path_chain: Vec<PathBuf>,
}

/// The raw json format provided by Minecraft.
///
/// Use `parse` to parse a Minecraft version json, and see the detail info of the version.
///
/// With `ResolvedVersion`, you can use the resolved version to launch the game.
///
/// ### Example
///
/// usage 1:
///
/// ```rust
/// use mgl_core::core::version::Version;
///
/// async fn fn_name() {
///     let version = reqwest::get("https://piston-meta.mojang.com/v1/packages/715ccf3330885e75b205124f09f8712542cbe7e0/1.20.1.json")
///         .await
///         .unwrap()
///         .json::<Version>()
///         .await
///         .unwrap();
///     println!("{:#?}", version);
/// }
/// ```
///
/// usage 2:
///
/// ```rust
/// use std::str::FromStr;
/// use mgl_core::core::version::Version;
///
/// async fn fn_name() {
///     let response = reqwest::get("https://piston-meta.mojang.com/v1/packages/715ccf3330885e75b205124f09f8712542cbe7e0/1.20.1.json")
///         .await
///         .unwrap()
///         .text()
///         .await
///         .unwrap();
///     let version = Version::from_str(&response).unwrap();
///     println!("{:#?}", version);
/// }
/// ```
///
/// usage 3:
///
/// ```rust
/// use mgl_core::core::version::Version;
/// use mgl_core::core::folder::MinecraftLocation;
/// use mgl_core::core::PlatformInfo;
///
/// async fn fn_name(version: Version) {
///     let platform = PlatformInfo::new().await;
///     let resolved_version = version.parse(&MinecraftLocation::new("test"), &platform).await;
///     println!("{:#?}", resolved_version);
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    pub time: Option<String>,
    pub r#type: Option<String>,
    pub release_time: Option<String>,
    pub inherits_from: Option<String>,
    pub minimum_launcher_version: Option<i32>,
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<Arguments>,
    pub main_class: Option<String>,
    pub libraries: Option<Vec<Value>>,
    pub jar: Option<String>,
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    pub downloads: Option<HashMap<String, Download>>,
    pub client: Option<String>,
    pub server: Option<String>,
    pub logging: Option<HashMap<String, Logging>>,
    pub java_version: Option<JavaVersion>,
    pub client_version: Option<String>,
}

impl Version {
    pub fn from_value(raw: Value) -> Result<Version, serde_json::Error> {
        serde_json::from_value(raw)
    }

    pub fn from_versions_folder(
        minecraft: MinecraftLocation,
        version_name: &str,
    ) -> Result<Version, std::io::Error> {
        let versions_folder = minecraft.versions;
        let path = versions_folder
            .join(version_name)
            .join(format!("{}.json", version_name));

        let raw = read_to_string(path)?;
        let version: Version = serde_json::from_str((&raw).as_ref())?;
        Ok(version)
    }

    pub fn from_str(raw: &str) -> Result<Version, serde_json::Error> {
        serde_json::from_str(raw)
    }

    /// parse a Minecraft version json
    pub async fn parse(
        &self,
        minecraft: &MinecraftLocation,
        platform: &PlatformInfo,
    ) -> Result<ResolvedVersion> {
        let mut inherits_from = self.inherits_from.clone();
        let versions_folder = &minecraft.versions;
        let mut versions = Vec::new();
        let mut inheritances = Vec::new();
        let mut path_chain = Vec::new();
        versions.push(self.clone());
        while let Some(inherits_from_unwrap) = inherits_from {
            inheritances.push(inherits_from_unwrap.clone());

            let path = versions_folder
                .join(inherits_from_unwrap.clone())
                .join(format!("{}.json", inherits_from_unwrap.clone()));
            path_chain.push(path.clone());
            let version_json = read_to_string(path)?;
            let version_json: Version = serde_json::from_str((&version_json).as_ref())?;

            versions.push(version_json.clone());
            inherits_from = version_json.inherits_from;
        }

        let mut assets = "".to_string();
        let mut minimum_launcher_version = 0;
        // let game_args = match self.arguments.clone() {
        //     None => DEFAULT_GAME_ARGS.clone(),
        //     Some(v) => match v.game {
        //         None => DEFAULT_GAME_ARGS.clone(),
        //         Some(v) => v,
        //     },
        // };
        // let jvm_args = match self.arguments.clone() {
        //     None => DEFAULT_JVM_ARGS.clone(),
        //     Some(v) => match v.jvm {
        //         None => DEFAULT_JVM_ARGS.clone(),
        //         Some(v) => v,
        //     },
        // };
        let game_args = DEFAULT_GAME_ARGS.clone();
        let jvm_args = DEFAULT_JVM_ARGS.clone();
        let mut release_time = "".to_string();
        let mut time = "".to_string();
        let mut version_type = "".to_string();
        let mut logging = HashMap::new();
        let mut main_class = "".to_string();
        let mut assets_index = AssetIndex {
            size: 0,
            url: "".to_string(),
            id: "".to_string(),
            total_size: 0,
        };
        let mut java_version = JavaVersion {
            component: "jre-legacy".to_string(),
            major_version: 8,
        };
        let mut libraries_raw = Vec::new();
        let mut downloads = HashMap::new();

        while versions.len() != 0 {
            let version = versions.pop().unwrap();
            println!("{}", version.id);
            minimum_launcher_version = std::cmp::max(
                version.minimum_launcher_version.unwrap_or(0),
                minimum_launcher_version,
            );

            // if let Some(arguments) = version.arguments {
            //     if let Some(mut game) = arguments.game {
            //         game_args.append(&mut game);
            //     }
            //     if let Some(mut jvm) = arguments.jvm {
            //         jvm_args.append(&mut jvm);
            //     }
            // }

            release_time = version.release_time.unwrap_or(release_time);
            time = version.time.unwrap_or(time);
            logging = version.logging.unwrap_or(logging);
            assets = version.assets.unwrap_or(assets);
            version_type = version.r#type.unwrap_or(version_type);
            main_class = version.main_class.unwrap_or(main_class);
            assets_index = version.asset_index.unwrap_or(assets_index);
            java_version = version.java_version.unwrap_or(java_version);

            if let Some(mut libraries) = version.libraries {
                libraries_raw.append(&mut libraries);
            }
            downloads = version.downloads.unwrap_or(downloads);
        }

        if main_class == ""
            || assets_index
                == (AssetIndex {
                    size: 0,
                    url: "".to_string(),
                    id: "".to_string(),
                    total_size: 0,
                })
            || downloads.len() == 0
        {
            panic!("Bad Version JSON");
        }
        Ok(ResolvedVersion {
            id: self.id.clone(),
            arguments: Some(ResolvedArguments {
                game: game_args,
                jvm: jvm_args,
                // game: resolve_arguments(game_args, platform).await,
                // jvm: resolve_arguments(jvm_args, platform).await,
            }),
            main_class,
            asset_index: self.asset_index.clone(),
            assets: self.assets.clone().unwrap_or("".to_string()),
            downloads: self.downloads.clone(),
            libraries: resolve_libraries(libraries_raw, platform).await,
            minimum_launcher_version,
            release_time,
            time,
            version_type,
            logging: self.logging.clone(),
            java_version: self.java_version.clone().unwrap_or(JavaVersion {
                component: "jre-legacy".to_string(),
                major_version: 8,
            }),
            minecraft_version: self.client_version.clone().unwrap_or(self.id.clone()),
            inheritances,
            path_chain,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedArguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLibrary {
    pub download_info: LibraryDownload,
    pub is_native_library: bool,
}

async fn _resolve_arguments(arguments: Vec<Value>, platform: &PlatformInfo) -> Vec<String> {
    let mut result = Vec::with_capacity(arguments.len());
    for argument in arguments {
        if argument.is_string() {
            result.push(argument.as_str().unwrap().to_string());
            continue;
        }
        if !argument.is_object() {
            continue;
        }
        let rules = argument["rules"].as_array();
        if let Some(rules) = rules {
            if !check_allowed(rules.clone(), platform) {
                continue;
            };
        }
        if argument["value"].is_string() {
            result.push(argument["value"].as_str().unwrap().to_string());
            continue;
        }
        if argument["value"].is_array() {
            result.extend(
                argument["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string()),
            );
        }
    }
    result
}

async fn resolve_libraries(libraries: Vec<Value>, platform: &PlatformInfo) -> Vec<ResolvedLibrary> {
    let mut result = Vec::new();
    for library in libraries {
        let rules = library["rules"].as_array();
        // check rules
        if let Some(rules) = rules {
            if !check_allowed(rules.clone(), &platform) {
                continue;
            }
        }
        // resolve native lib
        let classifiers = library["downloads"]["classifiers"].as_object();
        let natives = library["natives"].as_object();
        if classifiers.is_some() && natives.is_some() {
            let classifiers = classifiers.unwrap();
            let natives = natives.unwrap();
            let classifier_key = natives[&platform.name].as_str();
            if classifier_key.is_none() {
                continue;
            }
            let classifier = classifiers[classifier_key.unwrap()].as_object();
            if classifier.is_none() {
                continue;
            }
            let classifier = classifier.unwrap();
            result.push(ResolvedLibrary {
                download_info: LibraryDownload {
                    sha1: classifier["sha1"].as_str().unwrap_or("").to_string(),
                    size: classifier["size"].as_u64().unwrap_or(0),
                    url: match classifier["url"].as_str() {
                        Some(url) => url.to_string(),
                        None => continue,
                    },
                    path: match classifier["path"].as_str() {
                        Some(path) => path.to_string(),
                        None => continue,
                    },
                },
                is_native_library: true,
            });
        }
        // resolve common lib
        if library["downloads"]["artifact"].is_object() {
            result.push(ResolvedLibrary {
                download_info: serde_json::from_value(library["downloads"]["artifact"].clone())
                    .unwrap(),
                is_native_library: false,
            });
            continue;
        }
        let name = library["name"].as_str();
        if name == None {
            continue;
        }

        // resolve forge

        let name: Vec<&str> = name.unwrap().split(":").collect();
        if name.len() != 3 {
            continue;
        }

        let package = name.get(0).unwrap().replace(".", "/");
        let version = name.get(2).unwrap();
        let name = name.get(1).unwrap();

        let url;
        if let Some(url_) = library["url"].as_str() {
            url = url_;
        } else {
            url = "http://files.minecraftforge.net/maven/"
        }
        let path = format!("{package}/{name}/{version}/{name}-{version}.jar");
        result.push(ResolvedLibrary {
            download_info: LibraryDownload {
                sha1: "".to_string(),
                size: 0,
                url: format!("{url}{path}"),
                path,
            },
            is_native_library: false,
        });
    }
    result
}

/// Check if all the rules in Rule[] are acceptable in certain OS platform and features.
fn check_allowed(rules: Vec<Value>, platform: &PlatformInfo) -> bool {
    // by default it's allowed
    if rules.is_empty() {
        return true;
    }
    // else it's disallow by default
    let mut allow = false;
    for rule in rules {
        let action = rule["action"].as_str().unwrap() == "allow";
        let os = rule["os"].clone();
        if !os.is_object() {
            allow = action;
            continue;
        }
        if !os["name"].is_string() {
            allow = action;
            continue;
        }
        if platform.name != os["name"].as_str().unwrap() {
            continue;
        }
        if os["features"].is_object() {
            return false;
        }
        if !os["version"].is_string() {
            allow = action;
            continue;
        }
        let version = os["version"].as_str().unwrap();
        if Regex::is_match(
            &Regex::new(version).unwrap(),
            (&platform.version.to_string()).as_ref(),
        ) {
            allow = action;
        }
        // todo: check `features`
    }
    allow
}

pub struct LibraryInfo {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub is_snapshot: bool,

    /// The file extension. Default is `jar`. Some files in forge are `zip`.
    pub r#type: String,

    /// The classifier. Normally, this is empty. For forge, it can be like `universal`, `installer`.
    pub classifier: String,

    /// The maven path.
    pub path: String,

    /// The original maven name of this library
    pub name: String,
}

impl LibraryInfo {
    // /// Resolve the library info from the maven path.
    // ///
    // pub fn forge_maven_path(path: String) {}

    /// Get the base info of the library from its name
    /// * `lib` - The name of library of the library itself
    pub fn from_value(lib: &Value) -> Self {
        let name = lib["name"].as_str().unwrap().to_string();
        let split_name = name.split("@").collect::<Vec<&str>>();
        let body = split_name.get(0).unwrap().split(":").collect::<Vec<&str>>();
        let r#type = split_name.get(1).unwrap_or(&"jar").to_string();
        let group_id = body.get(0).unwrap().to_string();
        let artifact_id = body.get(1).unwrap().to_string();
        let version = body.get(2).unwrap().to_string();
        let is_snapshot = version.ends_with("SNAPSHOT");
        let group_path = group_id.replace(".", "/");
        let base = format!("{group_path}/{artifact_id}/{version}/{artifact_id}-{version}");
        let classifier = match body.get(3) {
            Some(classifier) => format!("{base}-{classifier}"),
            None => "".to_string(),
        };
        let path = format!("{base}.{type}");
        Self {
            group_id,
            artifact_id,
            version,
            is_snapshot,
            r#type,
            classifier,
            path,
            name,
        }
    }
}
