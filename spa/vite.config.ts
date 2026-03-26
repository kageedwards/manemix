import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig(({ mode }) => ({
	plugins: [sveltekit(), tailwindcss()],
	resolve: {
		conditions: mode === 'test' ? ['browser'] : []
	},
	test: {
		environment: 'happy-dom',
		include: ['src/**/*.test.ts'],
		alias: {
			'$app/navigation': '/src/lib/__mocks__/app-navigation.ts'
		}
	},
	server: {
		proxy: {
			'/api/v1': {
				target: 'http://localhost:8642',
				changeOrigin: true
			},
			'/static': {
				target: 'http://localhost:8642',
				changeOrigin: true
			},
			'/tracks/latest/atom': {
				target: 'http://localhost:8642',
				changeOrigin: true
			},
			'/tracks/featured/atom': {
				target: 'http://localhost:8642',
				changeOrigin: true
			}
		}
	}
}));
