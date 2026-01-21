<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	type ValidationReport = {
		valid: boolean;
		messages: string[];
	};

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
	let message = 'Paste or edit an Aurora card JSON payload, then select "Validate" to run the shared Rust library.';
	let busy = false;

	async function validateCard() {
		busy = true;
		try {
			const response = await invoke<ValidationReport>('validate_card_json', { cardJson });
			if (response.valid) {
				status = 'valid';
				message = 'Card passed deterministic validation checks ✅';
			} else {
				status = 'invalid';
				message = response.messages.join('\n');
			}
		} catch (error) {
			status = 'error';
			message = error instanceof Error ? error.message : String(error);
		} finally {
			busy = false;
		}
	}
</script>

<main>
	<section>
		<header>
			<h1>Aurora Editor Preview</h1>
			<p>
				The UI calls a shared Rust library to ensure schema alignment and deterministic invariants before
				persisting card updates.
			</p>
		</header>
	</section>

	<section class="editor">
		<label for="card-json">Aurora card JSON</label>
		<textarea id="card-json" bind:value={cardJson} class="code" spellcheck="false" rows={20}></textarea>
		<button class="primary" on:click|preventDefault={validateCard} disabled={busy}>
			{busy ? 'Validating…' : 'Validate'}
		</button>
	</section>

	<section class={`status ${status}`}>
		<h2>Validation status</h2>
		<pre>{message}</pre>
	</section>
</main>

<style>
	:global(body) {
		margin: 0;
		font-family:
			'Inter',
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			sans-serif;
		background: radial-gradient(circle at top, #0f172a, #020617 70%);
		min-height: 100vh;
		color: #f8fafc;
	}

	main {
		max-width: 960px;
		margin: 0 auto;
		padding: 2rem 1.5rem 4rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	section {
		background: rgba(15, 23, 42, 0.65);
		border: 1px solid rgba(148, 163, 184, 0.2);
		border-radius: 1rem;
		padding: 1.5rem;
		box-shadow: 0 20px 45px rgba(2, 6, 23, 0.6);
	}

	.editor {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	label {
		font-weight: 600;
	}

	textarea {
		border-radius: 0.75rem;
		border: 1px solid rgba(148, 163, 184, 0.35);
		background: rgba(2, 6, 23, 0.9);
		color: #e2e8f0;
		padding: 1rem;
		font-size: 0.9rem;
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		resize: vertical;
		line-height: 1.45;
	}

	button.primary {
		align-self: flex-start;
		background: linear-gradient(120deg, #34d399, #22d3ee);
		border: none;
		border-radius: 2rem;
		padding: 0.75rem 1.75rem;
		font-weight: 600;
		color: #02131d;
		cursor: pointer;
		transition:
			transform 120ms ease,
			opacity 120ms ease;
	}

	button.primary:disabled {
		opacity: 0.65;
		cursor: progress;
	}

	button.primary:not(:disabled):hover {
		transform: translateY(-1px) scale(1.01);
	}

	.status pre {
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.status.valid {
		border-color: rgba(52, 211, 153, 0.45);
	}

	.status.invalid {
		border-color: rgba(248, 113, 113, 0.45);
	}

	.status.error {
		border-color: rgba(251, 191, 36, 0.5);
	}

	pre {
		margin: 0;
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		font-size: 0.95rem;
	}
</style>
