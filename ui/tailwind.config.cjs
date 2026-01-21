/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        bg: "#0c1020",
        "bg-soft": "#121a2f",
        "bg-deep": "#0a0e1a",
        panel: "#141f38",
        line: "#223156",
        accent: "#ffb454",
        accent2: "#55d6be",
        accent3: "#ef476f",
        text: "#f7f3e9",
        muted: "#a8b2d1"
      },
      fontFamily: {
        sans: ['"Space Grotesk"', "Segoe UI", "system-ui", "sans-serif"],
        serif: ['"Fraunces"', "serif"]
      },
      boxShadow: {
        card: "0 20px 40px rgba(0, 0, 0, 0.3)",
        panel: "0 18px 36px rgba(0, 0, 0, 0.25)"
      },
      backgroundImage: {
        "base-gradient": "radial-gradient(circle at top left, #1b2547 0%, #0c1020 45%, #0a0e1a 100%)",
        "card-gradient":
          "linear-gradient(135deg, rgba(20, 31, 56, 0.95), rgba(19, 27, 44, 0.9))"
      }
    }
  },
  plugins: []
};
