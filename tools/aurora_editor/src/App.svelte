<script lang="ts">
	import { onMount } from 'svelte';

	import ActionButton from './lib/components/ActionButton.svelte';
	import StatusPill from './lib/components/StatusPill.svelte';
	import SurfaceCard from './lib/components/SurfaceCard.svelte';
	import {
		discoverModels,
		healthCheck,
		isTauriAvailable,
		renderAllAssets,
		setWorkspace,
		validateModelSnapshot,
		workspaceStatus,
		writeCompactExport,
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
	let outputDir = 'docs/design';
	let compactOutput = '';

	let validationSummary = 'No report available yet.';
	let validationTone: StatusTone = 'neutral';
	let diagnostics = [
		{ title: 'Schema validation', detail: 'No report available yet.' },
		{ title: 'Top findings', detail: 'Waiting for the first load.' },
		{ title: 'Workspace trust', detail: 'Pending trust verification.' },
	];

	let isBusy = false;
	let interactionCount = 0;
	let lastInteraction = 'No interactions yet.';

	function formatTime(value: Date): string {
		return new Intl.DateTimeFormat('en', {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit',
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
			entry.title === 'Workspace trust' ? { ...entry, detail: trustState } : entry,
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
				'Tauri host not detected. Launch the desktop app to enable actions.',
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
					detail: currentWorkspace?.trusted ? 'Trusted' : 'Untrusted',
				},
			];
		} catch (error) {
			validationTone = 'warn';
			validationSummary = 'Validation unavailable.';
			diagnostics = [
				{ title: 'Schema validation', detail: validationSummary },
				{ title: 'Top findings', detail: formatError(error) },
				{
					title: 'Workspace trust',
					detail: currentWorkspace?.trusted ? 'Trusted' : 'Untrusted',
				},
			];
		}
	}

	async function handleConnectWorkspace(): Promise<void> {
		if (!ensureBackendAvailable()) {
			return;
		}
		const normalizedPath = workspacePath.trim();
		if (!normalizedPath) {
			setAction('warn', 'Workspace required', 'Enter a workspace root path to continue.');
			return;
		}
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
				`Wrote ${summary.cards_written} cards and ${summary.views_written} views to ${summary.output_dir}.`,
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
			void handleConnectWorkspace();
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

<div class="app">
	<header class="hero">
		<div>
			<p class="hero__eyebrow">Aurora Editor</p>
			<h1>Model, validate, and visualize architectural systems.</h1>
			<p class="hero__subtitle">
				A sleek command center for Aurora models with instant validation and sci-fi inspired clarity.
			</p>
		</div>
		<div class="hero__status">
			<StatusPill tone={backendTone}>{backendLabel}</StatusPill>
			<StatusPill tone={actionTone}>{actionLabel}</StatusPill>
			<p>Last check: {lastPing}</p>
			<p class="hero__status-detail">{actionDetail}</p>
		</div>
	</header>

	<section class="shell">
		<nav class="nav">
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

		<main class="main">
			<SurfaceCard
				title="Workspace control"
				subtitle="Connect a trusted workspace and select the model home to operate on."
			>
				<div class="form-grid">
					<div class="field">
						<label for="workspace-path">Workspace root</label>
						<input
							id="workspace-path"
							type="text"
							placeholder="/home/you/repos/aurora"
							bind:value={workspacePath}
							on:keydown={handleWorkspaceKeydown}
						/>
						<p class="field__hint">Supports ~ paths and absolute workspace roots.</p>
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
						<button class="primary" type="button" on:click={handleConnectWorkspace} disabled={isBusy}>
							Connect workspace
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
	</section>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		padding: 2.5rem clamp(1.5rem, 4vw, 4rem) 3rem;
	}

	.hero {
		display: flex;
		flex-wrap: wrap;
		gap: 2rem;
		align-items: center;
		justify-content: space-between;
	}

	.hero h1 {
		margin: 0;
		font-size: clamp(2rem, 4vw, 3.4rem);
		line-height: 1.1;
		color: #f8fafc;
		text-shadow: 0 0 40px rgba(56, 189, 248, 0.25);
	}

	.hero__eyebrow {
		margin: 0 0 0.75rem;
		font-size: 0.85rem;
		letter-spacing: 0.3em;
		text-transform: uppercase;
		color: #7dd3fc;
	}

	.hero__subtitle {
		margin: 1rem 0 0;
		max-width: 32rem;
		color: #cbd5f5;
		font-size: 1.05rem;
	}

	.hero__status {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		background: rgba(15, 23, 42, 0.8);
		border: 1px solid rgba(59, 130, 246, 0.3);
		border-radius: 1.5rem;
		padding: 1.25rem 1.5rem;
		box-shadow: var(--glow);
		min-width: 220px;
	}

	.hero__status p {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.85rem;
	}

	.hero__status-detail {
		color: #cbd5f5;
		line-height: 1.4;
	}

	.shell {
		display: grid;
		grid-template-columns: minmax(220px, 260px) minmax(0, 1fr);
		gap: 2rem;
	}

	.nav {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		background: var(--surface-strong);
		border: 1px solid var(--surface-border);
		border-radius: 1.75rem;
		padding: 1.5rem;
		box-shadow: 0 20px 45px rgba(15, 23, 42, 0.4);
	}

	.nav__section h2 {
		margin: 0 0 0.35rem;
		font-size: 0.9rem;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: #c7d2fe;
	}

	.nav__section p {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.nav__section ul {
		list-style: none;
		margin: 0.6rem 0 0;
		padding: 0;
		display: grid;
		gap: 0.4rem;
		color: #e2e8f0;
		font-size: 0.9rem;
	}

	.nav__section li {
		padding: 0.35rem 0.5rem;
		border-radius: 0.75rem;
		background: rgba(30, 41, 59, 0.4);
		border: 1px solid rgba(99, 102, 241, 0.2);
	}

	.main {
		display: grid;
		gap: 1.5rem;
	}

	.form-grid {
		display: grid;
		gap: 1rem;
	}

	.field {
		display: grid;
		gap: 0.4rem;
	}

	.field label {
		font-size: 0.85rem;
		text-transform: uppercase;
		letter-spacing: 0.14em;
		color: #a5b4fc;
	}

	.field input,
	.field select {
		padding: 0.75rem 0.9rem;
		border-radius: 0.9rem;
		border: 1px solid rgba(99, 102, 241, 0.35);
		background: rgba(15, 23, 42, 0.75);
		color: #e2e8f0;
		font-size: 0.95rem;
	}

	.field input::placeholder {
		color: rgba(226, 232, 240, 0.4);
	}

	.field__hint {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.8rem;
	}

	.field__details {
		color: var(--text-muted);
		font-size: 0.8rem;
	}

	.field__details summary {
		cursor: pointer;
	}

	.field__details ul {
		margin: 0.5rem 0 0;
		padding-left: 1.2rem;
	}

	.field-grid {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
	}

	.toggle {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-size: 0.9rem;
		color: #c7d2fe;
	}

	.toggle input {
		width: 1.05rem;
		height: 1.05rem;
		accent-color: #6366f1;
	}

	.field-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
	}

	.primary,
	.ghost {
		border-radius: 999px;
		padding: 0.65rem 1.4rem;
		font-weight: 600;
		border: 1px solid transparent;
		cursor: pointer;
		transition:
			transform 0.2s ease,
			box-shadow 0.2s ease;
	}

	.primary {
		background: linear-gradient(135deg, #4f46e5, #38bdf8);
		color: #0f172a;
		box-shadow: 0 16px 35px rgba(56, 189, 248, 0.3);
	}

	.primary:hover:not(:disabled) {
		transform: translateY(-1px);
	}

	.ghost {
		background: transparent;
		border-color: rgba(99, 102, 241, 0.4);
		color: #e2e8f0;
	}

	.primary:disabled,
	.ghost:disabled {
		opacity: 0.6;
		cursor: not-allowed;
		box-shadow: none;
	}

	.action-grid {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
	}

	.pulse-grid {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
	}

	.pulse {
		padding: 1rem 1.2rem;
		border-radius: 1rem;
		border: 1px solid rgba(129, 140, 248, 0.2);
		background: rgba(15, 23, 42, 0.65);
		display: grid;
		gap: 0.35rem;
	}

	.pulse span {
		font-size: 0.75rem;
		letter-spacing: 0.2em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.pulse strong {
		font-weight: 600;
		color: #e0e7ff;
	}

	.pulse small {
		font-size: 0.75rem;
		color: var(--text-muted);
	}

	.diagnostics {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 0.85rem;
	}

	.diagnostics li {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.8rem 1rem;
		border-radius: 1rem;
		border: 1px solid rgba(59, 130, 246, 0.2);
		background: rgba(30, 41, 59, 0.4);
	}

	.diagnostics span {
		font-size: 0.75rem;
		letter-spacing: 0.2em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.diagnostics strong {
		font-weight: 500;
		color: var(--text-soft);
	}

	.diagnostic-pill {
		align-self: center;
	}

	@media (max-width: 960px) {
		.shell {
			grid-template-columns: 1fr;
		}
	}
</style>
