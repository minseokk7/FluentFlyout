/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./index.html', './src/**/*.{svelte,ts}'],
  theme: {
    extend: {
      colors: {
        fluent: {
          bg: '#1d1f1d',
          titlebar: '#3a3a3a',
          panel: '#282a28',
          hover: '#303230',
          stroke: 'rgba(255,255,255,0.095)'
        }
      },
      fontFamily: {
        ui: ['Segoe UI Variable', 'Segoe UI', 'system-ui', 'sans-serif']
      },
      borderRadius: {
        fluent: '8px'
      }
    }
  },
  plugins: []
};
