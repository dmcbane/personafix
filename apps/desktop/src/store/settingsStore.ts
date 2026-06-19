import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Theme =
  | "cyber"
  | "high-contrast"
  | "deuteranopia"
  | "tritanopia"
  | "light";

export type AppFont = "mono" | "sans" | "opendyslexic";

export type FontSize = "sm" | "md" | "lg" | "xl";

export const FONT_SIZE_PX: Record<FontSize, string> = {
  sm: "13px",
  md: "15px",
  lg: "17px",
  xl: "20px",
};

interface SettingsState {
  theme: Theme;
  font: AppFont;
  fontSize: FontSize;
  setTheme: (theme: Theme) => void;
  setFont: (font: AppFont) => void;
  setFontSize: (size: FontSize) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      theme: "cyber",
      font: "mono",
      fontSize: "md",
      setTheme: (theme) => set({ theme }),
      setFont: (font) => set({ font }),
      setFontSize: (fontSize) => set({ fontSize }),
    }),
    { name: "personafix-settings" },
  ),
);
