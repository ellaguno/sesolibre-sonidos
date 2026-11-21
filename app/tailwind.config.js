/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        fondo: 'rgb(var(--fondo) / <alpha-value>)',
        panel: 'rgb(var(--panel) / <alpha-value>)',
        borde: 'rgb(var(--borde) / <alpha-value>)',
        texto: 'rgb(var(--texto) / <alpha-value>)',
        tenue: 'rgb(var(--tenue) / <alpha-value>)',
        comprobado: 'rgb(var(--comprobado) / <alpha-value>)',
        prometedor: 'rgb(var(--prometedor) / <alpha-value>)',
        exploracion: 'rgb(var(--exploracion) / <alpha-value>)',
        propio: 'rgb(var(--propio) / <alpha-value>)',
      },
      fontFamily: {
        // La de la marca en la cabecera, pariente de la de Somnia. OFL, empaquetada.
        marca: ['Quicksand', 'system-ui', 'sans-serif'],
        sans: ['system-ui', '-apple-system', 'Segoe UI', 'Roboto', 'Noto Sans', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
