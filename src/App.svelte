<script>
  // Other Imports
  import { onMount } from "svelte";
  import { Router, Route } from "svelte-navigator";
  import Game from "./routes/Game.svelte";
  import Settings from "./routes/Settings.svelte";
  import Sidebar from "./components/sidebar/Sidebar.svelte";
  import Background from "./components/background/Background.svelte";
  import Header from "./components/header/Header.svelte";
  import Update from "./routes/Update.svelte";
  import { isInDebugMode } from "$lib/utils/common";
  import GameInProgress from "./components/games/GameInProgress.svelte";
  import Toast from "./components/toast/Toast.svelte";
  import Help from "./routes/Help.svelte";
  import { isLoading } from "svelte-i18n";
  import { getLocale, setLocale } from "$lib/rpc/config";
  import GameFeature from "./routes/GameFeature.svelte";

  let revokeSpecificActions = false;

  // Events
  onMount(async () => {
    // Temporary fix related to https://github.com/open-goal/launcher/issues/110
    // NOTE - this doesn't feel required anymore after i fixed the window switching
    // but let's keep it for now because im paranoid about the issue cropping up again...
    if (window.sessionStorage.getItem("refreshHack") !== "true") {
      location.reload();
      window.sessionStorage.setItem("refreshHack", "true");
    }
    // Set locale from settings
    const locale = await getLocale();
    if (locale !== null) {
      setLocale(locale);
    }
  });

  if (!isInDebugMode()) {
    revokeSpecificActions = true;
    // Disable Right Click
    document.addEventListener("contextmenu", (event) => {
      if (revokeSpecificActions) {
        event.preventDefault();
      }
    });
    // Disable Refreshing (F5 / Ctrl+R)
    document.addEventListener("keydown", (e) => {
      if (e.code == "F5") {
        if (revokeSpecificActions) {
          e.preventDefault();
        }
      }
      if (e.code == "KeyR" && e.ctrlKey) {
        if (revokeSpecificActions) {
          e.preventDefault();
        }
      }
      // super secret keybind to reverse the above so we can debug a release version
      // Shift+Ctrl F12
      if (e.code == "F12" && e.ctrlKey && e.shiftKey) {
        revokeSpecificActions = false;
      }
    });
  }
</script>

<Router>
  <div class={`container h-screen max-w-none flex flex-col bg-black`}>
    {#if !$isLoading}
      <Background />
      <Header />
      <div class="flex flex-row grow shrink h-[90%] z-10">
        <Sidebar />
        <div id="content" class="overflow-y-auto grow shrink">
          <Route path="/" component={Game} primary={false} />
          <Route path="/:game_name" component={Game} primary={false} />
          <Route
            path="/:game_name/features/:feature"
            component={GameFeature}
            primary={false}
          />
          <Route
            path="/:game_name/features/mods/:source_url/:mod_name"
            component={Game}
            primary={false}
          />
          <Route path="/jak3" component={GameInProgress} primary={false} />
          <Route path="/settings/:tab" component={Settings} primary={false} />
          <Route path="/faq" component={Help} primary={false} />
          <Route path="/update" component={Update} primary={false} />
        </div>
      </div>
      <Toast />
    {/if}
  </div>
</Router>
