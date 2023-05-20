import { platform } from "@tauri-apps/api/os";
import mods from "$assets/localmodtest/mods.json";

export interface ReleaseInfo {
  releaseType: "official" | "unofficial" | "devel";
  version: string;
  date: string | undefined;
  githubLink: string | undefined;
  downloadUrl: string | undefined;
  isDownloaded: boolean;
  pendingAction: boolean;
}

async function getDownloadLinkForCurrentPlatform(
  release
): Promise<string | undefined> {
  const platformName = await platform();
  for (const asset of release.assets) {
    if (platformName === "darwin" && asset.name.includes("opengoal-macos-v")) {
      return asset.browser_download_url;
    } else if (
      platformName === "win32" &&
      (asset.name.startsWith("opengoal-windows-v") ||
        (asset.name.startsWith("opengoal-v") && asset.name.includes("windows")))
    ) {
      return asset.browser_download_url;
    } else if (
      platformName === "linux" &&
      (asset.name.startsWith("opengoal-linux-v") ||
        (asset.name.startsWith("opengoal-v") && asset.name.includes("linux")))
    ) {
      return asset.browser_download_url;
    }
  }
  return undefined;
}

export async function listOfficialReleases(): Promise<ReleaseInfo[]> {
  let releases = [];
  // TODO - handle rate limiting
  // TODO - long term - handle pagination (more than 100 releases)
  // TODO - even longer term - extract this out into an API we control (avoid github rate limiting) -- will be needed for unofficial releases as well anyway
  const resp = await fetch(
    "https://api.github.com/repos/open-goal/jak-project/releases?per_page=100"
  );
  // TODO - handle error
  const githubReleases = await resp.json();

  for (const release of githubReleases) {
    releases.push({
      releaseType: "official",
      version: release.tag_name,
      date: release.published_at,
      githubLink: release.html_url,
      //downloadUrl: "http://www.google.com",
      downloadUrl: await getDownloadLinkForCurrentPlatform(release),
      isDownloaded: false,
      pendingAction: false,
    });
  }

  return releases.sort((a, b) => b.date.localeCompare(a.date));
}

export async function getLatestOfficialRelease(): Promise<ReleaseInfo> {
  // TODO - handle rate limiting
  // TODO - even longer term - extract this out into an API we control (avoid github rate limiting) -- will be needed for unofficial releases as well anyway
  const resp = await fetch(
    "https://api.github.com/repos/open-goal/jak-project/releases/latest"
  );
  // TODO - handle error
  const githubRelease = await resp.json();
  return {
    releaseType: "official",
    version: githubRelease.tag_name,
    date: githubRelease.published_at,
    githubLink: githubRelease.html_url,
    //downloadUrl: "http://www.google.com",
    downloadUrl: await getDownloadLinkForCurrentPlatform(githubRelease),
    isDownloaded: false,
    pendingAction: false,
  };
}

export async function listModReleases(): Promise<ReleaseInfo[]> {
  let releases = [];
  // TODO - handle rate limiting
  // TODO - long term - handle pagination (more than 100 releases)
  // TODO - even longer term - extract this out into an API we control (avoid github rate limiting) -- will be needed for unofficial releases as well anyway
  const response = await fetch('https://raw.githubusercontent.com/OpenGOAL-Unofficial-Mods/OpenGoal-ModLauncher-dev/main/resources/jak1_mods.json');
  const data = await response.json();


  
  //TODO - DO THIS ALL ON CLICK TO LIMIT API CALLS?
  const resp = await fetch(
    "https://api.github.com/repos/OpenGOAL-Unofficial-Mods/flutflut-legacy/releases?per_page=100"
  );
  // TODO - handle error
  let githubReleases = await resp.json()



  .catch(error => {
    // handle any errors that occur during the fetch operation
    console.error(error);
  });

    //TEMPLOCALTEST
    githubReleases = JSON.stringify(mods);
   
    console.log(typeof githubReleases);

  for (const release of githubReleases) {
    releases.push({
      releaseType: "official",
      version: release.tag_name,
      date: release.published_at,
      githubLink: release.html_url,
      downloadUrl: "http://www.google.com",
      //downloadUrl: await getDownloadLinkForCurrentPlatform(release),
      isDownloaded: false,
      pendingAction: false,
    });
  }

 // return releases.sort((a, b) => b.date.localeCompare(a.date));
 // Zed
 return releases
}

export async function getLatestModRelease(): Promise<ReleaseInfo> {
  // TODO - handle rate limiting
  // TODO - even longer term - extract this out into an API we control (avoid github rate limiting) -- will be needed for unofficial releases as well anyway
  //https://github.com/OpenGOAL-Unofficial-Mods/flutflut-legacy
  const resp = await fetch(
    "https://api.github.com/repos/OpenGOAL-Unofficial-Mods/flutflut-legacy/releases/latest"
  );
  // TODO - handle error
  const githubRelease = await resp.json();
  return {
    releaseType: "official",
    version: githubRelease.tag_name,
    date: githubRelease.published_at,
    githubLink: githubRelease.html_url,
    downloadUrl: "http://www.google.com",
    //downloadUrl: await getDownloadLinkForCurrentPlatform(release),
    isDownloaded: false,
    pendingAction: false,
  };
}
