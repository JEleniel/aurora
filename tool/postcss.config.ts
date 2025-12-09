import type { Plugin } from 'postcss'

// PostCSS will load this file via Vite's TS support. Export a default object
// compatible with PostCSS configuration.
const config: { plugins: Record<string, Plugin | Record<string, unknown>> } = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}

export default config
