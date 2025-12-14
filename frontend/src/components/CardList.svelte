<script lang="ts">
  import { Plus, Search } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let cards: any[] = [];
  export let selectedCardId: string | null = null;

  const dispatch = createEventDispatcher();

  let searchQuery = '';
  let showNewCardDialog = false;
  let newCardType = 'driver';
  let newCardName = '';
  let newCardId = '';

  $: filteredCards = cards.filter(
    (card) =>
      card.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      card.id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function handleSelectCard(cardId: string) {
    dispatch('select', cardId);
  }

  function handleCreateCard() {
    if (!newCardId || !newCardName) return;

    dispatch('create', {
      id: newCardId,
      type: newCardType,
      name: newCardName,
    });

    newCardId = '';
    newCardName = '';
    newCardType = 'driver';
    showNewCardDialog = false;
  }

  function getCardTypeColor(type: string): string {
    const colors: Record<string, string> = {
      driver: 'bg-red-100 text-red-700',
      requirement: 'bg-blue-100 text-blue-700',
      behavior: 'bg-purple-100 text-purple-700',
      interface: 'bg-green-100 text-green-700',
      constraint: 'bg-yellow-100 text-yellow-700',
      actor: 'bg-pink-100 text-pink-700',
      'logical-component': 'bg-indigo-100 text-indigo-700',
      'deployable-node': 'bg-cyan-100 text-cyan-700',
      test: 'bg-orange-100 text-orange-700',
      artifact: 'bg-gray-100 text-gray-700',
      view: 'bg-teal-100 text-teal-700',
      note: 'bg-slate-100 text-slate-700',
    };
    return colors[type] || 'bg-gray-100 text-gray-700';
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Search -->
  <div class="p-4 border-b border-gray-200 flex-shrink-0">
    <div class="relative">
      <Search size={16} class="absolute left-3 top-2.5 text-gray-400" />
      <input
        type="text"
        placeholder="Search cards..."
        bind:value={searchQuery}
        class="w-full pl-9 pr-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>
  </div>

  <!-- Cards list -->
  <div class="flex-1 overflow-y-auto">
    {#each filteredCards as card (card.id)}
      <button
        on:click={() => handleSelectCard(card.id)}
        class={`w-full text-left px-4 py-3 border-b border-gray-100 hover:bg-gray-50 transition ${
          selectedCardId === card.id
            ? 'bg-blue-50 border-l-4 border-l-blue-600'
            : ''
        }`}
      >
        <div class="flex items-start gap-2">
          <span
            class={`text-xs px-2 py-1 rounded font-medium flex-shrink-0 mt-0.5 ${getCardTypeColor(card.type)}`}
          >
            {card.type}
          </span>
          <div class="min-w-0 flex-1">
            <p class="font-medium text-gray-900 truncate">{card.name}</p>
            <p class="text-xs text-gray-500 truncate">{card.id}</p>
          </div>
        </div>
      </button>
    {/each}
  </div>

  <!-- New card button -->
  <div class="p-4 border-t border-gray-200 flex-shrink-0">
    <button
      on:click={() => (showNewCardDialog = true)}
      class="w-full flex items-center justify-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
    >
      <Plus size={16} />
      New Card
    </button>
  </div>
</div>

<!-- New Card Dialog -->
{#if showNewCardDialog}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  >
    <div class="bg-white rounded-lg p-6 w-96 shadow-lg">
      <h3 class="text-lg font-semibold mb-4">Create New Card</h3>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1"
            >Type</label
          >
          <select
            bind:value={newCardType}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg"
          >
            <option value="driver">Driver</option>
            <option value="requirement">Requirement</option>
            <option value="behavior">Behavior</option>
            <option value="interface">Interface</option>
            <option value="constraint">Constraint</option>
            <option value="actor">Actor</option>
            <option value="logical-component">Logical Component</option>
            <option value="deployable-node">Deployable Node</option>
            <option value="test">Test</option>
            <option value="artifact">Artifact</option>
            <option value="view">View</option>
            <option value="note">Note</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">ID</label>
          <input
            type="text"
            bind:value={newCardId}
            placeholder="namespace:element-name"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
          />
          <p class="text-xs text-gray-500 mt-1">
            Format: namespace:element-name
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1"
            >Name</label
          >
          <input
            type="text"
            bind:value={newCardName}
            placeholder="Card name"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
          />
        </div>
      </div>

      <div class="flex gap-3 mt-6">
        <button
          on:click={() => (showNewCardDialog = false)}
          class="flex-1 px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
        >
          Cancel
        </button>
        <button
          on:click={handleCreateCard}
          disabled={!newCardId || !newCardName}
          class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition"
        >
          Create
        </button>
      </div>
    </div>
  </div>
{/if}
