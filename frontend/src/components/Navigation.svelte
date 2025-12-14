<script lang="ts">
  import { Save, Settings } from 'lucide-svelte';

  export let onSave: () => void;

  let autoSavePending = 0;
</script>

<nav
  class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between"
>
  <div class="flex items-center gap-3">
    <div
      class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center"
    >
      <span class="text-white font-bold text-sm">A</span>
    </div>
    <h1 class="text-xl font-bold text-gray-900">AURORA</h1>
    <span class="text-xs text-gray-500">Architecture Tooling v1.0</span>
  </div>

  <div class="flex items-center gap-4">
    {#if autoSavePending > 0}
      <div class="text-xs text-amber-600 flex items-center gap-1">
        <div class="w-1.5 h-1.5 bg-amber-600 rounded-full animate-pulse"></div>
        Saving ({autoSavePending})
      </div>
    {:else}
      <div class="text-xs text-green-600 flex items-center gap-1">
        <div class="w-1.5 h-1.5 bg-green-600 rounded-full"></div>
        All saved
      </div>
    {/if}

    <button
      on:click={onSave}
      class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
    >
      <Save size={16} />
      Save Project
    </button>

    <button class="p-2 hover:bg-gray-100 rounded-lg transition">
      <Settings size={20} class="text-gray-600" />
    </button>
  </div>
</nav>
