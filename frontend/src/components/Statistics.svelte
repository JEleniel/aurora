<script lang="ts">
  export let stats: Record<string, number> = {};
</script>

<div class="grid grid-cols-3 gap-4">
  <div class="bg-white rounded-lg border border-gray-200 p-4 text-center">
    <p class="text-gray-600 text-sm">Total Cards</p>
    <p class="text-3xl font-bold text-gray-900 mt-2">
      {stats['total_cards'] || 0}
    </p>
  </div>
  <div class="bg-white rounded-lg border border-gray-200 p-4 text-center">
    <p class="text-gray-600 text-sm">Links</p>
    <p class="text-3xl font-bold text-gray-900 mt-2">
      {stats['total_links'] || 0}
    </p>
  </div>
  <div class="bg-white rounded-lg border border-gray-200 p-4 text-center">
    <p class="text-gray-600 text-sm">Views</p>
    <p class="text-3xl font-bold text-gray-900 mt-2">
      {stats['total_views'] || 0}
    </p>
  </div>
</div>
