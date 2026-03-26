/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {}
  },
  plugins: [],
  // daisyUI themes are configured via @plugin directives in src/app.css (Tailwind v4 / daisyUI v5)
  // Theme definitions:
  //   manemix_dark  — dark purple palette, default theme
  //   manemix_light — light purple palette, toggle variant
  daisyui: {
    themes: [
      {
        manemix_dark: {
          'primary': '#9b7abf',
          'primary-content': '#ffffff',
          'secondary': '#7c5fa8',
          'secondary-content': '#ffffff',
          'accent': '#b89edb',
          'accent-content': '#1a1625',
          'neutral': '#2a2438',
          'neutral-content': '#e0d8ec',
          'base-100': '#1a1625',
          'base-200': '#221d30',
          'base-300': '#2a2438',
          'base-content': '#e0d8ec',
          'info': '#7cb3d4',
          'success': '#7bc47f',
          'warning': '#d4a74c',
          'error': '#d46b6b'
        }
      },
      {
        manemix_light: {
          'primary': '#7c5fa8',
          'primary-content': '#ffffff',
          'secondary': '#9b7abf',
          'secondary-content': '#ffffff',
          'accent': '#6b4d94',
          'accent-content': '#ffffff',
          'neutral': '#f5f0fa',
          'neutral-content': '#2a2438',
          'base-100': '#ffffff',
          'base-200': '#f8f5fc',
          'base-300': '#f0ebf5',
          'base-content': '#2a2438',
          'info': '#4a90b8',
          'success': '#4a9e4f',
          'warning': '#b8892a',
          'error': '#b84a4a'
        }
      }
    ]
  }
};
