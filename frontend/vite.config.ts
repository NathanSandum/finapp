import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:3000',
				changeOrigin: true,
				rewrite: (path) => path.replace(/^\/api/, ''),
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				configure: (proxy: any) => {
					// eslint-disable-next-line @typescript-eslint/no-explicit-any
					proxy.on('error', (err: Error, _req: any, res: any) => {
						if (!res.headersSent) {
							res.writeHead(503, { 'Content-Type': 'application/json' });
						}
						res.end(
							JSON.stringify({
								status: 'error',
								message: `backend unreachable: ${err.message}`,
								code: 'BACKEND_DOWN'
							})
						);
					});
				}
			}
		}
	}
});
