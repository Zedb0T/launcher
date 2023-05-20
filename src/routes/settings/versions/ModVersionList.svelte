<script lang="ts">
  import { loadModsLocal, type CurrentSelectedMod, type GameMod, type ModRepositoryFile } from "$lib/mods/mods";
  import { onMount } from "svelte";
  import {
    downloadModVersion, update_cache_if_need,
  } from "$lib/rpc/versions";
  
  let modLists: ModRepositoryFile[] = [];

  let currentMod: CurrentSelectedMod = {
    currentModInternalName: '',
    currentModDisplayName: '',
    currentModDescription: '',
    currentVersion: '',
    currentModURL: '',
    currentModReleaseDate: '',
    currentContributors: '',
    currentModTags: '',
    currentModWebsiteUrl: '',
    currentModBackgroundVideo: '',
    currentModImage: ''
  };

  onMount(async () => {
    const testModFile = await loadModsLocal();
    modLists = [...modLists, testModFile];
    //console.log(modLists);
  });

  function updateModFields(modData: GameMod): void {
    currentMod.currentModInternalName = modData.internalName || '';
    currentMod.currentModDisplayName = modData.displayName || '';
    currentMod.currentModDescription = modData.description || '';
    currentMod.currentVersion = modData.version || '';
    currentMod.currentModURL = modData.url || '';
    currentMod.currentModReleaseDate = modData.releaseDate || '';
    currentMod.currentContributors = modData.contributors || '';
    currentMod.currentModTags = modData.tags || '';
    currentMod.currentModWebsiteUrl = modData.websiteUrl || '';
    currentMod.currentModBackgroundVideo = modData.backgroundVideo || '';
    currentMod.currentModImage = modData.modImage || '';    
  }
  
  //This is for the drop-down box
  let selectedModName: string;

  function handleOptionSelected(event) {
    selectedModName = event.target.value;
    // Call your desired function or perform actions based on the selected option
    //console.log('Selected option:', selectedModName);
    // Call the updateModFields function
    updateModFields(getModData(selectedModName));
  }

  function getModData(modName: string): GameMod | undefined {
    // Iterate through the modLists array to find the mod data
    for (const modFile of modLists) {
      // Check if the selected mod name exists in the modFile's games
      if (modFile.games && modFile.games.jak1 && modFile.games.jak1[modName]) {
        return modFile.games.jak1[modName];
      }
      if (modFile.games && modFile.games.jak2 && modFile.games.jak2[modName]) {
        return modFile.games.jak2[modName];
      }
      if (modFile.games && modFile.games.jak3 && modFile.games.jak3[modName]) {
        return modFile.games.jak3[modName];
      }
    }

    // Return undefined if mod data is not found
    return undefined;
  }

  async function onDownloadModVersion(event: any) {

    await downloadModVersion(
      "v0.1.26",
      "https://github.com/open-goal/jak-project/releases/download/v0.1.26/opengoal-windows-v0.1.26.zip"
    );

  }

  async function onCheckFileCache(event: any) {

await update_cache_if_need(
  "C:\\Users\\NinjaPC\\Downloads\\New Folder\\versions\\mods\\v0.1.26\\extractor.exe"
);

}

</script>

<div class="flex flex-col gap-2 mt-2">
  <div>
    <!-- placeholders function does not work -->
    <button class="custom-button" on:click={onDownloadModVersion}>Install ZIP</button>
    <button class="custom-button" on:click={onCheckFileCache}>Reinstall ZIP</button>
    <button class="custom-button" on:click={handleOptionSelected}>Uninstall ZIP</button>
    

    {#each modLists as modFile}
      <div class="mod-category">
        <p>Jak 1 Mods - {Object.keys(modFile.games.jak1).length}</p>
        <select class="custom-select" id="modSelect" on:change={handleOptionSelected}>
          {#each Object.keys(modFile.games.jak1) as modName}
            <option value={modName}>{getModData(modName).displayName}</option>
          {/each}
        </select>
      </div>
    {/each}

    <img src={currentMod.currentModImage} alt="" width="300" height="300">
    <br>
    <p>Display Name: {currentMod.currentModDisplayName}</p>
    <br>
    <p>Internal Name: {currentMod.currentModInternalName}</p>
    <br>
    <p>Description: {currentMod.currentModDescription}</p>
    <br>
    <p>Version: {currentMod.currentVersion}</p>
    <br>
    <p>URL: {currentMod.currentModURL}</p>
    <br>
    <p>Release Date: {currentMod.currentModReleaseDate}</p>
    <br>
    <p>Contributors: {currentMod.currentContributors}</p>
    <br>
    <p>Tags: {currentMod.currentModTags}</p>
    <br>
    <p>Website URL: {currentMod.currentModWebsiteUrl}</p>
    <br>
    <p>Background Video: {currentMod.currentModBackgroundVideo}</p>
    <br>
    
    {#each modLists as modFile}
      <div class="mod-category">
        <p>Jak 2 Mods - {Object.keys(modFile.games.jak2).length}</p>
        <select on:change={handleOptionSelected}>
          {#each Object.keys(modFile.games.jak2) as modName}
            <option value={modName}>{modName}</option>
          {/each}
        </select>
      </div>
    {/each}

    {#each modLists as modFile}
      <div class="mod-category">
        <p>Jak 3 Mods - {Object.keys(modFile.games.jak3).length}</p>
        <select on:change={handleOptionSelected}>
          {#each Object.keys(modFile.games.jak3) as modName}
            <option value={modName}>{modName}</option>
          {/each}
        </select>
      </div>
    {/each}
  </div>
</div>

<style>
  select {
    color: black;
  }

  .mod-category {
    margin-bottom: 10px;
  }

  .custom-select {
    display: inline-block;
    padding: 0.5rem 1rem;
    font-size: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: #fff;
    color: #333;
    cursor: pointer;}
    .custom-button {
  display: inline-block;
  padding: 0.5rem 1rem;
  font-size: 1rem;
  border: none;
  border-radius: 4px;
  background-color: #007bff;
  color: #fff;
  cursor: pointer;
}

.custom-button:hover {
  background-color: #0056b3;
}

.custom-button:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.5);
}

.custom-button:active {
  background-color: #0056b3;
}

  
</style>
