<script lang="ts">
  import { formatDistanceToNow } from 'date-fns';
  import { ChevronDown } from 'lucide-svelte';

  export let history: any[] = [];

  let expandedChanges = new Set<number>();

  function toggleExpand(changeNumber: number) {
    if (expandedChanges.has(changeNumber)) {
      expandedChanges.delete(changeNumber);
    } else {
      expandedChanges.add(changeNumber);
    }
    expandedChanges = expandedChanges;
  }

  function formatDate(timestamp: string): string {
    return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
  }

  function getEventColor(event: string): string {
    const colors: Record<string, string> = {
      created: 'bg-green-100 text-green-700',
      modified: 'bg-blue-100 text-blue-700',
      approved: 'bg-purple-100 text-purple-700',
      deprecated: 'bg-yellow-100 text-yellow-700',
      retired: 'bg-red-100 text-red-700',
    };
    return colors[event] || 'bg-gray-100 text-gray-700';
  }
</script>

<div class="bg-gray-50 rounded-lg border border-gray-200 mt-4 overflow-hidden">
  <div class="divide-y divide-gray-200">
    {#each history as change (change.change_number)}
      <div class="p-4">
        <button
          on:click={() => toggleExpand(change.change_number)}
          class="w-full text-left flex items-center justify-between hover:bg-gray-100 rounded px-2 py-1 -mx-2 -my-1"
        >
          <div class="flex items-center gap-3 flex-1">
            <span
              class={`text-xs px-2 py-1 rounded font-medium ${getEventColor(change.event)}`}
            >
              {change.event}
            </span>
            <div class="min-w-0 flex-1">
              <p class="font-medium text-gray-900">
                Change #{change.change_number}
              </p>
              <p class="text-xs text-gray-500">
                {formatDate(change.timestamp)} by {change.by}
              </p>
            </div>
          </div>
          <ChevronDown
            size={16}
            class={`text-gray-400 transition ${expandedChanges.has(change.change_number) ? 'rotate-180' : ''}`}
          />
        </button>

        {#if expandedChanges.has(change.change_number)}
          <div class="mt-3 space-y-2 ml-12 text-sm">
            {#if change.fields_modified && change.fields_modified.length > 0}
              <div>
                <p class="font-medium text-gray-700">Fields Modified:</p>
                <ul class="list-disc list-inside text-gray-600 mt-1">
                  {#each change.fields_modified as field}
                    <li>{field}</li>
                  {/each}
                </ul>
              </div>
            {/if}

            {#if change.note}
              <div>
                <p class="font-medium text-gray-700">Note:</p>
                <p class="text-gray-600 mt-1">{change.note}</p>
              </div>
            {/if}

            {#if change.previous_values && Object.keys(change.previous_values).length > 0}
              <div>
                <p class="font-medium text-gray-700">Previous Values:</p>
                <div
                  class="bg-white rounded mt-1 p-2 space-y-1 text-xs font-mono"
                >
                  {#each Object.entries(change.previous_values) as [key, value]}
                    <div>
                      <span class="text-gray-600">{key}:</span>
                      <span class="text-gray-900">{JSON.stringify(value)}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>
