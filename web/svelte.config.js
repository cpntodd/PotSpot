import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      fallback: 'index.html',
      pages: 'build',
      assets: 'build',
      precompress: true,
    }),

    alias: {
      $lib: 'src/lib',
      $components: 'src/lib/components',
    },

    // Content Security Policy
    csp: {
      directives: {
        'default-src': ['self'],
        'img-src': ['self', 'data:', 'https:'],
        'style-src': ['self', 'unsafe-inline', 'https://fonts.googleapis.com'],
        'font-src': ['self', 'https://fonts.gstatic.com'],
        'script-src': ['self'],
        'connect-src': ['self', 'http://localhost:3000'],
      },
    },
  },
};

export default config;
