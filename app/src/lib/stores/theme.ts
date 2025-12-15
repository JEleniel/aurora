import { writable } from 'svelte/store';

export type ThemeMode = 'light' | 'dark' | 'auto';

function applyTheme(theme: 'light' | 'dark') {
	const html = document.documentElement;
	if (theme === 'dark') {
		html.classList.add('dark');
		html.style.colorScheme = 'dark';
	} else {
		html.classList.remove('dark');
		html.style.colorScheme = 'light';
	}
}

function createThemeStore() {
	// Get initial theme from localStorage or system preference
	const getInitialTheme = (): ThemeMode => {
		if (typeof window === 'undefined') return 'auto';

		const stored = localStorage.getItem('theme') as ThemeMode | null;
		if (stored && ['light', 'dark', 'auto'].includes(stored)) {
			return stored;
		}
		return 'auto';
	};

	const getSystemPreference = (): 'light' | 'dark' => {
		if (typeof window === 'undefined') return 'light';
		return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
	};

	const getEffectiveTheme = (mode: ThemeMode): 'light' | 'dark' => {
		return mode === 'auto' ? getSystemPreference() : mode;
	};

	const { subscribe, set } = writable<ThemeMode>(getInitialTheme());

	return {
		subscribe,
		setTheme: (mode: ThemeMode) => {
			localStorage.setItem('theme', mode);
			set(mode);
			applyTheme(getEffectiveTheme(mode));
		},
		toggleTheme: () => {
			const currentMode = getInitialTheme();
			const nextMode: ThemeMode = currentMode === 'light' ? 'dark' : currentMode === 'dark' ? 'auto' : 'light';
			localStorage.setItem('theme', nextMode);
			set(nextMode);
			applyTheme(getEffectiveTheme(nextMode));
		},
	};
}

export const themeStore = createThemeStore();

// Watch system preference changes when in auto mode
if (typeof window !== 'undefined') {
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	let currentMode: ThemeMode = 'auto';

	themeStore.subscribe((mode) => {
		currentMode = mode;
	});

	mediaQuery.addListener((e) => {
		if (currentMode === 'auto') {
			applyTheme(e.matches ? 'dark' : 'light');
		}
	});
}
