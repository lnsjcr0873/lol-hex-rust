/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        hex: {
          bg: '#090d1a',
          card: '#0f172a',
          cardHover: '#162238',
          subtle: '#131d33',
          border: '#1e2c47',
          borderLight: '#2a3b5c',
          cyan: '#0ac8b9',
          cyanLight: '#00efd6',
          gold: '#c8aa6e',
          goldLight: '#f0e6d2',
          goldDark: '#785a28',
          blueTeam: '#1e3a8a',
          redTeam: '#450a0a',
          win: '#3b82f6',
          loss: '#ef4444',
          textMuted: '#64748b',
          textBase: '#94a3b8',
          textBright: '#e2e8f0',
        }
      },
      fontFamily: {
        sans: ['Segoe UI', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      }
    },
  },
  plugins: [],
}
