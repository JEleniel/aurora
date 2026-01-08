import { defineConfig } from 'vite';
import type { UserConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import fs from 'fs';
import os from 'os';
import tailwindcss from '@tailwindcss/vite';

const host: string = os.hostname();
process.env['TAURI_DEV_URL'] = `http://${host}:1420`;

const tlsKey = process.env['AUR_TLS_KEY'];
const tlsCert = process.env['AUR_TLS_CERT'];

const httpsConfig =
	tlsKey && tlsCert && fs.existsSync(tlsKey) && fs.existsSync(tlsCert)
		? {
				key: fs.readFileSync(tlsKey),
				cert: fs.readFileSync(tlsCert),
		  }
		: undefined;

export default defineConfig(
	(): UserConfig => ({
		plugins: [tailwindcss(), sveltekit()],

		clearScreen: false,
		server: {
			https: httpsConfig,
			port: 1420,
			strictPort: true,
			host: host || false,
			hmr: host
				? {
						protocol: httpsConfig ? ('wss' as const) : ('ws' as const),
						host,
						port: 1420,
				  }
				: undefined,
			watch: {
				ignored: ['**/src-tauri/**'],
			},
		},
	}),
);
