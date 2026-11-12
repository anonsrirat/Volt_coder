import { Palette, Monitor, Moon, Sun, Check } from "lucide-react";
import clsx from "clsx";
import { useSettingsStore } from "../lib/store";
import { useT, LANGUAGES, type Lang } from "../lib/i18n";
import { ACCENT_PRESETS, applyTheme, type ThemeMode } from "../lib/theme";
import { updateSettings } from "../lib/ipc";

export function AppearanceTab() {
  const t = useT();
  const { theme, accent, language, setTheme, setAccent, setLanguage } = useSettingsStore();

  async function persist(patch: { theme?: ThemeMode; accent?: number; language?: string }) {
    applyTheme(
      (patch.theme ?? theme) as ThemeMode,
      patch.accent ?? accent,
    );
    try {
      await updateSettings(patch);
    } catch {
      /* non-fatal */
    }
  }

  const themes: { id: ThemeMode; icon: typeof Moon; label: string }[] = [
    { id: "dark", icon: Moon, label: t("theme.dark") },
    { id: "light", icon: Sun, label: t("theme.light") },
    { id: "system", icon: Monitor, label: t("theme.system") },
  ];

  return (
    <div className="space-y-5">
      <div className="flex items-center gap-2">
        <Palette size={15} className="accent-text" />
        <span className="label">{t("settings.appearance")}</span>
      </div>

      {/* Theme */}
      <div>
        <label className="label block mb-2">{t("settings.appearance")}</label>
        <div className="grid grid-cols-3 gap-2">
          {themes.map((th) => (
            <button
              key={th.id}
              onClick={() => {
                setTheme(th.id);
                persist({ theme: th.id });
              }}
              className={clsx(
                "flex flex-col items-center gap-2 rounded-xl border p-3 transition",
                theme === th.id ? "accent-border accent-soft-bg" : "bd hover:bd-strong",
              )}
            >
              <th.icon size={18} className={theme === th.id ? "accent-text" : "tx-muted"} />
              <span className="text-xs tx">{th.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Accent */}
      <div>
        <label className="label block mb-2">{t("theme.accent")}</label>
        <div className="flex flex-wrap gap-2">
          {ACCENT_PRESETS.map((p) => (
            <button
              key={p.name}
              onClick={() => {
                setAccent(p.hue);
                persist({ accent: p.hue });
              }}
              className={clsx(
                "group relative h-9 w-9 rounded-full transition",
                accent === p.hue && "ring-2 ring-offset-2 ring-offset-[var(--bg)]",
              )}
              style={{
                background: p.color,
                boxShadow: accent === p.hue ? `0 0 16px -2px ${p.color}` : "none",
              }}
              title={p.name}
            >
              {accent === p.hue && (
                <Check size={14} className="absolute inset-0 m-auto text-white" />
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Language */}
      <div>
        <label className="label block mb-2">{t("settings.language")}</label>
        <div className="grid grid-cols-2 gap-2">
          {LANGUAGES.map((l) => (
            <button
              key={l.id}
              onClick={() => {
                setLanguage(l.id);
                persist({ language: l.id });
              }}
              className={clsx(
                "flex items-center gap-2 rounded-lg border px-3 py-2 text-sm transition",
                language === l.id ? "accent-border accent-soft-bg tx" : "bd tx-muted hover:bd-strong",
              )}
            >
              <span className="text-base">{l.flag}</span>
              {l.label}
              {language === l.id && <Check size={14} className="ml-auto accent-text" />}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
