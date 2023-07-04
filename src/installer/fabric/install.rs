use tokio::fs;

use crate::utils::folder::MinecraftLocation;

use super::*;

/// 根据 yarn 和 loader 生成 fabric 版本的 JSON 文件到磁盘中。
pub async fn install_fabric(
    loader: FabricLoaderArtifact,
    minecraft_location: MinecraftLocation,
    options: Option<FabricInstallOptions>,
) -> String {
    let options = match options {
        None => FabricInstallOptions {
            inherits_from: None,
            version_id: None,
            size: None,
            yarn_version: None
        },
        Some(options) => options
    };
    let yarn: Option<String>;
    let side = options.size.unwrap_or(FabricInstallSide::Client);
    let mut id = options.version_id;
    let mut minecraft_version = "".to_string();

    match options.yarn_version {
        Some(yarn_version) => match yarn_version {
            YarnVersion::String(yarn_version) => {
                yarn = Some(yarn_version);
            }
            YarnVersion::FabricArtifactVersion(yarn_version) => {
                yarn = Some(yarn_version.version);
            }
        },
        None => {
            yarn = None;
            minecraft_version = loader.intermediary.version;
        }
    }
    if let None = id {
        if let Some(_) = yarn.clone() {
            id = Some(format!(
                "{}-loader{}",
                minecraft_version, loader.loader.version
            ));
        } else {
            id = Some(format!(
                "{}-fabric{}",
                minecraft_version, loader.loader.version
            ))
        }
    }
    let mut libraries = vec![
        LauncherMetaLibrariesItems {
            name: Some(loader.loader.maven.clone()),
            url: Some(String::from("https://maven.fabricmc.net/")),
        },
        LauncherMetaLibrariesItems {
            name: Some(loader.intermediary.maven.clone()),
            url: Some(String::from("https://maven.fabricmc.net/")),
        },
    ];
    if let Some(yarn) = yarn.clone() {
        libraries.push(LauncherMetaLibrariesItems {
            name: Some(format!("net.fabricmc:yarn:{}", yarn)),
            url: Some(String::from("https://maven.fabricmc.net/")),
        });
    }
    libraries.extend(loader.launcher_meta.libraries.common.iter().cloned());
    match side {
        FabricInstallSide::Client => {
            libraries.extend(loader.launcher_meta.libraries.client.iter().cloned())
        }
        FabricInstallSide::Server => {
            libraries.extend(loader.launcher_meta.libraries.server.iter().cloned())
        }
    }
    let main_class = match side {
        FabricInstallSide::Client => loader.launcher_meta.main_class["client"]
            .as_str()
            .unwrap_or(loader.launcher_meta.main_class.as_str().unwrap_or(""))
            .to_string(),
        FabricInstallSide::Server => loader.launcher_meta.main_class["server"]
            .as_str()
            .unwrap_or(loader.launcher_meta.main_class.as_str().unwrap_or(""))
            .to_string(),
    };
    let inherits_from = options.inherits_from.unwrap_or(minecraft_version);

    let json_file_path = minecraft_location.get_version_json(&id.clone().unwrap());
    fs::create_dir_all(json_file_path.parent().unwrap()).await.unwrap();
    if let Ok(metadata) = fs::metadata(&json_file_path).await {
        if metadata.is_file() {
            fs::remove_file(&json_file_path).await.unwrap();
        } else {
            fs::remove_dir_all(&json_file_path).await.unwrap();
        }
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct FabricVersionJSON {
        id: String,
        inherits_from: String,
        main_class: String,
        libraries: String,
        arguments: FabricVersionJSONArg,
        release_time: String,
        time: String,
    }
    #[derive(Serialize)]
    struct FabricVersionJSONArg {
        game: Vec<i32>,
        jvm: Vec<i32>,
    }
    let version_json = FabricVersionJSON {
        id: id.clone().unwrap_or("".to_string()),
        inherits_from,
        main_class,
        libraries: serde_json::to_string(&libraries).unwrap_or("".to_string()),
        arguments: FabricVersionJSONArg {
            game: vec![],
            jvm: vec![],
        },
        release_time: "2023-05-13T15:58:54.493Z".to_string(),
        time: "2023-05-13T15:58:54.493Z".to_string(),
    };
    let json_data = serde_json::to_string_pretty(&version_json)
        .unwrap_or("".to_string())
        .to_string();
    tokio::fs::write(json_file_path, json_data).await.unwrap();

    id.unwrap_or("".to_string())
}

#[tokio::test]
async fn test() {
    let artifact = super::version_list::get_fabric_loader_artifact("1.19.4", "0.1.0.48").await;
    let location = MinecraftLocation::new("test");
    install_fabric(artifact, location, None).await;
}