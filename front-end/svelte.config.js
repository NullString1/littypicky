import adapter from '@sveltejs/adapter-cloudflare';

const config = {
	kit: {
		adapter: adapter({
			routes: {
					exclude: ['/api/*'] 
			}
		})
	}
};

export default config;
