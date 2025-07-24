// Import shared design system configuration
const { designSystem } = require('../shared-design-config.js');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "./dist/**/*.html"
  ],
  theme: {
    extend: {
      colors: designSystem.colors,
      fontFamily: designSystem.fontFamily,
      fontSize: designSystem.fontSize,
      spacing: designSystem.spacing,
      borderRadius: designSystem.borderRadius,
      boxShadow: designSystem.boxShadow,
      transitionDuration: designSystem.transitionDuration,
      zIndex: designSystem.zIndex,
      screens: designSystem.screens,

    },
  },
  plugins: [],
}
