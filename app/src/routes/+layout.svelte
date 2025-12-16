<script lang="ts">
	import { onMount } from 'svelte';
	import { themeStore } from '$lib/stores/theme';
	import type { ThemeMode } from '$lib/stores/theme';
	import '$lib/styles/material3.css';

	let currentTheme: ThemeMode = 'auto';

	onMount(() => {
		// Apply initial theme
		themeStore.subscribe((mode) => {
			currentTheme = mode;
		});
	});

	function toggleTheme() {
		themeStore.toggleTheme();
	}
</script>

<div class="app-container">
	<header class="app-header">
		<div class="header-content">
			<h1>AURORA</h1>
			<nav class="nav-secondary">
				<button class="md-button md-button--text" on:click={toggleTheme} title="Toggle theme">
					{#if currentTheme === 'light'}
						☀️
					{:else if currentTheme === 'dark'}
						🌙
					{:else}
						🔄
					{/if}
					<span>{currentTheme}</span>
				</button>
			</nav>
		</div>
	</header>

	<div class="app-main">
		<aside class="app-sidebar">
			<nav class="sidebar-nav">
				<ul>
					<li><a href="/" class="nav-item">Dashboard</a></li>
					<li><a href="/cards" class="nav-item">Cards</a></li>
					<li><a href="/links" class="nav-item">Links</a></li>
					<li><a href="/matrix" class="nav-item">Traceability Matrix</a></li>
					<li><a href="/graph" class="nav-item">Dependency Graph</a></li>
					<li><a href="/views" class="nav-item">Views</a></li>
					<li><a href="/settings" class="nav-item">Settings</a></li>
				</ul>
			</nav>
		</aside>

		<main class="app-content">
			<slot />
		</main>
	</div>
</div>

<style>
	.app-container {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background-color: var(--md-sys-color-background);
		color: var(--md-sys-color-on-background);
	}

	.app-header {
		background-color: var(--md-sys-color-surface);
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		z-index: 100;
	}

	.header-content {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem 1.5rem;
		max-width: 100%;
	}

	.app-header h1 {
		font-size: 1.5rem;
		font-weight: 500;
		margin: 0;
		color: var(--md-sys-color-primary);
	}

	.nav-secondary {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.app-main {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.app-sidebar {
		width: 280px;
		background-color: var(--md-sys-color-surface);
		border-right: 1px solid var(--md-sys-color-outline-variant);
		overflow-y: auto;
		padding: 1rem 0;
	}

	.sidebar-nav ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.sidebar-nav li {
		margin: 0;
	}

	.nav-item {
		display: block;
		padding: 0.75rem 1.5rem;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition: all 0.2s ease;

		&:hover {
			background-color: color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
			color: var(--md-sys-color-primary);
		}

		&:active {
			background-color: color-mix(in srgb, var(--md-sys-color-primary) 16%, transparent);
		}
	}

	.app-content {
		flex: 1;
		overflow-y: auto;
		padding: 2rem;
		background-color: var(--md-sys-color-background);
	}

	/* Scrollbar styling */
	::-webkit-scrollbar {
		width: 8px;
		height: 8px;
	}

	::-webkit-scrollbar-track {
		background: transparent;
	}

	::-webkit-scrollbar-thumb {
		background-color: var(--md-sys-color-outline-variant);
		border-radius: 4px;

		&:hover {
			background-color: var(--md-sys-color-outline);
		}
	}
</style>
