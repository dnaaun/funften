/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs", "./node_modules/flowbite/**/*.js"],
  },
  theme: {
    extend: {},
  },
  plugins: [
    require('flowbite/plugin')
  ],
}
