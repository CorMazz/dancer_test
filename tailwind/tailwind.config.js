/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["../templates/**/*.html"],
  theme: {
    extend: {
      colors: {
        "primary": {
          "50": "#e9fbf5",
          "100": "#d4f6ea",
          "200": "#a8edd5",
          "300": "#7de4c0",
          "400": "#51dbab",
          "500": "#26d296",
          "600": "#1ea878",
          "700": "#177e5a",
          "800": "#0f543c",
          "900": "#082a1e"
        },
        "secondary": {
          "50": "#fbeaf0",
          "100": "#f7d5e1",
          "200": "#f0abc3",
          "300": "#e881a5",
          "400": "#e15787",
          "500": "#d92d69",
          "600": "#ae2454",
          "700": "#821b3f",
          "800": "#57122a",
          "900": "#2b0915"
        },
      },
    },
  },
  plugins: [
    require('flowbite/plugin')
  ]
}

