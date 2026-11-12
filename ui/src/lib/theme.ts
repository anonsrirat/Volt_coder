// Theme + appearance: applies theme mode (dark/light/system) and accent hue
// as CSS custom properties on :root.

export type ThemeMode = "dark" | "light" | "system";

/** Apply the theme mode + accent to the document root. */
export function applyTheme(mode: ThemeMode, accentHue: number) {
  const root = document.documentElement;

  const isDark =
    mode === "dark" ||
    (mode === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  root.classList.toggle("dark", isDark);
  root.dataset.theme = isDark ? "dark" : "light";
  root.style.setProperty("--accent-h", String(accentHue));
}

/** Watch the OS colour scheme and re-apply when in "system" mode. */
export function watchSystemTheme(getMode: () => ThemeMode, getAccent: () => number) {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    if (getMode() === "system") applyTheme("system", getAccent());
  };
  mq.addEventListener("change", handler);
  return () => mq.removeEventListener("change", handler);
}

export const ACCENT_PRESETS = [
  { name: "Violet", hue: 262, color: "#7c4dff" },
  { name: "Cyan", hue: 190, color: "#22d3ee" },
  { name: "Emerald", hue: 160, color: "#10b981" },
  { name: "Amber", hue: 38, color: "#f59e0b" },
  { name: "Rose", hue: 346, color: "#f43f5e" },
  { name: "Indigo", hue: 234, color: "#6366f1" },
];
