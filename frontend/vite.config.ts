import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		sveltekit(),
		{
			name: 'suppress-duckdb-sourcemap-warnings',
			configureServer(server) {
				const warn = server.config.logger.warn;
				server.config.logger.warn = (msg, options) => {
					if (msg.includes('@duckdb') || msg.includes('apache-arrow')) return;
					warn(msg, options);
				};
			}
		}
	],
	server: {
		host: '0.0.0.0',
		port: 5174,
		proxy: {
			'/api': {
				target: 'http://localhost:3001',
				changeOrigin: true
			}
		}
	},
	build: {
		rollupOptions: {
			onwarn(warning, warn) {
				if (warning.code === 'SOURCEMAP_BROKEN') return;
				warn(warning);
			}
		}
	}
});
