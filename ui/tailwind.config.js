/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class", '[data-theme="dark"]'],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        volt: {
          50: "#f2f0ff",
          100: "#e8e4ff",
          200: "#d4ccff",
          300: "#b5a6ff",
          400: "#9271ff",
          500: "#7c4dff",
          600: "#6a2ff5",
          700: "#5a1fd8",
          800: "#4a1bb0",
          900: "#3e1a8e",
        },
        ink: {
          900: "#0a0a0f",
          850: "#0f0f16",
          800: "#14141d",
          750: "#1a1a26",
          700: "#22222f",
          600: "#2e2e3d",
        },
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Cascadia Code", "Consolas", "monospace"],
      },
      boxShadow: {
        glow: "0 0 20px -4px hsl(var(--accent-h) 100% 65% / 0.45)",
        "glow-sm": "0 0 12px -2px hsl(var(--accent-h) 100% 65% / 0.35)",
      },
      keyframes: {
        "fade-in": {
          from: { opacity: "0", transform: "translateY(4px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        "fade-in": "fade-in 0.2s ease-out",
      },
    },
  },
  plugins: [],
};
