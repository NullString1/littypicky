import adapter from '@sveltejs/adapter-cloudflare';

const config = {
	kit: {
		adapter: adapter({
			routes: {
				exclude: ['/api', '/api/*']
			}
		})
	}
};

export default config;
