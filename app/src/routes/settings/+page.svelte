<script lang="ts">
	import { themeStore, type ThemeMode } from '$lib/stores/theme';
	import type { PageData } from './$types';

	export const data: PageData = {};

	let currentTheme: ThemeMode;

	themeStore.subscribe((theme) => {
		currentTheme = theme;
	});

	const themeOptions: Array<{ value: ThemeMode; label: string }> = [
		{ value: 'light', label: '☀️ Light' },
		{ value: 'dark', label: '🌙 Dark' },
		{ value: 'auto', label: '🔄 Auto (System)' },
	];
</script>

<div class="page">
	<header class="page-header">
		<div class="header-content">
			<h1>Settings</h1>
			<p>Configure AURORA preferences and behavior</p>
		</div>
	</header>

	<section class="settings-container">
		<div class="settings-group">
			<h2>Theme</h2>
			<div class="setting-item">
				<div class="setting-label">
					<span class="label-title">Appearance</span>
					<span class="label-description">Choose how AURORA looks</span>
				</div>
				<div class="theme-selector">
					{#each themeOptions as option}
						<button
							class="theme-button"
							class:active={currentTheme === option.value}
							onclick={() => {
								if (option.value !== currentTheme) {
									themeStore.setTheme(option.value);
								}
							}}
						>
							{option.label}
						</button>
					{/each}
				</div>
			</div>

			<div class="setting-item">
				<div class="setting-label">
					<span class="label-title">Current Theme</span>
					<span class="label-description">Currently using {currentTheme} mode</span>
				</div>
			</div>
		</div>

		<div class="settings-group">
			<h2>About</h2>
			<div class="about-content">
				<p>
					<strong>AURORA</strong> - Agent-Unified Representation Of Requirements And Architecture
				</p>
				<p>
					A specification and reference tool for architectural practice combining MBSE with machine-agent
					compatibility.
				</p>
				<p class="version-info">
					Version: 1.0.0<br />
					License: GPL-3.0-or-later<br />
					<a href="https://github.com/jeleniel/aurora" target="_blank" rel="noreferrer">GitHub Repository</a>
				</p>
			</div>
		</div>
	</section>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
	}

	.page-header {
		background: linear-gradient(135deg, var(--md-sys-color-primary) 0%, var(--md-sys-color-secondary) 100%);
		color: var(--md-sys-color-on-primary);
		padding: 2rem;
		border-radius: 12px;
		margin-bottom: 2rem;
	}

	.header-content h1 {
		font-size: 2rem;
		font-weight: 500;
		margin: 0 0 0.5rem 0;
	}

	.header-content p {
		margin: 0;
		opacity: 0.9;
	}

	.settings-container {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		max-width: 800px;
	}

	.settings-group {
		background-color: var(--md-sys-color-surface-container);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 1.5rem;
	}

	.settings-group h2 {
		font-size: 1.25rem;
		font-weight: 500;
		margin: 0 0 1rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.setting-item {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: 1rem 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}

	.setting-item:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.setting-label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.label-title {
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
	}

	.label-description {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.theme-selector {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.theme-button {
		background-color: var(--md-sys-color-surface);
		border: 2px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		padding: 0.75rem 1rem;
		cursor: pointer;
		transition: all 0.2s ease;
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9rem;
	}

	.theme-button:hover {
		border-color: var(--md-sys-color-primary);
		background-color: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}

	.theme-button.active {
		background-color: var(--md-sys-color-primary);
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}

	.about-content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.about-content p {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.6;
	}

	.about-content strong {
		color: var(--md-sys-color-primary);
	}

	.version-info {
		font-size: 0.9rem;
		white-space: pre-line;
	}

	.version-info a {
		color: var(--md-sys-color-primary);
		text-decoration: none;
	}

	.version-info a:hover {
		text-decoration: underline;
	}
</style>
