<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  interface TestResult {
    name: string;
    status: 'pending' | 'running' | 'success' | 'error';
    result?: string;
    error?: string;
    timestamp?: number;
  }

  let tests: TestResult[] = [];
  let isRunning = false;
  let expandedTest: string | null = null;
  let eventLog: string[] = [];
  let toggleState = false;
  let formData = { name: '', description: '' };
  let formResponse = '';

  const testSuites = [
    {
      name: 'IPC Communication',
      tests: [
        { name: 'query_get_statistics', command: 'query_get_statistics', params: {} },
        { name: 'query_list_cards', command: 'query_list_cards', params: {} },
        { name: 'query_list_links', command: 'query_list_links', params: {} },
        { name: 'query_list_views', command: 'query_list_views', params: {} },
      ],
    },
    {
      name: 'Button Events',
      tests: [
        { name: 'Toggle Button Click', action: 'toggle' },
        { name: 'Form Submit', action: 'form' },
        { name: 'Multiple Rapid Clicks', action: 'rapidClick' },
      ],
    },
  ];

  onMount(() => {
    // Initialize tests
    testSuites.forEach((suite) => {
      suite.tests.forEach((test) => {
        tests.push({
          name: suite.name + ' - ' + test.name,
          status: 'pending',
        });
      });
    });
    tests = tests; // Trigger reactivity
    addLog('Test suite initialized');
  });

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    eventLog = [`[${timestamp}] ${message}`, ...eventLog.slice(0, 99)];
  }

  function toggleButton() {
    toggleState = !toggleState;
    addLog(`Toggle button clicked: now ${toggleState}`);
  }

  async function submitForm(e: Event) {
    e.preventDefault();
    addLog(`Form submitted: ${JSON.stringify(formData)}`);
    formResponse = 'Form submitted at ' + new Date().toLocaleTimeString();
    // Reset after 2 seconds
    setTimeout(() => {
      formResponse = '';
    }, 2000);
  }

  async function runAllTests() {
    isRunning = true;
    addLog('Starting test suite...');

    for (let i = 0; i < tests.length; i++) {
      const test = tests[i];
      test.status = 'running';
      tests = tests;

      // Find the corresponding test config
      let foundTest = false;
      for (const suite of testSuites) {
        for (const suiteTest of suite.tests) {
          const fullName = suite.name + ' - ' + suiteTest.name;
          if (fullName === test.name) {
            foundTest = true;
            try {
              if ('command' in suiteTest) {
                // IPC test
                const response = await invoke(suiteTest.command, suiteTest.params);
                test.status = 'success';
                test.result = JSON.stringify(response, null, 2);
                addLog(`✓ ${test.name}`);
              } else if ('action' in suiteTest) {
                // Action test
                if (suiteTest.action === 'toggle') {
                  toggleButton();
                  test.status = 'success';
                  test.result = 'Toggle event fired';
                } else if (suiteTest.action === 'form') {
                  const formEvent = new Event('submit');
                  await submitForm(formEvent);
                  test.status = 'success';
                  test.result = 'Form event fired';
                } else if (suiteTest.action === 'rapidClick') {
                  for (let j = 0; j < 5; j++) {
                    toggleButton();
                  }
                  test.status = 'success';
                  test.result = '5 rapid clicks completed';
                }
                addLog(`✓ ${test.name}`);
              }
            } catch (error) {
              test.status = 'error';
              test.error = String(error);
              addLog(`✗ ${test.name}: ${error}`);
            }
            tests = tests;
            break;
          }
        }
        if (foundTest) break;
      }

      // Small delay between tests
      await new Promise((resolve) => setTimeout(resolve, 200));
    }

    isRunning = false;
    addLog('Test suite complete');
  }

  function toggleExpand(name: string) {
    expandedTest = expandedTest === name ? null : name;
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      pending: 'bg-gray-100 text-gray-700',
      running: 'bg-blue-100 text-blue-700',
      success: 'bg-green-100 text-green-700',
      error: 'bg-red-100 text-red-700',
    };
    return colors[status] || 'bg-gray-100 text-gray-700';
  }

  function getStatusText(status: string): string {
    const texts: Record<string, string> = {
      pending: '⏳ Pending',
      running: '⚙️ Running',
      success: '✓ Success',
      error: '✗ Error',
    };
    return texts[status] || status;
  }
</script>

<div class="w-full h-screen bg-gradient-to-br from-gray-50 to-gray-100 overflow-hidden">
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-white border-b border-gray-200 px-6 py-4 flex-shrink-0">
      <h1 class="text-3xl font-bold text-gray-900">AURORA Button Debug Suite</h1>
      <p class="text-gray-600 text-sm mt-1">
        Test Tauri IPC communication and interactive components
      </p>
    </div>

    <!-- Main content -->
    <div class="flex-1 overflow-hidden flex gap-4 p-4">
      <!-- Left panel: Test controls and results -->
      <div class="flex-1 flex flex-col bg-white rounded-lg border border-gray-200 overflow-hidden">
        <!-- Test controls -->
        <div class="border-b border-gray-200 px-6 py-4 bg-gray-50 flex-shrink-0">
          <button
            on:click={runAllTests}
            disabled={isRunning}
            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition font-medium"
          >
            {isRunning ? '⚙️ Running Tests...' : '▶️ Run All Tests'}
          </button>
        </div>

        <!-- Test results -->
        <div class="flex-1 overflow-y-auto">
          <div class="divide-y divide-gray-200">
            {#each tests as test (test.name)}
              <div class="border-b border-gray-100">
                <button
                  on:click={() => toggleExpand(test.name)}
                  class="w-full text-left p-4 hover:bg-gray-50 transition flex items-center justify-between"
                >
                  <div class="flex items-center gap-3 flex-1">
                    <span
                      class={`text-xs px-2 py-1 rounded font-medium ${getStatusColor(test.status)}`}
                    >
                      {getStatusText(test.status)}
                    </span>
                    <span class="text-sm font-medium text-gray-900">{test.name}</span>
                  </div>
                  <span class="text-gray-400">
                    {expandedTest === test.name ? '▼' : '▶'}
                  </span>
                </button>

                {#if expandedTest === test.name}
                  <div class="px-4 py-3 bg-gray-50 border-t border-gray-200">
                    {#if test.result}
                      <div class="bg-white rounded border border-green-200 p-3 mb-2">
                        <p class="text-xs font-mono text-green-700 whitespace-pre-wrap break-words">
                          {test.result}
                        </p>
                      </div>
                    {/if}
                    {#if test.error}
                      <div class="bg-white rounded border border-red-200 p-3">
                        <p class="text-xs font-mono text-red-700 whitespace-pre-wrap break-words">
                          {test.error}
                        </p>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Right panel: Interactive components and logs -->
      <div class="flex-1 flex flex-col gap-4">
        <!-- Interactive test section -->
        <div class="bg-white rounded-lg border border-gray-200 p-6 flex-shrink-0">
          <h2 class="text-lg font-semibold text-gray-900 mb-4">Interactive Tests</h2>

          <!-- Toggle button test -->
          <div class="mb-6">
            <p class="text-sm font-medium text-gray-700 mb-2">Toggle Button</p>
            <button
              on:click={toggleButton}
              class={`px-4 py-2 rounded-lg font-medium transition ${toggleState ? 'bg-green-600 text-white hover:bg-green-700' : 'bg-gray-200 text-gray-900 hover:bg-gray-300'}`}
            >
              {toggleState ? '✓ ON' : '○ OFF'}
            </button>
            <p class="text-xs text-gray-500 mt-2">
              Current state: <span class="font-mono">{toggleState}</span>
            </p>
          </div>

          <!-- Form test -->
          <div class="border-t border-gray-200 pt-4">
            <p class="text-sm font-medium text-gray-700 mb-3">Form Submission</p>
            <form on:submit={submitForm} class="space-y-3">
              <input
                type="text"
                bind:value={formData.name}
                placeholder="Name"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <input
                type="text"
                bind:value={formData.description}
                placeholder="Description"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <button
                type="submit"
                class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium text-sm"
              >
                Submit Form
              </button>
            </form>
            {#if formResponse}
              <p class="text-xs text-green-700 mt-2">{formResponse}</p>
            {/if}
          </div>
        </div>

        <!-- Event log -->
        <div class="flex-1 bg-white rounded-lg border border-gray-200 p-4 flex flex-col min-h-0">
          <h2 class="text-sm font-semibold text-gray-900 mb-3">Event Log</h2>
          <div class="flex-1 overflow-y-auto bg-gray-50 rounded border border-gray-200 p-2">
            {#if eventLog.length === 0}
              <p class="text-xs text-gray-500">Waiting for events...</p>
            {:else}
              <div class="space-y-1">
                {#each eventLog as event (event)}
                  <p class="text-xs font-mono text-gray-700">{event}</p>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
  }
</style>
