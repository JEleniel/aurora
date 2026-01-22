<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	type ValidationReport = {
		valid: boolean;
		messages: string[];
	};

	type CardSummary = {
		id: string;
		name: string;
		cardType: string;
		cardSubtype?: string;
		status?: string;
		relativePath: string;
	};

	type MissionSummary = {
		id: string;
		name: string;
		cards: CardSummary[];
	};

	type ModelHomeSummary = {
		root: string;
		missions: MissionSummary[];
		cardCount: number;
	};

	type AuroraCard = {
		$schema?: string;
		id: string;
		card_type: string;
		card_subtype?: string;
		name: string;
		description: string;
		status?: string;
	};

	type LoadedCard = {
		path: string;
		raw: string;
		card: AuroraCard;
	};

	let modelHomePath = '';
	let summary: ModelHomeSummary | null = null;
	let selectedCard: CardSummary | null = null;
	let loadedCard: LoadedCard | null = null;
	let isLoadingModel = false;
	let loadError = '';
	let loadStatus = 'Select a model home to begin.';
	let cardJson = `{
"$schema": "./Aurora.schema.json",
"id": "MIS-001",
"card_type": "Mission",
"name": "Enable Deterministic Aurora CLI Tooling",
"description": "Demo card used by the Aurora Editor",
"links": [],
"audit_trail": {
	"version": "1.0.0",
	"hash": null,
	"history": [
		{
			"editor": "System",
			"timestamp": "2024-01-01T00:00:00Z",
			"event": "created",
			"hash": null
		}
	]
}
}`;
	let status: 'idle' | 'valid' | 'invalid' | 'error' = 'idle';
	let validationMessage =
		'Paste or edit an Aurora card JSON payload, then select "Validate" to run the shared Rust library.';
	let busy = false;

	function formatError(error: unknown): string {
		return error instanceof Error ? error.message : String(error);
	}

	async function handleLoadModelHome() {
		const trimmed = modelHomePath.trim();
		if (!trimmed) {
			loadError = 'Enter a model home path to continue.';
			return;
		}

		isLoadingModel = true;
		loadError = '';
		try {
			const result = await invoke<ModelHomeSummary>('load_model_home', { path: trimmed });
			summary = result;
			selectedCard = null;
			loadedCard = null;
			loadStatus = `Loaded ${result.cardCount} cards from ${result.root}`;
			status = 'idle';
			validationMessage = 'Select a card to inspect its JSON payload.';
		} catch (error) {
			loadError = formatError(error);
		} finally {
			isLoadingModel = false;
		}
	}

	async function selectCard(card: CardSummary) {
		if (!summary) {
			return;
		}

		try {
			const response = await invoke<LoadedCard>('load_card_file', {
				rootPath: summary.root,
				relativePath: card.relativePath,
			});
			selectedCard = card;
			loadedCard = response;
			cardJson = response.raw;
			status = 'idle';
			validationMessage = `Loaded ${card.id} from ${response.path}`;
		} catch (error) {
			status = 'error';
			validationMessage = `Failed to load ${card.id}: ${formatError(error)}`;
		}
	}

	async function validateCard() {
		busy = true;
		try {
			const response = await invoke<ValidationReport>('validate_card_json', { cardJson });
			if (response.valid) {
				status = 'valid';
				validationMessage = 'Card passed deterministic validation checks ✅';
			} else {
				status = 'invalid';
				validationMessage = response.messages.join('\n');
			}
		} catch (error) {
			status = 'error';
			validationMessage = formatError(error);
		} finally {
			busy = false;
		}
	}
</script>

<main>
	<header class="hero">
		<div class="hero-copy">
			<h1>Aurora Viewer + Editor</h1>
			<p>Load an Aurora model home, browse mission trees, and validate cards using the shared Rust library.</p>
		</div>
		<div class="context-panel">
			<label for="model-home">Model home path</label>
			<div class="input-row">
				<input
					id="model-home"
					type="text"
					placeholder="/path/to/aurora/model"
					spellcheck="false"
					bind:value={modelHomePath}
				/>
				<button
					type="button"
					class="primary"
					on:click|preventDefault={handleLoadModelHome}
					disabled={isLoadingModel}
				>
					{isLoadingModel ? 'Loading…' : 'Load Model Home'}
				</button>
			</div>
			<p class="status-line">{loadStatus}</p>
			{#if loadError}
				<p class="warning">{loadError}</p>
			{/if}
		</div>
		<div class="metrics">
			<div class="metric">
				<p>Total Cards</p>
				<strong>{summary ? summary.cardCount : '—'}</strong>
			</div>
			<div class="metric">
				<p>Missions</p>
				<strong>{summary ? summary.missions.length : '—'}</strong>
			</div>
			<div class="metric">
				<p>Selected Card</p>
				<strong>{selectedCard ? selectedCard.id : '—'}</strong>
			</div>
		</div>
	</header>

	<section class="workspace">
		<aside class="navigator">
			<div class="panel-header">
				<h2>Model Tree</h2>
				<p>Mission → cards, sorted deterministically.</p>
			</div>
			{#if summary}
				<div class="mission-list">
					{#each summary.missions as mission}
						<div class="mission-card">
							<div class="mission-header">
								<p class="mission-id">{mission.id}</p>
								<h3>{mission.name}</h3>
								<span>{mission.cards.length} cards</span>
							</div>
							<ul>
								{#each mission.cards as card}
									<li>
										<button
											type="button"
											class:selected={selectedCard?.relativePath === card.relativePath}
											on:click={() => selectCard(card)}
										>
											<span class="card-id">{card.id}</span>
											<span class="card-name">{card.name}</span>
											<span class="card-meta">
												{card.cardType}
												{#if card.status}
													&nbsp;•&nbsp;{card.status}
												{/if}
											</span>
										</button>
									</li>
								{/each}
							</ul>
						</div>
					{/each}
				</div>
			{:else}
				<p class="placeholder">Load a model home to explore missions, requirements, and capabilities.</p>
			{/if}
		</aside>

		<section class="editor">
			<div class="editor-header">
				<div>
					<h2>{selectedCard ? selectedCard.name : 'Card JSON Editor'}</h2>
					<p>
						{#if selectedCard}
							<span>{selectedCard.id}</span>
							&nbsp;•&nbsp;
							<span>{selectedCard.cardType}</span>
							{#if selectedCard.status}
								&nbsp;•&nbsp;<span>{selectedCard.status}</span>
							{/if}
						{:else}
							Choose a card from the navigator to load its JSON payload.
						{/if}
					</p>
				</div>
				{#if loadedCard}
					<div class="path-chip">{loadedCard.path}</div>
				{/if}
			</div>

			<label for="card-json">Aurora card JSON</label>
			<textarea id="card-json" bind:value={cardJson} rows={18} spellcheck="false" class="code"></textarea>
			<div class="editor-actions">
				<button type="button" class="primary" on:click|preventDefault={validateCard} disabled={busy}>
					{busy ? 'Validating…' : 'Validate'}
				</button>
			</div>
		</section>
	</section>

	<section class={`status-panel ${status}`}>
		<h2>Validation status</h2>
		<pre>{validationMessage}</pre>
	</section>
</main>

<style>
	:global(:root) {
		--panel-bg: rgba(3, 22, 36, 0.92);
		--panel-border: rgba(111, 255, 229, 0.35);
		--accent: #5ff8de;
		--accent-strong: #1fb9ff;
		--text-muted: #a7c2d1;
		--danger: #ff9d8e;
	}

	:global(body) {
		margin: 0;
		font-family:
			'Space Grotesk',
			'IBM Plex Sans',
			'Segoe UI',
			system-ui,
			-apple-system,
			sans-serif;
		background: radial-gradient(circle at 25% 20%, rgba(32, 190, 214, 0.25), transparent 55%),
			linear-gradient(130deg, #020c13, #01060c 55%, #03101a);
		color: #f4fdff;
		min-height: 100vh;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem 1.5rem 4rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.hero {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 1.5rem;
		background: var(--panel-bg);
		border: 1px solid var(--panel-border);
		border-radius: 1.5rem;
		padding: 1.75rem;
		box-shadow: 0 30px 60px rgba(0, 0, 0, 0.35);
	}

	.hero-copy h1 {
		margin: 0 0 0.35rem;
		font-size: 2.25rem;
	}

	.hero-copy p {
		margin: 0;
		color: var(--text-muted);
	}

	.context-panel {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	label {
		font-weight: 600;
		letter-spacing: 0.02em;
	}

	.input-row {
		display: flex;
		gap: 0.75rem;
	}

	input[type='text'] {
		flex: 1;
		border-radius: 0.75rem;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: rgba(2, 11, 18, 0.8);
		color: #f6feff;
		padding: 0.75rem 1rem;
		font-size: 0.95rem;
	}

	.status-line {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.warning {
		margin: 0;
		color: var(--danger);
		font-weight: 600;
	}

	.metrics {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
		gap: 0.75rem;
	}

	.metric {
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 1rem;
	}

	.metric p {
		margin: 0;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.metric strong {
		display: block;
		margin-top: 0.25rem;
		font-size: 1.5rem;
	}

	.workspace {
		display: grid;
		grid-template-columns: 360px 1fr;
		gap: 1.5rem;
	}

	.navigator,
	.editor,
	.status-panel {
		background: var(--panel-bg);
		border: 1px solid var(--panel-border);
		border-radius: 1.5rem;
		padding: 1.5rem;
		box-shadow: 0 30px 60px rgba(0, 0, 0, 0.3);
	}

	.panel-header {
		margin-bottom: 1rem;
	}

	.panel-header h2 {
		margin: 0;
	}

	.panel-header p {
		margin: 0.25rem 0 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.mission-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		max-height: 520px;
		overflow-y: auto;
		padding-right: 0.25rem;
	}

	.mission-card {
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 1rem;
		background: rgba(2, 14, 20, 0.6);
	}

	.mission-header {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
		margin-bottom: 0.35rem;
	}

	.mission-id {
		margin: 0;
		font-size: 0.85rem;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.mission-header h3 {
		margin: 0;
		font-size: 1.1rem;
	}

	.mission-header span {
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.mission-card ul {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.mission-card button {
		width: 100%;
		text-align: left;
		border: 1px solid rgba(255, 255, 255, 0.07);
		background: rgba(4, 12, 18, 0.65);
		border-radius: 0.9rem;
		padding: 0.6rem 0.8rem;
		color: inherit;
		cursor: pointer;
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.35rem 0.6rem;
		transition:
			border-color 160ms ease,
			background 160ms ease;
		font-family: inherit;
	}

	.mission-card button .card-id {
		font-weight: 600;
	}

	.mission-card button .card-name {
		grid-column: span 2;
	}

	.mission-card button .card-meta {
		grid-column: span 2;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.mission-card button:hover,
	.mission-card button.selected {
		border-color: var(--accent);
		background: rgba(3, 40, 50, 0.85);
	}

	.placeholder {
		color: var(--text-muted);
		margin: 2rem 0 0;
	}

	.editor {
		display: flex;
		flex-direction: column;
		gap: 0.85rem;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		gap: 1rem;
		align-items: flex-start;
	}

	.editor-header h2 {
		margin: 0;
	}

	.editor-header p {
		margin: 0.2rem 0 0;
		color: var(--text-muted);
	}

	.path-chip {
		font-size: 0.85rem;
		padding: 0.4rem 0.75rem;
		border-radius: 999px;
		border: 1px solid rgba(255, 255, 255, 0.2);
		background: rgba(255, 255, 255, 0.05);
	}

	textarea {
		border-radius: 1rem;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: rgba(2, 11, 18, 0.85);
		color: #f4fdff;
		padding: 1rem;
		font-family: 'IBM Plex Mono', 'Space Mono', 'Fira Code', monospace;
		font-size: 0.9rem;
		line-height: 1.5;
		resize: vertical;
	}

	.editor-actions {
		display: flex;
		gap: 0.75rem;
	}

	button.primary {
		background: linear-gradient(110deg, var(--accent), var(--accent-strong));
		border: none;
		border-radius: 999px;
		padding: 0.75rem 1.75rem;
		color: #041a22;
		font-weight: 600;
		cursor: pointer;
		transition:
			transform 160ms ease,
			box-shadow 160ms ease;
	}

	button.primary:disabled {
		opacity: 0.6;
		cursor: progress;
	}

	button.primary:not(:disabled):hover {
		transform: translateY(-1px) scale(1.01);
		box-shadow: 0 8px 18px rgba(31, 185, 255, 0.35);
	}

	.status-panel {
		white-space: pre-wrap;
		word-break: break-word;
	}

	.status-panel h2 {
		margin-top: 0;
	}

	.status-panel pre {
		margin: 0;
		font-family: 'IBM Plex Mono', 'Space Mono', monospace;
		font-size: 0.95rem;
	}

	.status-panel.valid {
		border-color: rgba(95, 248, 222, 0.65);
	}

	.status-panel.invalid {
		border-color: rgba(255, 174, 91, 0.65);
	}

	.status-panel.error {
		border-color: rgba(255, 140, 140, 0.75);
	}

	@media (max-width: 1100px) {
		.workspace {
			grid-template-columns: 1fr;
		}

		.mission-list {
			max-height: 360px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		* {
			transition: none !important;
			animation: none !important;
		}
	}
</style>
