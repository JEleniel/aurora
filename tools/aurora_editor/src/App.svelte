<script lang="ts">
	import { onMount } from 'svelte';

	import ActionButton from './lib/components/ActionButton.svelte';
	import StatusPill from './lib/components/StatusPill.svelte';
	import SurfaceCard from './lib/components/SurfaceCard.svelte';
	import {
		discoverModels,
		chooseWorkspaceFolder,
		healthCheck,
		isTauriAvailable,
		renderAllAssets,
		setWorkspace,
		validateModelSnapshot,
		workspaceStatus,
		writeCompactExport
	} from './lib/tauri';
	import type { ModelHomeInfo, ValidationReport, WorkspaceInfo } from './lib/tauri';

	type StatusTone = 'good' | 'warn' | 'error' | 'neutral';

	let backendTone: StatusTone = 'neutral';
	let backendLabel = 'Checking backend';
	let backendDetail = 'Connecting to the local Tauri host.';
	let lastPing = '—';

	let actionTone: StatusTone = 'neutral';
	let actionLabel = 'Awaiting actions';
	let actionDetail = 'No commands have been sent yet.';

	let workspaceStatusText = 'No workspace selected';
	let workspacePath = '';
	let workspaceTrusted = false;
	let currentWorkspace: WorkspaceInfo | null = null;

	let modelHomes: ModelHomeInfo[] = [];
	let selectedModelHome = '';
	let outputDir = 'docs/design/';
	let compactOutput = '';

	let validationSummary = 'No report available yet.';
	let validationTone: StatusTone = 'neutral';
	let diagnostics = [
		{ title: 'Schema validation', detail: 'No report available yet.' },
		{ title: 'Top findings', detail: 'Waiting for the first load.' },
		{ title: 'Workspace trust', detail: 'Pending trust verification.' }
	];

	let isBusy = false;
	let interactionCount = 0;
	let lastInteraction = 'No interactions yet.';

	function formatTime(value: Date): string {
		return new Intl.DateTimeFormat('en', {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		}).format(value);
	}

	function formatError(error: unknown): string {
		if (error instanceof Error) {
			return error.message;
		}
		if (typeof error === 'string') {
			return error;
		}
		if (error && typeof error === 'object') {
			const record = error as Record<string, unknown>;
			if (typeof record['message'] === 'string') {
				return record['message'] as string;
			}
			if (typeof record['error'] === 'string') {
				return record['error'] as string;
			}
			if (typeof record['details'] === 'string') {
				return record['details'] as string;
			}
			try {
				return JSON.stringify(record);
			} catch {
				return 'Unknown error response';
			}
		}
		return String(error);
	}

	function setAction(tone: StatusTone, label: string, detail: string): void {
		actionTone = tone;
		actionLabel = label;
		actionDetail = detail;
	}

	function updateWorkspace(info: WorkspaceInfo): void {
		currentWorkspace = info;
		workspacePath = info.root;
		workspaceTrusted = info.trusted;
		workspaceStatusText = `${info.root} · ${info.trusted ? 'Trusted' : 'Untrusted'}`;
		updateTrustDiagnostic();
	}

	function updateTrustDiagnostic(): void {
		const trustState = currentWorkspace?.trusted ? 'Trusted' : 'Untrusted';
		diagnostics = diagnostics.map((entry) =>
			entry.title === 'Workspace trust' ? { ...entry, detail: trustState } : entry
		);
	}

	function recordInteraction(source: string): void {
		interactionCount += 1;
		lastInteraction = `${source} · ${formatTime(new Date())}`;
	}

	function ensureBackendAvailable(): boolean {
		if (!isTauriAvailable()) {
			setAction(
				'error',
				'Backend unavailable',
				'Tauri host not detected. Launch the desktop app to enable actions.'
			);
			return false;
		}
		return true;
	}

	async function pingBackend(): Promise<void> {
		if (!isTauriAvailable()) {
			backendTone = 'error';
			backendLabel = 'Backend unavailable';
			backendDetail = 'Tauri host not detected. Launch the desktop app to enable actions.';
			lastPing = formatTime(new Date());
			return;
		}
		try {
			await healthCheck();
			backendTone = 'good';
			backendLabel = 'Backend online';
			backendDetail = 'Tauri host responded successfully.';
		} catch (error) {
			backendTone = 'error';
			backendLabel = 'Backend offline';
			backendDetail = formatError(error);
			console.error(error);
		}
		lastPing = formatTime(new Date());
	}

	async function refreshWorkspaceStatus(): Promise<void> {
		if (!isTauriAvailable()) {
			return;
		}
		try {
			const info = await workspaceStatus();
			updateWorkspace(info);
			await refreshModels();
		} catch (error) {
			console.warn('Workspace status not available yet', error);
		}
	}

	async function refreshModels(): Promise<void> {
		modelHomes = await discoverModels();
		if (modelHomes.length === 0) {
			selectedModelHome = '';
			return;
		}
		const matched = modelHomes.find((home) => home.root === selectedModelHome);
		if (!matched) {
			const firstHome = modelHomes[0];
			if (firstHome) {
				selectedModelHome = firstHome.root;
			}
		}
		if (selectedModelHome) {
			await runValidation(selectedModelHome);
		}
	}

	function summarizeDiagnostics(report: ValidationReport): string {
		let errors = 0;
		let warnings = 0;
		let infos = 0;
		for (const diagnostic of report.diagnostics) {
			switch (diagnostic.severity) {
				case 'Error':
					errors += 1;
					break;
				case 'Warning':
					warnings += 1;
					break;
				case 'Info':
					infos += 1;
					break;
			}
		}
		validationTone =
			errors > 0 ? 'error'
			: warnings > 0 ? 'warn'
			: 'good';
		return `${errors} errors · ${warnings} warnings · ${infos} info`;
	}

	function topFindings(report: ValidationReport): string {
		const findings = report.diagnostics.slice(0, 3).map((entry) => {
			return `${entry.code}: ${entry.message}`;
		});
		return findings.length > 0 ? findings.join(' • ') : 'No diagnostics reported.';
	}

	async function runValidation(modelHome: string): Promise<void> {
		try {
			const report = await validateModelSnapshot(modelHome);
			validationSummary = summarizeDiagnostics(report);
			diagnostics = [
				{ title: 'Schema validation', detail: validationSummary },
				{ title: 'Top findings', detail: topFindings(report) },
				{
					title: 'Workspace trust',
					detail: currentWorkspace?.trusted ? 'Trusted' : 'Untrusted'
				}
			];
		} catch (error) {
			validationTone = 'warn';
			validationSummary = 'Validation unavailable.';
			diagnostics = [
				{ title: 'Schema validation', detail: validationSummary },
				{ title: 'Top findings', detail: formatError(error) },
				{
					title: 'Workspace trust',
					detail: currentWorkspace?.trusted ? 'Trusted' : 'Untrusted'
				}
			];
		}
	}

	async function applyWorkspace(normalizedPath: string): Promise<void> {
		isBusy = true;
		setAction('neutral', 'Connecting workspace', 'Applying workspace configuration.');
		try {
			const info = await setWorkspace(normalizedPath, workspaceTrusted);
			updateWorkspace(info);
			await refreshModels();
			setAction('good', 'Workspace connected', 'Model inventory refreshed successfully.');
		} catch (error) {
			setAction('error', 'Workspace error', formatError(error));
		} finally {
			isBusy = false;
		}
	}

	async function handleChooseWorkspace(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		const selection = await chooseWorkspaceFolder();
		if (!selection) {
			return;
		}
		workspacePath = selection;
		await applyWorkspace(selection);
	}

	async function handleRefreshModels(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		if (!currentWorkspace) {
			setAction('warn', 'Workspace missing', 'Connect a workspace first.');
			return;
		}
		isBusy = true;
		setAction('neutral', 'Refreshing models', 'Discovering model homes...');
		try {
			await refreshModels();
			setAction('good', 'Models refreshed', 'Discovery and validation complete.');
		} catch (error) {
			setAction('error', 'Refresh failed', formatError(error));
		} finally {
			isBusy = false;
		}
	}

	async function handleValidate(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		if (!selectedModelHome) {
			setAction('warn', 'Model not selected', 'Select a model home to validate.');
			return;
		}
		isBusy = true;
		setAction('neutral', 'Validating model', 'Running schema and invariant checks.');
		try {
			await runValidation(selectedModelHome);
			setAction('good', 'Validation complete', validationSummary);
		} catch (error) {
			setAction('error', 'Validation failed', formatError(error));
		} finally {
			isBusy = false;
		}
	}

	async function handleRenderAll(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		if (!selectedModelHome) {
			setAction('warn', 'Model not selected', 'Select a model home to render.');
			return;
		}
		if (!currentWorkspace?.trusted) {
			setAction('warn', 'Workspace untrusted', 'Enable trust to render artifacts.');
			return;
		}
		const normalizedOutput = outputDir.trim();
		if (!normalizedOutput) {
			setAction('warn', 'Output directory required', 'Enter a workspace-relative output directory.');
			return;
		}
		isBusy = true;
		setAction('neutral', 'Rendering views', 'Generating DOT, SVG, and Markdown output.');
		try {
			const summary = await renderAllAssets(selectedModelHome, normalizedOutput);
			setAction(
				'good',
				'Render complete',
				`Wrote ${summary.cards_written} cards and ${summary.views_written} views to ${summary.output_dir}.`
			);
		} catch (error) {
			setAction('error', 'Render failed', formatError(error));
		} finally {
			isBusy = false;
		}
	}

	async function handleCompactExport(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		if (!selectedModelHome) {
			setAction('warn', 'Model not selected', 'Select a model home to export.');
			return;
		}
		if (!currentWorkspace?.trusted) {
			setAction('warn', 'Workspace untrusted', 'Enable trust to export.');
			return;
		}
		isBusy = true;
		setAction('neutral', 'Exporting compact model', 'Packaging the agent-ready file.');
		try {
			const outputPath = compactOutput.trim().length > 0 ? compactOutput.trim() : null;
			const result = await writeCompactExport(selectedModelHome, outputPath);
			setAction('good', 'Compact export complete', `Exported to ${result}.`);
		} catch (error) {
			setAction('error', 'Export failed', formatError(error));
		} finally {
			isBusy = false;
		}
	}

	async function handleModelSelectionChange(): Promise<void> {
		if (selectedModelHome) {
			await runValidation(selectedModelHome);
		}
	}

	function handleWorkspaceKeydown(event: KeyboardEvent): void {
		if (event.key === 'Enter') {
			event.preventDefault();
			void handleChooseWorkspace();
		}
	}

	onMount(() => {
		const handlePointer = (event: PointerEvent): void => {
			const target = event.target;
			const label = target instanceof HTMLElement ? target.tagName.toLowerCase() : 'pointer';
			recordInteraction(`Pointer (${label})`);
		};
		const handleKeydown = (event: KeyboardEvent): void => {
			recordInteraction(`Key ${event.key}`);
		};
		window.addEventListener('pointerdown', handlePointer, { capture: true });
		window.addEventListener('keydown', handleKeydown, { capture: true });
		recordInteraction('UI ready');
		pingBackend();
		refreshWorkspaceStatus();
		return () => {
			window.removeEventListener('pointerdown', handlePointer, { capture: true });
			window.removeEventListener('keydown', handleKeydown, { capture: true });
		};
	});
</script>

<div>
	<header>
		<img src="/images/Aurora.64.png" alt="Aurora logo" />
		<div class="titles">
			<h1>Aurora Editor</h1>
			<p class="hero__eyebrow">Modeling workbench</p>
		</div>
	</header>

	<div class="content">
		<nav>
			<div class="nav__section">
				<h2>Workspace</h2>
				<p>{workspaceStatusText}</p>
			</div>
			<div class="nav__section">
				<h2>Model library</h2>
				<ul>
					<li>Default tooling mission</li>
					<li>Security & trust views</li>
					<li>Render pipelines</li>
				</ul>
			</div>
			<div class="nav__section">
				<h2>Viewports</h2>
				<ul>
					<li>Requirements map</li>
					<li>Capability heatmap</li>
					<li>Process flows</li>
				</ul>
			</div>
		</nav>

		<main>
			<SurfaceCard
				title="Workspace control"
				subtitle="Connect a trusted workspace and select the model home to operate on."
			>
				<div class="form-grid">
					<div class="field">
						<label for="workspace-path">Workspace root</label>
						<div class="field-row">
							<input
								id="workspace-path"
								type="text"
								placeholder="/home/you/repos/aurora"
								bind:value={workspacePath}
								readonly
								on:keydown={handleWorkspaceKeydown}
							/>
							<button class="ghost" type="button" on:click={handleChooseWorkspace} disabled={isBusy}>
								Choose folder
							</button>
						</div>
						<p class="field__hint">Select a workspace folder to scan for Aurora model homes.</p>
					</div>
					<label class="toggle">
						<input type="checkbox" bind:checked={workspaceTrusted} />
						<span>Trust workspace for render and export operations</span>
					</label>
					<div class="field">
						<label for="model-home">Model home</label>
						<select id="model-home" bind:value={selectedModelHome} on:change={handleModelSelectionChange}>
							<option value="">Select a model home</option>
							{#each modelHomes as home (home.root)}
								<option value={home.root}>
									{home.root}
									{home.has_schema ? '' : '· missing schema'}
								</option>
							{/each}
						</select>
						<p class="field__hint">
							{#if modelHomes.length === 0}
								No model homes detected yet.
							{:else}
								Detected {modelHomes.length} model home{modelHomes.length === 1 ? '' : 's'}.
							{/if}
						</p>
						<details class="field__details">
							<summary>Show detected model homes</summary>
							<ul>
								{#each modelHomes as home (home.root)}
									<li>{home.root}</li>
								{/each}
							</ul>
						</details>
					</div>
					<div class="field-grid">
						<div class="field">
							<label for="output-dir">Render output directory</label>
							<input id="output-dir" type="text" bind:value={outputDir} />
							<p class="field__hint">
								Use a workspace-relative path (~/ is supported within the workspace).
							</p>
						</div>
						<div class="field">
							<label for="compact-output">Compact export path (optional)</label>
							<input
								id="compact-output"
								type="text"
								placeholder="AGENT-MIS-001.jsjson"
								bind:value={compactOutput}
							/>
						</div>
					</div>
					<div class="field-actions">
						<button class="primary" type="button" on:click={handleChooseWorkspace} disabled={isBusy}>
							Choose workspace
						</button>
						<button
							class="ghost"
							type="button"
							on:click={handleRefreshModels}
							disabled={isBusy || !currentWorkspace}
						>
							Refresh models
						</button>
					</div>
				</div>
			</SurfaceCard>

			<SurfaceCard title="Quick actions" subtitle="Run validation, render views, and export compact models.">
				<div class="action-grid">
					<ActionButton
						label="Validate model"
						description="Run schema and invariant checks."
						on:click={handleValidate}
						disabled={isBusy || !selectedModelHome}
					/>
					<ActionButton
						label="Render all views"
						description="Generate DOT, SVG, and Markdown outputs."
						on:click={handleRenderAll}
						disabled={isBusy || !selectedModelHome || !currentWorkspace?.trusted}
					/>
					<ActionButton
						label="Compact export"
						description="Write the agent-ready compact model file."
						on:click={handleCompactExport}
						disabled={isBusy || !selectedModelHome || !currentWorkspace?.trusted}
					/>
				</div>
			</SurfaceCard>

			<SurfaceCard title="System pulse" subtitle="Live diagnostics and trusted workspace indicators.">
				<div class="pulse-grid">
					<div class="pulse">
						<span>Backend</span>
						<strong>{backendDetail}</strong>
					</div>
					<div class="pulse">
						<span>Workspace</span>
						<strong>{workspaceStatusText}</strong>
					</div>
					<div class="pulse">
						<span>Command queue</span>
						<strong>{actionDetail}</strong>
					</div>
					<div class="pulse">
						<span>Last interaction</span>
						<strong>{lastInteraction}</strong>
						<small>{interactionCount} interactions recorded</small>
					</div>
				</div>
			</SurfaceCard>

			<SurfaceCard title="Diagnostics feed" subtitle="Validation output appears here as the model changes.">
				<div slot="action" class="diagnostic-pill">
					<StatusPill tone={validationTone}>{validationSummary}</StatusPill>
				</div>
				<ul class="diagnostics">
					{#each diagnostics as diagnostic}
						<li>
							<span>{diagnostic.title}</span>
							<strong>{diagnostic.detail}</strong>
						</li>
					{/each}
				</ul>
			</SurfaceCard>
		</main>
	</div>
</div>
