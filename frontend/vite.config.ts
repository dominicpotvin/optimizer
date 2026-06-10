import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// En dev, le serveur Vite proxifie /api vers le backend Rust (port local 8099).
// En production, le binaire Rust sert directement le build statique (meme origine).
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5199,
    proxy: {
      // En dev, on tape directement le backend (expose sur 8097 par docker compose).
      '/api': 'http://localhost:8097',
    },
  },
  build: {
    outDir: 'dist',
  },
})
