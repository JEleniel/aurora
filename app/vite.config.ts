import { defineConfig } from 'vite';
import type { UserConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import fs from 'fs';
import os from 'os';

const host: string = os.hostname();

const tlsKey = process.env['AUR_TLS_KEY'];
const tlsCert = process.env['AUR_TLS_CERT'];

const httpsConfig = tlsKey && tlsCert && fs.existsSync(tlsKey) && fs.existsSync(tlsCert)
	? {
			key: fs.readFileSync(tlsKey),
			cert: fs.readFileSync(tlsCert),
		}
	: undefined;

export default defineConfig((): UserConfig => ({
	plugins: [sveltekit()],

	clearScreen: false,
	server: {
		https: httpsConfig,
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'wss' as const,
					host,
					port: 1421,
			  }
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**'],
		},
	},
}));
