<script lang="ts">
  import { Tabs } from "flowbite-svelte";
  import OfficialVersions from "./versions/OfficialVersions.svelte";
  import DevelVersions from "./versions/DevelVersions.svelte";
</script>

<div>
  <p class="text-sm mt-2 mb-2">Configure your active tooling version</p>
  <Tabs style="pill" contentClass="p-4 rounded-lg mt-0 pb-20 overflow-y-auto">
    <OfficialVersions />
    <DevelVersions />
  </Tabs>
</div>
