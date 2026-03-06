import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
    plugins: [react()],
    base: '/framework-documentation/',
    build: {
        rollupOptions: {
            output: {
                manualChunks(id) {
                    if (id.includes('node_modules/react') || id.includes('node_modules/react-dom')) {
                        return 'react'
                    }

                    if (id.includes('node_modules/prismjs')) {
                        return 'prism'
                    }

                    if (
                        id.includes('node_modules/clsx') ||
                        id.includes('node_modules/lucide-react') ||
                        id.includes('node_modules/marked') ||
                        id.includes('node_modules/tailwind-merge')
                    ) {
                        return 'vendor'
                    }

                    if (id.includes('/src/pages/getting-started/')) {
                        return 'docs-getting-started'
                    }

                    if (id.includes('/src/pages/framework-features/') || id.includes('/src/pages/async/')) {
                        return 'docs-features'
                    }

                    if (id.includes('/src/pages/http/') || id.includes('/src/pages/security/')) {
                        return 'docs-http-security'
                    }

                    if (id.includes('/src/pages/database/')) {
                        return 'docs-database'
                    }

                    if (id.includes('/src/pages/cookbook/')) {
                        return 'docs-cookbook'
                    }
                },
            },
        },
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
})
