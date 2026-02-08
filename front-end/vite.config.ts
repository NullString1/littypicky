import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), 'VITE_');

	return {
		plugins: [sveltekit()],
		server: {
			proxy: {
				'/api': {
					target: env.VITE_BACKEND_URL || 'http://localhost:6780',
					changeOrigin: true
				}
			}
		}
	};
});
