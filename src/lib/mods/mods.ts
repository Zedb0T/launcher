import { platform } from "@tauri-apps/api/os";
// temporarily use local file
import mods from "$assets/localmodtest/mods.json";
import type { SupportedGame } from "$lib/constants";

export interface GameMod {
  internalName: string;
  displayName : string;
  version: string;
  url: string;
  description: string;
  releaseDate: string;
  contributors: string[] | null;
  tags: string[] | null;
  websiteUrl: string | null;
  modImage: string | null;
  backgroundVideo: string | null;
}

export interface CurrentSelectedMod {
    currentModInternalName: string;
    currentModDisplayName: string;
    currentModDescription: string;
    currentVersion: string;
    currentModURL: string;
    currentModReleaseDate: string;
    currentContributors: string[] | "";
    currentModTags: string[] | "";
    currentModWebsiteUrl: string;
    currentModBackgroundVideo: string;
    currentModImage: string;
  }

export interface GameMods {
  [key: string]: GameMod;
}

export interface Game {
  jak1: GameMods;
  jak2: GameMods;
  jak3: GameMods;
  jakx: GameMods | null;
}

export interface ModRepositoryFile {
  games: Game;
}

export async function loadModsLocal(): Promise<ModRepositoryFile> {
  const file: ModRepositoryFile = JSON.parse(JSON.stringify(mods));
  return file;
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

    githubReleases = mods.ModList1.Jak1;
   
    //githubReleases = data.ModList1.Jak1[0]
    //console.log(typeof githubReleases);
   

    for (const obj of githubReleases) {
      for (const release of Object.values(obj)) {
        releases.push({
          name: release.tag_name,
          releaseType: "unofficial",
          version: release.tag_name,
          date: release.published_at,
          githubLink: release.html_url,
          downloadUrl: release.URL,
          //downloadUrl: await getDownloadLinkForCurrentPlatform(release),
          isDownloaded: false,
          pendingAction: false,
        });
        console.log(release.name)
      }
    }

    return releases.sort((a, b) => a.version.localeCompare(b.version));

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
  let githubRelease = await resp.json();
  githubRelease = mods.ModList1.Jak1;
  return {
    releaseType: "unofficial",
    name: githubRelease.name,
    version: githubRelease.tag_name,
    date: githubRelease.published_at,
    githubLink: githubRelease.html_url,
    downloadUrl: "http://www.google.com",
    //downloadUrl: await getDownloadLinkForCurrentPlatform(release),
    isDownloaded: false,
    pendingAction: false,
  };
}
