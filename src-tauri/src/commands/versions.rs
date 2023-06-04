use std::path::Path;

use log::info;

use crate::{
  config::LauncherConfig,
  util::{
    file::{create_dir, delete_dir},
    network::download_file,
    os::open_dir_in_os,
    tar::extract_and_delete_tar_ball,
    zip::extract_and_delete_zip_file,
  },
};

use super::CommandError;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[tauri::command]
pub async fn list_downloaded_versions(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version_folder: String,
) -> Result<Vec<String>, CommandError> {
  let config_lock = config.lock().await;
  let install_path = match &config_lock.installation_dir {
    None => return Ok(Vec::new()),
    Some(path) => Path::new(path),
  };

  let expected_path = Path::new(install_path)
    .join("versions")
    .join(version_folder);
  if !expected_path.exists() || !expected_path.is_dir() {
    log::info!(
      "No {} folder found, returning no releases",
      expected_path.display()
    );
    return Ok(Vec::new());
  }

  let entries = std::fs::read_dir(&expected_path).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to read versions from {}",
      expected_path.display()
    ))
  })?;
  Ok(
    entries
      .filter_map(|e| {
        e.ok().and_then(|d| {
          let p = d.path();
          if p.is_dir() {
            Some(
              p.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or("".into()),
            )
          } else {
            None
          }
        })
      })
      .collect(),
  )
}

#[tauri::command]
pub async fn download_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
  version_folder: String,
  url: String,
) -> Result<(), CommandError> {
  let config_lock = config.lock().await;
  let install_path = match &config_lock.installation_dir {
    None => {
      return Err(CommandError::VersionManagement(format!(
        "Cannot install version, no installation directory set"
      )))
    }
    Some(path) => Path::new(path),
  };

  let dest_dir = install_path
    .join("versions")
    .join(&version_folder)
    .join(&version);

  // Delete the directory if it exists, and create it from scratch
  delete_dir(&dest_dir).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to prepare destination folder '{}' for download",
      dest_dir.display()
    ))
  })?;
  create_dir(&dest_dir).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to prepare destination folder '{}' for download",
      dest_dir.display()
    ))
  })?;

  if cfg!(windows) {
    let download_path = install_path
      .join("versions")
      .join(version_folder)
      .join(format!("{}.zip", version));

    // Download the file
    download_file(&url, &download_path).await.map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully download version"))
    })?;

    // Extract the zip file
    extract_and_delete_zip_file(&download_path, &dest_dir).map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully extract downloaded version"))
    })?;
    return Ok(());
  } else if cfg!(unix) {
    let download_path = install_path
      .join("versions")
      .join(version_folder)
      .join(format!("{}.tar.gz", version));

    // Download the file
    download_file(&url, &download_path).await.map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully download version"))
    })?;

    // Extract the zip file
    extract_and_delete_tar_ball(&download_path, &dest_dir).map_err(|err| {
      log::error!("unable to extract and delete version tar.gz file {}", err);
      CommandError::VersionManagement(format!("Unable to successfully extract downloaded version"))
    })?;
    return Ok(());
  }
  Err(CommandError::VersionManagement(format!(
    "Unknown operating system, unable to download and extract correct release"
  )))
}


use std::fs::File;
use std::io::Read;
use serde_json::{self, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ModFile {
  // Define the structure of ModFile based on your JSON structure
  // Modify it according to your JSON schema
  games: Games,
  // Other fields...
}

#[derive(Debug, Deserialize)]
struct Games {
  jak1: Option<Vec<CurrentSelectedMod>>,
  jak2: Option<Vec<CurrentSelectedMod>>,
  jak3: Option<Vec<CurrentSelectedMod>>,
}

//Mod stuff
#[derive(Debug, Deserialize, Serialize,Clone)]
struct CurrentSelectedMod {
  currentModInternalName: String,
  currentModDisplayName: String,
  currentModDescription: String,
  currentVersion: String,
  currentModURL: String,
  currentModReleaseDate: String,
  currentContributors: String,
  currentModTags: String,
  currentModWebsiteUrl: String,
  currentModBackgroundVideo: String,
  currentModImage: String,
}






use std::io::Write;



#[derive(Debug, Deserialize, Serialize)]
struct Game {
    internalName: String,
    url: String,
    contributors: Vec<String>,
    description: String,
    displayName: String,
    releaseDate: String,
    tags: Vec<String>,
    websiteUrl: String,
    modImage: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameData {
    games: Value,
}
use reqwest::blocking::Client;


#[tauri::command]
pub fn get_or_check_cache(game_name: &str, mod_name: &str) {
    // Specify the path of the input JSON file
    let data_file_path = "C:\\Users\\NinjaPC\\Documents\\Github\\launcher\\src\\assets\\localmodtest\\mods.json";

    // Define the directory where the cache file will be created
    let cache_directory = format!("C:\\Users\\NinjaPC\\Downloads\\New Folder\\versions\\mods\\cache\\{}", mod_name);

    // Read the JSON file
    let file = File::open(data_file_path).expect("Failed to open file");
    let data: GameData = serde_json::from_reader(file).expect("Failed to parse JSON");

    // Check if the cache directory exists, create it if it doesn't
    if !Path::new(&cache_directory).exists() {
        fs::create_dir_all(&cache_directory).expect("Failed to create cache directory");
        println!("Cache directory created: {}", cache_directory);
    }

    // Find the game data by name
    if let Value::Object(games) = data.games {
        if let Some(game_data) = games.get(game_name) {
            if let Some(mod_data) = game_data.get(mod_name) {
                // Check if the cache file exists
                let cache_file_path = format!("{}/{}_{}_cache.json", &cache_directory, game_name, mod_name);
                let cache_file_exists = fs::metadata(&cache_file_path).is_ok();

                if cache_file_exists {
                    // Check if the cache file is older than 24 hours
                    let cache_file_modified = fs::metadata(&cache_file_path)
                        .expect("Failed to get cache file metadata")
                        .modified()
                        .expect("Failed to get cache file modified time");

                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Failed to get current time")
                        .as_secs();

                    let cache_file_modified_time = cache_file_modified
                        .duration_since(UNIX_EPOCH)
                        .expect("Failed to get cache file modified time")
                        .as_secs();

                    let twenty_four_hours = 24 * 60 * 60;
                    if current_time - cache_file_modified_time >= twenty_four_hours {
                        // Cache file is older than 24 hours, delete it
                        fs::remove_file(&cache_file_path).expect("Failed to delete cache file");
                        println!("Old cache file deleted: {}", cache_file_path);
                    } else {
                        // Cache file is up to date, no need to regenerate
                        println!("Cache file is up to date: {}", cache_file_path);
                        return;
                    }
                }

                // Download the mod image file
                if let Some(mod_image) = mod_data.get("modImage") {
                  if let Some(mod_image_url) = mod_image.as_str() {
                      let client = Client::new();
                      let response = client.get(mod_image_url).send();
              
                      if let Ok(mut file) = File::create(cache_directory.clone() + "\\modImage.png") {
                          if let Ok(body) = response {
                              let bytes = body.bytes().unwrap();
                              let bytes_slice: &[u8] = &bytes;
              
                              file.write_all(bytes_slice).expect("Failed to write mod image to file");
                          }
                      }
                  }
              }

                // Convert the mod data into a JSON string
                let mod_json = serde_json::to_string_pretty(&mod_data).expect("Failed to convert to JSON");

                // Create a new JSON file with the mod data
                let mut new_file = File::create(&cache_file_path).expect("Failed to create file");
                write!(new_file, "{}", mod_json).expect("Failed to write to file");

                println!("Cache file generated: {}", cache_file_path);
                return;
            }
        }
    }

    // Game data or mod data not found for the specified names
    println!("Game data or mod data not found for game: {}, mod: {}", game_name, mod_name);
}

use chrono::{DateTime, Utc};



use std::io::{self, prelude::*};
use serde_json::{json};

fn generate_api_url(repo_url: &str) -> String {
  let repo_path = repo_url.trim_end_matches(".git");
  let api_url = format!("{}/releases?per_page=100", repo_path.replace("https://github.com/", "https://api.github.com/repos/"));
  api_url
}
use base64::encode;

#[derive(Debug, Serialize)] // Implement Serialize trait for the Result type
pub struct InvokeError {
    message: String,
}

impl std::convert::From<std::io::Error> for InvokeError {
    fn from(error: std::io::Error) -> Self {
        Self {
            message: format!("I/O error: {}", error),
        }
    }
}

#[tauri::command]
pub fn get_image_file(path: &str) -> Result<String, InvokeError> {
    let mut file = std::fs::File::open(path)?;
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data)?;
    let encoded_data = encode(&image_data);
    Ok(encoded_data)
}

#[tauri::command]
pub fn check_and_create_json(path: &str) -> io::Result<()> {
    let json_exists = std::path::Path::new(path).exists();

    if !json_exists {
        let jak_ratchet_json = json!({
            "Jak": "",
            "ratchet": {
                "clank": ""
            }
        });

        let json_string = jak_ratchet_json.to_string();

        let mut file = File::create(path)?;
        file.write_all(json_string.as_bytes())?;
    }

    Ok(())
}



#[tauri::command]
pub async fn download_mod_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
  version_folder: String,
  url: String,
) -> Result<(), CommandError> {
  let config_lock = config.lock().await;
  let install_path = match &config_lock.installation_dir {
    None => {
      return Err(CommandError::VersionManagement(format!(
        "Cannot install version, no installation directory set"
      )))
    }
    Some(path) => Path::new(path),
  };

  let dest_dir = install_path
    .join("versions")
    .join(&version_folder)
    .join(&version);

  // Delete the directory if it exists, and create it from scratch
  delete_dir(&dest_dir).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to prepare destination folder '{}' for download",
      dest_dir.display()
    ))
  })?;
  create_dir(&dest_dir).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to prepare destination folder '{}' for download",
      dest_dir.display()
    ))
  })?;

  if cfg!(windows) {
    let download_path = install_path
      .join("versions")
      .join(version_folder)
      .join(format!("{}.zip", version));

    // Download the file
    download_file(&url, &download_path).await.map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully download version"))
    })?;

    // Extract the zip file
    extract_and_delete_zip_file(&download_path, &dest_dir).map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully extract downloaded version"))
    })?;
    return Ok(());
  } else if cfg!(unix) {
    let download_path = install_path
      .join("versions")
      .join(version_folder)
      .join(format!("{}.tar.gz", version));

    // Download the file
    download_file(&url, &download_path).await.map_err(|_| {
      CommandError::VersionManagement(format!("Unable to successfully download version"))
    })?;

    // Extract the zip file
    extract_and_delete_tar_ball(&download_path, &dest_dir).map_err(|err| {
      log::error!("unable to extract and delete version tar.gz file {}", err);
      CommandError::VersionManagement(format!("Unable to successfully extract downloaded version"))
    })?;
    return Ok(());
  }
  Err(CommandError::VersionManagement(format!(
    "Unknown operating system, unable to download and extract correct release"
  )))
}

#[tauri::command]
pub async fn remove_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
  version_folder: String,
) -> Result<(), CommandError> {
  let config_lock = config.lock().await;
  let install_path = match &config_lock.installation_dir {
    None => {
      return Err(CommandError::VersionManagement(format!(
        "Cannot install version, no installation directory set"
      )))
    }
    Some(path) => Path::new(path),
  };

  info!("Deleting Version {}:{}", version_folder, version);

  let version_dir = install_path
    .join("versions")
    .join(&version_folder)
    .join(&version);

  delete_dir(&version_dir)?;

  Ok(())
}

#[tauri::command]
pub async fn go_to_version_folder(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version_folder: String,
) -> Result<(), CommandError> {
  let config_lock = config.lock().await;
  let install_path = match &config_lock.installation_dir {
    None => {
      return Err(CommandError::VersionManagement(format!(
        "Cannot go to version folder, no installation directory set"
      )))
    }
    Some(path) => Path::new(path),
  };

  let folder_path = Path::new(install_path)
    .join("versions")
    .join(version_folder);
  create_dir(&folder_path).map_err(|_| {
    CommandError::VersionManagement(format!(
      "Unable to go to create version folder '{}' in order to open it",
      folder_path.display()
    ))
  })?;

  open_dir_in_os(folder_path.to_string_lossy().into_owned())
    .map_err(|_| CommandError::VersionManagement(format!("Unable to go to open folder in OS")))?;
  Ok(())
}
