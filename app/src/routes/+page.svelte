<script lang="ts">
	import { themeStore, type ThemeMode } from '$lib/stores/theme';

	let currentTheme: ThemeMode;

	themeStore.subscribe((theme) => {
		currentTheme = theme;
	});
</script>

<div class="home-container">
	<h2>Welcome to AURORA</h2>
	<p>Agent-Unified Representation of Requirements and Architecture</p>

	<section class="intro-section">
		<h3>Getting Started</h3>
		<p>
			AURORA provides a unified architectural modeling framework designed for symmetric readability by human
			engineers and autonomous agents.
		</p>

		<div class="card-grid">
			<div class="info-card">
				<h4>📋 Create Cards</h4>
				<p>
					Define drivers, requirements, behaviors, interfaces, and constraints with complete metadata
					tracking.
				</p>
				<a href="/cards" class="md-button md-button--primary">Go to Cards</a>
			</div>

			<div class="info-card">
				<h4>🔗 Manage Links</h4>
				<p>Establish directional relationships between cards and external URLs for complete traceability.</p>
				<a href="/links" class="md-button md-button--primary">Go to Links</a>
			</div>

			<div class="info-card">
				<h4>👁️ Explore Views</h4>
				<p>Navigate your architecture through requirement, component, and custom views.</p>
				<a href="/views" class="md-button md-button--primary">Go to Views</a>
			</div>
		</div>
	</section>

	<section class="theme-section">
		<h3>Current Theme: <code>{currentTheme}</code></h3>
		<p>The app automatically adapts to your system preference. You can override it in the top navigation.</p>
	</section>
</div>

<style>
	.home-container {
		max-width: 1200px;
		margin: 0 auto;
	}

	h2 {
		font-size: 2rem;
		font-weight: 500;
		color: var(--md-sys-color-primary);
		margin-bottom: 0.5rem;
	}

	:global(> p) {
		font-size: 1.125rem;
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 2rem;
	}

	section {
		margin-bottom: 3rem;
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 500;
		margin-bottom: 1rem;
		color: var(--md-sys-color-on-background);
	}

	p {
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.6;
		margin-bottom: 1rem;
	}

	.card-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 1.5rem;
		margin-top: 1.5rem;
	}

	.info-card {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 1.5rem;
		transition: all 0.2s ease;
	}

	.info-card:hover {
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
	}

	.info-card h4 {
		font-size: 1.125rem;
		font-weight: 500;
		margin-bottom: 0.5rem;
		color: var(--md-sys-color-primary);
	}

	.info-card p {
		margin-bottom: 1rem;
		font-size: 0.95rem;
	}

	.info-card a {
		display: inline-block;
		text-decoration: none;
	}

	.theme-section {
		background-color: color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		padding: 1.5rem;
	}

	code {
		background-color: var(--md-sys-color-surface-variant);
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-family: 'Courier New', monospace;
		font-size: 0.875rem;
	}
</style>
