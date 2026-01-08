import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				// Material Design 3 Light Theme
				'md-primary': 'var(--md-sys-color-primary, #6750a4)',
				'md-on-primary': 'var(--md-sys-color-on-primary, #ffffff)',
				'md-primary-container': 'var(--md-sys-color-primary-container, #eaddff)',
				'md-on-primary-container':
					'var(--md-sys-color-on-primary-container, #21005d)',
				'md-secondary': 'var(--md-sys-color-secondary, #625b71)',
				'md-on-secondary': 'var(--md-sys-color-on-secondary, #ffffff)',
				'md-secondary-container':
					'var(--md-sys-color-secondary-container, #e8def8)',
				'md-on-secondary-container':
					'var(--md-sys-color-on-secondary-container, #1d192b)',
				'md-tertiary': 'var(--md-sys-color-tertiary, #7d5260)',
				'md-on-tertiary': 'var(--md-sys-color-on-tertiary, #ffffff)',
				'md-tertiary-container': 'var(--md-sys-color-tertiary-container, #ffd8e4)',
				'md-on-tertiary-container':
					'var(--md-sys-color-on-tertiary-container, #31111d)',
				'md-error': 'var(--md-sys-color-error, #b3261e)',
				'md-on-error': 'var(--md-sys-color-on-error, #ffffff)',
				'md-error-container': 'var(--md-sys-color-error-container, #f9dedc)',
				'md-on-error-container': 'var(--md-sys-color-on-error-container, #410e0b)',
				'md-background': 'var(--md-sys-color-background, #fffbfe)',
				'md-on-background': 'var(--md-sys-color-on-background, #1c1b1f)',
				'md-surface': 'var(--md-sys-color-surface, #fffbfe)',
				'md-on-surface': 'var(--md-sys-color-on-surface, #1c1b1f)',
				'md-surface-variant': 'var(--md-sys-color-surface-variant, #e7e0ec)',
				'md-on-surface-variant': 'var(--md-sys-color-on-surface-variant, #49454e)',
				'md-outline': 'var(--md-sys-color-outline, #79747e)',
				'md-outline-variant': 'var(--md-sys-color-outline-variant, #c4c7c5)',
				'md-shadow': 'var(--md-sys-color-shadow, #000000)',
				'md-inverse-surface': 'var(--md-sys-color-inverse-surface, #313033)',
				'md-inverse-on-surface':
					'var(--md-sys-color-inverse-on-surface, #f4eff4)',
				'md-inverse-primary': 'var(--md-sys-color-inverse-primary, #d0bcff)',
				'md-scrim': 'var(--md-sys-color-scrim, #000000)',
			},
			fontFamily: {
				serif: ["'Noto Serif'", 'serif'],
				sans: ["'Noto Sans'", '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
				mono: ["'Noto Sans Mono'", 'monospace'],
				antonio: ["'Antonio'", 'sans-serif'],
			},
			fontSize: {
				// Material Design 3 scale with 20% increase
				'display-large': ['2.4rem', { lineHeight: '1.2', letterSpacing: '-0.015625em' }],
				'display-medium': ['2.16rem', { lineHeight: '1.2', letterSpacing: '-0.0078125em' }],
				'display-small': ['1.92rem', { lineHeight: '1.25', letterSpacing: '0em' }],
				'headline-large': ['1.8rem', { lineHeight: '1.25', letterSpacing: '0em' }],
				'headline-medium': ['1.56rem', { lineHeight: '1.5', letterSpacing: '0em' }],
				'headline-small': ['1.32rem', { lineHeight: '1.5', letterSpacing: '0em' }],
				'title-large': ['1.32rem', { lineHeight: '1.5', letterSpacing: '0.015em' }],
				'title-medium': ['1.08rem', { lineHeight: '1.5', letterSpacing: '0.015em' }],
				'title-small': ['0.96rem', { lineHeight: '1.5', letterSpacing: '0.01em' }],
				'body-large': ['1rem', { lineHeight: '1.5', letterSpacing: '0.5px' }],
				'body-medium': ['0.875rem', { lineHeight: '1.43', letterSpacing: '0.25px' }],
				'body-small': ['0.75rem', { lineHeight: '1.33', letterSpacing: '0.4px' }],
				'label-large': ['0.875rem', { lineHeight: '1.43', letterSpacing: '0.1px' }],
				'label-medium': ['0.75rem', { lineHeight: '1.33', letterSpacing: '0.5px' }],
				'label-small': ['0.6875rem', { lineHeight: '1.26', letterSpacing: '0.5px' }],
			},
			borderRadius: {
				md3: '24px',
				'md3-lg': '28px',
			},
			spacing: {
				md3: '1rem',
				'md3-lg': '1.5rem',
				'md3-xl': '2rem',
			},
			boxShadow: {
				'md3-1': '0px 1px 3px 1px rgba(0, 0, 0, 0.15), 0px 1px 2px 0px rgba(0, 0, 0, 0.3)',
				'md3-2': '0px 3px 6px 0px rgba(0, 0, 0, 0.15), 0px 2px 4px 0px rgba(0, 0, 0, 0.3)',
				'md3-3': '0px 6px 10px 0px rgba(0, 0, 0, 0.15), 0px 2px 4px 0px rgba(0, 0, 0, 0.3)',
				'md3-4': '0px 8px 12px 0px rgba(0, 0, 0, 0.15), 0px 4px 8px 0px rgba(0, 0, 0, 0.3)',
				'md3-5': '0px 12px 16px 0px rgba(0, 0, 0, 0.15), 0px 4px 8px 0px rgba(0, 0, 0, 0.3)',
			},
		},
	},
	plugins: [],
	corePlugins: {
		preflight: false,
	},
} satisfies Config;
