<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api';
  import TestPage from './components/TestPage.svelte';

  let appReady = false;
  let error: string | null = null;
  let message = 'Initializing...';
  let showTestPage = false;

  onMount(async () => {
    try {
      message = 'Testing Tauri API...';

      // Simple test to see if the app even loads
      const response = await invoke('query_get_statistics', {});
      message = `Connected! Stats: ${JSON.stringify(response)}`;
      appReady = true;
      showTestPage = true; // Show test page on startup
    } catch (err) {
      error = `Init error: ${String(err)}`;
      console.error('Init error:', err);
    }
  });
</script>

{#if showTestPage}
  <TestPage />
{:else}
  <div class="flex flex-col w-full h-screen bg-white p-8">
    <h1 class="text-4xl font-bold text-blue-900 mb-4">AURORA</h1>

    {#if error}
      <div
        class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
      >
        <strong>Error:</strong>
        <span class="block wrap-break-word mt-2">{error}</span>
      </div>
    {/if}

    <div class="bg-gray-100 border border-gray-300 rounded p-4 mb-4">
      <p class="text-sm font-mono text-gray-800 wrap-break-word">{message}</p>
    </div>

    {#if appReady}
      <div class="text-green-600 font-semibold mb-4">✓ Application Ready</div>
      <p class="text-gray-700">The Tauri backend is responding correctly.</p>
    {:else}
      <div class="text-blue-600 font-semibold mb-4">⏳ Loading...</div>
      <p class="text-gray-700">Waiting for application initialization...</p>
    {/if}
  </div>
{/if}
