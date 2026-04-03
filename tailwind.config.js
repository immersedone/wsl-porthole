/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,vue}"],
  theme: {
    extend: {
      colors: {
        // Theme token references — actual values injected via CSS variables
        primary: "var(--bg-primary)",
        secondary: "var(--bg-secondary)",
        tertiary: "var(--bg-tertiary)",
        accent: "var(--accent)",
        "accent-dim": "var(--accent-dim)",
        "text-primary": "var(--text-primary)",
        "text-secondary": "var(--text-secondary)",
        "status-ok": "var(--status-ok)",
        "status-warn": "var(--status-warn)",
        "status-err": "var(--status-err)",
        border: "var(--border)",
      },
    },
  },
  plugins: [],
};
