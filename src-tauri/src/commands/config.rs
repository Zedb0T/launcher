use std::path::Path;

use crate::util::os::get_installed_vcc_runtime;
use crate::{config::LauncherConfig, util::file::delete_dir};
use semver::Version;
use sysinfo::Disks;
use tauri::Manager;

use super::CommandError;

#[tauri::command]
pub async fn has_old_data_directory(app_handle: tauri::AppHandle) -> Result<bool, CommandError> {
  match &app_handle.path_resolver().app_config_dir() {
    None => Ok(false),
    Some(dir) => Ok(dir.join("data").join("iso_data").exists()),
  }
}

#[tauri::command]
pub async fn delete_old_data_directory(app_handle: tauri::AppHandle) -> Result<(), CommandError> {
  match &app_handle.path_resolver().app_config_dir() {
    None => Ok(()),
    Some(dir) => Ok(delete_dir(&dir.join("data"))?),
  }
}

#[tauri::command]
pub async fn reset_to_defaults(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock.reset_to_defaults().map_err(|_| {
    CommandError::Configuration("Unable to reset configuration to defaults".to_owned())
  })?;
  Ok(())
}

#[tauri::command]
pub async fn get_install_directory(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<Option<String>, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.installation_dir {
    None => Ok(None),
    Some(dir) => Ok(Some(dir.to_string())),
  }
}

#[tauri::command]
pub async fn set_install_directory(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  new_dir: String,
) -> Result<Option<String>, CommandError> {
  let mut config_lock = config.lock().await;
  config_lock.set_install_directory(new_dir).map_err(|err| {
    log::error!("Unable to persist installation directory: {:?}", err);
    CommandError::Configuration("Unable to persist installation directory".to_owned())
  })
}

fn diskspace_threshold_for_fresh_install(game_name: &str) -> Result<u64, CommandError> {
  match game_name {
    "jak1" => Ok(4 * 1024 * 1024 * 1024),  // 4gb
    "jak2" => Ok(11 * 1024 * 1024 * 1024), // 11gb
    "jak3" => Ok(11 * 1024 * 1024 * 1024), // TODO! gb
    "jakx" => Ok(11 * 1024 * 1024 * 1024), // TODO! gb
    _ => Err(CommandError::UnknownGame(game_name.to_string())),
  }
}

#[tauri::command]
pub async fn is_diskspace_requirement_met(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<bool, CommandError> {
  // If the game is already installed, we assume they have enough drive space
  let mut config_lock = config.lock().await;
  if is_game_installed_impl(&mut config_lock, game_name.to_owned())? {
    return Ok(true);
  }
  if let Some(bypass) = config_lock.requirements.bypass_requirements {
    if bypass {
      log::warn!("Bypassing the Disk Space requirements check!");
      return Ok(true);
    }
  }

  let install_dir = match &config_lock.installation_dir {
    None => {
      log::error!("Can't check disk space, no install directory has been choosen!");
      return Err(CommandError::Configuration(
        "Can't check disk space, no install directory has been choosen!".to_owned(),
      ));
    }
    Some(dir) => Path::new(dir),
  };

  // Check the drive that the installation directory is set to
  let minimum_required_drive_space = diskspace_threshold_for_fresh_install(&game_name)?;
  for disk in Disks::new_with_refreshed_list().into_iter() {
    if install_dir.starts_with(disk.mount_point()) {
      if disk.available_space() < minimum_required_drive_space {
        log::warn!("Not enough space left on disk: {:?}", disk.name());
        return Ok(false);
      } else {
        return Ok(true);
      }
    }
  }

  log::error!("Unable to find relevant drive to check for space");
  return Err(CommandError::Configuration(
    "Unable to find relevant drive to check for space".to_owned(),
  ));
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn is_minimum_vcc_runtime_installed(
  _config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let minimum_version = semver::Version::new(14, 40, 33810);
  let installed_vcc_runtime_version = get_installed_vcc_runtime();
  if installed_vcc_runtime_version.is_none() {
    Err(CommandError::Configuration(
      "Unable to check if VCC runtime is installed".to_owned(),
    ))
  } else {
    Ok(installed_vcc_runtime_version.unwrap() >= minimum_version)
  }
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn is_minimum_vcc_runtime_installed(
  _config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  return Ok(false);
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn is_minimum_vcc_runtime_installed(
  _config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  return Ok(false);
}

#[tauri::command]
pub async fn is_avx_requirement_met(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  force: bool,
) -> Result<bool, CommandError> {
  let mut config_lock = config.lock().await;
  if force {
    config_lock.requirements.avx = None;
  }
  if let Some(bypass) = config_lock.requirements.bypass_requirements {
    if bypass {
      log::warn!("Bypassing the AVX requirements check!");
      return Ok(true);
    }
  }
  match config_lock.requirements.avx {
    None => {
      #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
      {
        if is_x86_feature_detected!("avx") || is_x86_feature_detected!("avx2") {
          config_lock.requirements.avx = Some(true);
        } else {
          config_lock.requirements.avx = Some(false);
        }
      }
      #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
      {
        // TODO - macOS check if on atleast sequoia and rosetta 2 is installed
        config_lock.requirements.avx = Some(false);
      }
      config_lock.save_config().map_err(|err| {
        log::error!("Unable to persist avx requirement change {}", err);
        CommandError::Configuration("Unable to persist avx requirement change".to_owned())
      })?;
      Ok(config_lock.requirements.avx.unwrap_or(false))
    }
    Some(val) => Ok(val),
  }
}

#[tauri::command]
pub async fn is_opengl_requirement_met(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: tauri::AppHandle,
  force: bool,
) -> Result<Option<bool>, CommandError> {
  let mut config_lock = config.lock().await;
  if force {
    config_lock.requirements.opengl = None;
  }
  if let Some(bypass) = config_lock.requirements.bypass_requirements {
    if bypass {
      log::warn!("Bypassing the OpenGL requirements check!");
      return Ok(Some(true));
    }
  }
  // If the value is already set, just return it
  if let Some(val) = config_lock.requirements.opengl {
    return Ok(Some(val));
  }

  // Check the active tooling version, if it's above 0.1.38 we can use the new
  // built-in gpu testing feature
  // If not, we have to use the spotty heuristic
  let active_version = config_lock
    .active_version
    .as_ref()
    .ok_or(CommandError::Configuration(
      "No active version set, can't perform operation".to_owned(),
    ))?;
  // If we can't determine the version, assume it's too old
  let tooling_version = Version::parse(active_version.strip_prefix('v').unwrap_or(&active_version))
    .unwrap_or(Version::new(0, 1, 37));
  if tooling_version.major == 0 && tooling_version.minor <= 1 && tooling_version.patch < 38 {
    // Assume it's fine
    log::warn!(
      "We no longer check for OpenGL support via heuristics, assuming they meet the requirement"
    );
    return Ok(Some(true));
  }
  // Do it the new way!
  log::info!("Checking for OpenGL support via `gk`");
  let test_result = crate::util::game_tests::run_game_gpu_test(&config_lock, &app_handle).await?;
  config_lock
    .set_opengl_requirement_met(Some(test_result.success))
    .map_err(|_| {
      CommandError::Configuration("Unable to persist opengl requirement change".to_owned())
    })?;
  Ok(Some(test_result.success))
}

#[tauri::command]
pub async fn finalize_installation(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: tauri::AppHandle,
  game_name: String,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .update_installed_game_version(&game_name, true)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist game installation status".to_owned())
    })?;
  app_handle.emit_all("gameInstalled", {})?;
  Ok(())
}

fn is_game_installed_impl(
  config_lock: &mut tokio::sync::MutexGuard<LauncherConfig>,
  game_name: String,
) -> Result<bool, CommandError> {
  // Check that the version and version folder config field is set properly as well
  let version = config_lock.game_install_version(&game_name);
  let version_folder = config_lock.game_install_version_folder(&game_name);

  if version.is_empty() || version_folder.is_empty() {
    config_lock
      .update_installed_game_version(&game_name, false)
      .map_err(|err| {
        log::error!(
          "Unable to mark partially installed game as uninstalled {}",
          err
        );
        CommandError::Configuration(
          "Unable to mark partially installed game as uninstalled".to_owned(),
        )
      })?;
    return Ok(false);
  }

  Ok(true)
}

#[tauri::command]
pub async fn is_game_installed(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<bool, CommandError> {
  let mut config_lock = config.lock().await;

  if !config_lock.is_game_installed(&game_name) {
    return Ok(false);
  }

  return is_game_installed_impl(&mut config_lock, game_name);
}

#[tauri::command]
pub async fn get_installed_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<String, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.game_install_version(&game_name))
}

#[tauri::command]
pub async fn get_installed_version_folder(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<String, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.game_install_version_folder(&game_name))
}

#[tauri::command]
pub async fn save_active_version_change(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version_folder: String,
  new_active_version: String,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_active_version_folder(version_folder)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist active version folder change".to_owned())
    })?;
  config_lock
    .set_active_version(new_active_version)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist active version change".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn get_active_tooling_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<Option<String>, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.active_version.clone())
}

#[tauri::command]
pub async fn get_active_tooling_version_folder(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<Option<String>, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.active_version_folder.clone())
}

#[tauri::command]
pub async fn get_locale(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<Option<String>, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.locale.clone())
}

#[tauri::command]
pub async fn set_locale(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  locale: String,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_locale(locale)
    .map_err(|_| CommandError::Configuration("Unable to persist locale change".to_owned()))?;
  Ok(())
}

#[tauri::command]
pub async fn get_bypass_requirements(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match config_lock.requirements.bypass_requirements {
    Some(val) => Ok(val),
    None => Ok(false),
  }
}

#[tauri::command]
pub async fn set_bypass_requirements(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  bypass: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock.set_bypass_requirements(bypass).map_err(|_| {
    CommandError::Configuration("Unable to persist bypass requirements change".to_owned())
  })?;
  Ok(())
}

#[tauri::command]
pub async fn get_check_for_latest_mod_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match config_lock.check_for_latest_mod_version {
    Some(val) => Ok(val),
    None => Ok(true), // default true
  }
}

#[tauri::command]
pub async fn set_check_for_latest_mod_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  check_for_latest_mod_version: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_check_for_latest_mod_version(check_for_latest_mod_version)
    .map_err(|_| {
      CommandError::Configuration("Unable to set check_for_latest_mod_version flag".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn get_enabled_texture_packs(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<Vec<String>, CommandError> {
  let config_lock = config.lock().await;
  Ok(config_lock.game_enabled_textured_packs(&game_name))
}

#[tauri::command]
pub async fn cleanup_enabled_texture_packs(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
  cleanup_list: Vec<String>,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .cleanup_game_enabled_texture_packs(&game_name, cleanup_list)
    .map_err(|_| {
      CommandError::Configuration("Unable to cleanup enabled texture packs".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn set_enabled_texture_packs(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
  packs: Vec<String>,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_game_enabled_texture_packs(&game_name, packs)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist change to enabled texture packs".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn does_active_tooling_version_support_game(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.active_version {
    Some(version) => {
      // If we can't determine the version, assume its our first release
      let tooling_version = Version::parse(version.strip_prefix('v').unwrap_or(&version))
        .unwrap_or(Version::new(0, 0, 1));
      match game_name.as_str() {
        "jak1" => Ok(true),
        "jak2" => Ok(tooling_version.minor > 1 || tooling_version.patch >= 44),
        _ => Ok(false),
      }
    }
    None => {
      log::warn!("No active tooling version set, can't check the game supports it!");
      Ok(false)
    }
  }
}

#[tauri::command]
pub async fn get_playtime(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: String,
) -> Result<u64, CommandError> {
  let mut config_lock = config.lock().await;
  match config_lock.get_game_seconds_played(&game_name) {
    Ok(playtime) => Ok(playtime),
    Err(err) => Err(CommandError::Configuration(format!(
      "Error occurred when getting game playtime: {}",
      err
    ))),
  }
}

#[tauri::command]
pub async fn does_active_tooling_version_meet_minimum(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  minimum_patch: u64,
  minimum_minor: u64,
  minimum_major: u64,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.active_version {
    Some(version) => {
      // If we can't determine the version, assume 0,0,0
      let tooling_version = Version::parse(version.strip_prefix('v').unwrap_or(&version))
        .unwrap_or(Version::new(0, 0, 0));
      let compare_version = Version::new(minimum_major, minimum_minor, minimum_patch);
      if tooling_version >= compare_version {
        Ok(true)
      } else {
        Ok(false)
      }
    }
    None => {
      log::warn!("No active tooling version set, can't check if the minimum!");
      Ok(false)
    }
  }
}

#[tauri::command]
pub async fn is_rip_levels_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.decompiler_settings {
    Some(settings) => Ok(settings.rip_levels_enabled.unwrap_or(false)),
    _ => Ok(false),
  }
}

#[tauri::command]
pub async fn set_rip_levels_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  enabled: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock.set_rip_levels_enabled(enabled).map_err(|_| {
    CommandError::Configuration("Unable to persist change to rip_levels_enabled".to_owned())
  })?;
  Ok(())
}

#[tauri::command]
pub async fn is_rip_collision_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.decompiler_settings {
    Some(settings) => Ok(settings.rip_collision_enabled.unwrap_or(false)),
    _ => Ok(false),
  }
}

#[tauri::command]
pub async fn set_rip_collision_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  enabled: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_rip_collision_enabled(enabled)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist change to rip_levels_enabled".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn is_rip_textures_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.decompiler_settings {
    Some(settings) => Ok(settings.rip_textures_enabled.unwrap_or(false)),
    _ => Ok(false),
  }
}

#[tauri::command]
pub async fn set_rip_textures_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  enabled: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock.set_rip_textures_enabled(enabled).map_err(|_| {
    CommandError::Configuration("Unable to persist change to rip_levels_enabled".to_owned())
  })?;
  Ok(())
}

#[tauri::command]
pub async fn is_rip_streamed_audio_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  match &config_lock.decompiler_settings {
    Some(settings) => Ok(settings.rip_streamed_audio_enabled.unwrap_or(false)),
    _ => Ok(false),
  }
}

#[tauri::command]
pub async fn set_rip_streamed_audio_enabled(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  enabled: bool,
) -> Result<(), CommandError> {
  let mut config_lock = config.lock().await;
  config_lock
    .set_rip_streamed_audio_enabled(enabled)
    .map_err(|_| {
      CommandError::Configuration("Unable to persist change to rip_levels_enabled".to_owned())
    })?;
  Ok(())
}

#[tauri::command]
pub async fn get_proceed_after_successful_operation(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<bool, CommandError> {
  let config_lock = config.lock().await;
  Ok(
    config_lock
      .proceed_after_successful_operation
      .unwrap_or(true),
  )
}
