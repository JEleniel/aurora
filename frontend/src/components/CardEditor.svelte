<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { formatDistanceToNow } from 'date-fns';
  import { ChevronDown } from 'lucide-svelte';
  import HistoryViewer from './HistoryViewer.svelte';

  export let cardId: string;

  let card: any = null;
  let loading = true;
  let saving = false;
  let showHistory = false;
  let editMode = false;

  // Edit form state
  let editForm = {
    name: '',
    description: '',
    status: '',
    rationale: '',
    priority: '',
    owner: '',
  };

  onMount(async () => {
    await loadCard();
  });

  async function loadCard() {
    loading = true;
    try {
      const response = await invoke('get_card', { id: cardId });
      if (response.success && response.data) {
        card = response.data;
        editForm = {
          name: card.name || '',
          description: card.description || '',
          status: card.status || '',
          rationale: card.rationale || '',
          priority: card.priority || '',
          owner: card.owner || '',
        };
      }
    } catch (error) {
      console.error('Error loading card:', error);
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    if (!editMode) {
      editMode = true;
      return;
    }

    saving = true;
    try {
      const response = await invoke('update_card', {
        id: cardId,
        updates: editForm,
      });
      if (response.success) {
        card = response.data;
        editMode = false;
        dispatch('save');
      }
    } catch (error) {
      console.error('Error saving card:', error);
    } finally {
      saving = false;
    }
  }

  function dispatch(type: string) {
    const event = new CustomEvent(type, { detail: card });
    window.dispatchEvent(event);
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      proposed: 'bg-gray-100 text-gray-700',
      approved: 'bg-blue-100 text-blue-700',
      implemented: 'bg-green-100 text-green-700',
      verified: 'bg-emerald-100 text-emerald-700',
      deprecated: 'bg-yellow-100 text-yellow-700',
      retired: 'bg-red-100 text-red-700',
    };
    return colors[status] || 'bg-gray-100 text-gray-700';
  }
</script>

<div class="h-full flex flex-col overflow-hidden bg-white">
  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-gray-500">Loading card...</div>
    </div>
  {:else if card}
    <!-- Header -->
    <div class="border-b border-gray-200 px-6 py-4 flex-shrink-0">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          {#if editMode}
            <input
              type="text"
              bind:value={editForm.name}
              class="text-2xl font-bold text-gray-900 w-full px-2 py-1 border border-gray-300 rounded"
            />
          {:else}
            <h2 class="text-2xl font-bold text-gray-900">{card.name}</h2>
          {/if}
          <p class="text-sm text-gray-500 mt-1">{card.id}</p>
        </div>

        <div class="flex items-center gap-2">
          {#if card.status}
            <span
              class={`text-xs px-3 py-1 rounded-full font-medium ${getStatusColor(card.status)}`}
            >
              {card.status}
            </span>
          {/if}
          <button
            on:click={handleSave}
            disabled={saving}
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition"
          >
            {editMode ? 'Save' : 'Edit'}
          </button>
        </div>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-6 py-6">
      <div class="space-y-6 max-w-3xl">
        <!-- Metadata -->
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <p class="text-gray-600 font-medium">Changes</p>
            <p class="text-gray-900 text-lg">{card.change_counter}</p>
          </div>
          <div>
            <p class="text-gray-600 font-medium">Last Modified</p>
            <p class="text-gray-900">
              {formatDistanceToNow(new Date(card.last_modified), {
                addSuffix: true,
              })}
            </p>
          </div>
          {#if card.owner}
            <div>
              <p class="text-gray-600 font-medium">Owner</p>
              <p class="text-gray-900">{card.owner}</p>
            </div>
          {/if}
          {#if card.version}
            <div>
              <p class="text-gray-600 font-medium">Version</p>
              <p class="text-gray-900">{card.version}</p>
            </div>
          {/if}
        </div>

        <!-- Description -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2"
            >Description</label
          >
          {#if editMode}
            <textarea
              bind:value={editForm.description}
              rows="4"
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          {:else}
            <p class="text-gray-700 whitespace-pre-wrap">
              {card.description || 'No description'}
            </p>
          {/if}
        </div>

        <!-- Rationale -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2"
            >Rationale</label
          >
          {#if editMode}
            <textarea
              bind:value={editForm.rationale}
              rows="3"
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          {:else}
            <p class="text-gray-700">{card.rationale || 'No rationale'}</p>
          {/if}
        </div>

        <!-- Additional metadata -->
        {#if editMode}
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2"
                >Priority</label
              >
              <select
                bind:value={editForm.priority}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg"
              >
                <option value="">None</option>
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2"
                >Owner</label
              >
              <input
                type="text"
                bind:value={editForm.owner}
                class="w-full px-3 py-2 border border-gray-300 rounded-lg"
              />
            </div>
          </div>
        {/if}

        <!-- History button -->
        <button
          on:click={() => (showHistory = !showHistory)}
          class="flex items-center gap-2 text-blue-600 hover:text-blue-700 text-sm font-medium mt-4"
        >
          <span>View Change History ({card.audit_history?.length || 0})</span>
          <ChevronDown
            size={16}
            class={`transition ${showHistory ? 'rotate-180' : ''}`}
          />
        </button>

        <!-- History -->
        {#if showHistory}
          <HistoryViewer history={card.audit_history} />
        {/if}
      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-gray-500">Card not found</div>
    </div>
  {/if}
</div>
