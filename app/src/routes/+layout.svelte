<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { themeStore } from '$lib/stores/theme';
	import { allCards } from '$lib/stores/architecture';
	import type { ThemeMode } from '$lib/stores/theme';
	import { invoke } from '@tauri-apps/api/core';
	import '../app.css';

	interface Props {
		children?: any;
	}

	let { children }: Props = $props();

	let currentTheme: ThemeMode = $state('auto');
	let isSavingArchitecture: boolean = $state(false);
	let saveMessage: string = $state('');
	let currentPathname: string = $state('/');

	const navItems = [
		{ href: '/', label: 'Home' },
		{ href: '/phases', label: 'Phases' },
		{ href: '/cards', label: 'Cards' },
		{ href: '/links', label: 'Links' },
		{ href: '/matrix', label: 'Traceability' },
		{ href: '/graph', label: 'Graph' },
		{ href: '/views', label: 'Views' },
		{ href: '/settings', label: 'Settings' },
	];

	$effect(() => {
		// Subscribe to page changes
		const unsubscribe = page.subscribe((p) => {
			currentPathname = p.url.pathname;
		});
		return unsubscribe;
	});

	onMount(() => {
		// Apply initial theme
		themeStore.subscribe((mode) => {
			currentTheme = mode;
		});
	});

	function toggleTheme() {
		themeStore.toggleTheme();
	}

	function isActive(href: string): boolean {
		return currentPathname === href || (currentPathname.startsWith(href) && href !== '/');
	}

	async function handleSaveArchitecture() {
		isSavingArchitecture = true;
		saveMessage = '';
		try {
			const path = await invoke<string | null>('select_save_file');
			if (path) {
				const { saveArchitecture } = await import('$lib/services/architecture');
				const result = await saveArchitecture(path);
				saveMessage = result;
				setTimeout(() => {
					saveMessage = '';
				}, 3000);
			}
		} catch (error) {
			saveMessage = error instanceof Error ? `Error: ${error.message}` : 'Error saving';
		} finally {
			isSavingArchitecture = false;
		}
	}
</script>

<div class="app-container">
	<a href="#main-content" class="skip-link">Skip to main content</a>
	<header class="app-header">
		<nav class="tabs-nav" aria-label="Main navigation">
			{#each navItems as item}
				<a
					href={item.href}
					class="tab-link"
					class:active={isActive(item.href)}
					aria-current={isActive(item.href) ? 'page' : undefined}
				>
					{item.label}
				</a>
			{/each}
			<div class="header-spacer"></div>
			{#if $allCards.length > 0}
				<button
					class="md-button md-button--secondary"
					onclick={handleSaveArchitecture}
					disabled={isSavingArchitecture}
					title="Save architecture to file"
					aria-label="Save architecture to file"
				>
					{isSavingArchitecture ? '⏳' : '💾'} Save
				</button>
			{/if}
			<button
				class="md-button md-button--text theme-toggle"
				onclick={toggleTheme}
				title="Toggle between light, dark, and auto theme"
				aria-label="Toggle between light, dark, and auto theme"
			>
				{#if currentTheme === 'light'}
					☀️
				{:else if currentTheme === 'dark'}
					🌙
				{:else}
					🔄
				{/if}
			</button>
		</nav>
		{#if saveMessage}
			<div
				class="save-notification"
				class:error={saveMessage.startsWith('Error')}
				role="status"
				aria-live="polite"
				aria-atomic="true"
			>
				{saveMessage}
			</div>
		{/if}
	</header>

	<main class="app-content" id="main-content">
		{@render children?.()}
	</main>
</div>

<style>
	.skip-link {
		position: absolute;
		top: -40px;
		left: 0;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		padding: 8px;
		text-decoration: none;
		z-index: 9999;
		border-radius: 0 0 4px 0;
		font-weight: 500;
	}

	.skip-link:focus {
		top: 0;
	}

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

	.tabs-nav {
		display: flex;
		gap: 0;
		padding: 0 1rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		overflow-x: auto;
		align-items: center;
	}

	.tab-link {
		padding: 0.75rem 1.5rem;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		transition: all 0.2s ease;
		white-space: nowrap;
		border-bottom: 3px solid transparent;
		display: flex;
		align-items: center;

		&:hover {
			color: var(--md-sys-color-on-surface);
			background-color: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
		}

		&:focus {
			outline: 2px solid var(--md-sys-color-primary);
			outline-offset: -2px;
		}

		&.active {
			color: var(--md-sys-color-primary);
			border-bottom-color: var(--md-sys-color-primary);
		}
	}

	.theme-toggle {
		padding: 0.75rem 1.5rem;
		border-bottom: 3px solid transparent;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.2rem;
		background: none;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;

		&:hover {
			color: var(--md-sys-color-on-surface);
			background-color: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
		}

		&:focus {
			outline: 2px solid var(--md-sys-color-primary);
			outline-offset: -2px;
		}
	}

	.header-spacer {
		flex: 1;
	}

	.save-notification {
		padding: 0.75rem 1rem;
		background-color: color-mix(in srgb, #4caf50 12%, transparent);
		border-bottom: 2px solid #4caf50;
		color: #2e7d32;
		font-size: 0.9rem;
		text-align: center;
		animation: slideDown 0.3s ease;
	}

	.save-notification.error {
		background-color: color-mix(in srgb, var(--md-sys-color-error) 12%, transparent);
		border-bottom-color: var(--md-sys-color-error);
		color: var(--md-sys-color-error);
	}

	@keyframes slideDown {
		from {
			opacity: 0;
			transform: translateY(-10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.app-content {
		flex: 1;
		overflow-y: auto;
		padding: 1.5rem 2rem;
		background-color: var(--md-sys-color-background);

		&:focus {
			outline: none;
		}
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
