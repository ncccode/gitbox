import { defineStore } from "pinia";
import type { ChangeSide } from "../types/gitbox";

export type LayoutPanelKey = "project" | "changes";
export type ThemeMode = "system" | "dark" | "light";
export type DirectWorktreeCommitPolicy = "ask" | "always" | "never";

const STORAGE_KEY = "gitbox.settings";

const DEFAULT_PANEL_WIDTHS: Record<LayoutPanelKey, number> = {
  project: 320,
  changes: 390,
};

const DEFAULT_PANEL_VISIBILITY: Record<LayoutPanelKey, boolean> = {
  project: true,
  changes: true,
};

const PANEL_WIDTH_LIMITS: Record<LayoutPanelKey, { min: number; max: number }> = {
  project: { min: 260, max: 460 },
  changes: { min: 260, max: 680 },
};

interface StoredSettings {
  includeIgnored: boolean;
  selectedSide: ChangeSide;
  compactMode: boolean;
  themeMode: ThemeMode;
  directWorktreeCommitPolicy: DirectWorktreeCommitPolicy;
  projectPaneCollapsed: boolean;
  panelWidths: Record<LayoutPanelKey, number>;
  panelVisibility: Record<LayoutPanelKey, boolean>;
}

function isChangeSide(value: unknown): value is ChangeSide {
  return value === "unstaged" || value === "staged";
}

function isThemeMode(value: unknown): value is ThemeMode {
  return value === "system" || value === "dark" || value === "light";
}

function isDirectWorktreeCommitPolicy(value: unknown): value is DirectWorktreeCommitPolicy {
  return value === "ask" || value === "always" || value === "never";
}

function clampPanelWidth(panel: LayoutPanelKey, width: number) {
  const limits = PANEL_WIDTH_LIMITS[panel];
  return Math.min(limits.max, Math.max(limits.min, Math.round(width)));
}

function readStoredSettings(): StoredSettings {
  const defaults: StoredSettings = {
    includeIgnored: false,
    selectedSide: "unstaged",
    compactMode: true,
    themeMode: "system",
    directWorktreeCommitPolicy: "ask",
    projectPaneCollapsed: false,
    panelWidths: { ...DEFAULT_PANEL_WIDTHS },
    panelVisibility: { ...DEFAULT_PANEL_VISIBILITY },
  };

  if (typeof window === "undefined") return defaults;

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaults;

    const parsed = JSON.parse(raw) as Partial<StoredSettings>;
    return {
      includeIgnored:
        typeof parsed.includeIgnored === "boolean"
          ? parsed.includeIgnored
          : defaults.includeIgnored,
      selectedSide: isChangeSide(parsed.selectedSide)
        ? parsed.selectedSide
        : defaults.selectedSide,
      compactMode:
        typeof parsed.compactMode === "boolean" ? parsed.compactMode : defaults.compactMode,
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : defaults.themeMode,
      directWorktreeCommitPolicy: isDirectWorktreeCommitPolicy(parsed.directWorktreeCommitPolicy)
        ? parsed.directWorktreeCommitPolicy
        : defaults.directWorktreeCommitPolicy,
      projectPaneCollapsed:
        typeof parsed.projectPaneCollapsed === "boolean"
          ? parsed.projectPaneCollapsed
          : defaults.projectPaneCollapsed,
      panelWidths: {
        project: clampPanelWidth("project", parsed.panelWidths?.project ?? defaults.panelWidths.project),
        changes: clampPanelWidth(
          "changes",
          parsed.panelWidths?.changes ?? defaults.panelWidths.changes,
        ),
      },
      panelVisibility: {
        project:
          typeof parsed.panelVisibility?.project === "boolean"
            ? parsed.panelVisibility.project
            : defaults.panelVisibility.project,
        changes:
          typeof parsed.panelVisibility?.changes === "boolean"
            ? parsed.panelVisibility.changes
            : defaults.panelVisibility.changes,
      },
    };
  } catch {
    return defaults;
  }
}

export const useSettingsStore = defineStore("settings", {
  state: () => readStoredSettings(),
  actions: {
    persist() {
      if (typeof window === "undefined") return;
      window.localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          includeIgnored: this.includeIgnored,
          selectedSide: this.selectedSide,
          compactMode: this.compactMode,
          themeMode: this.themeMode,
          directWorktreeCommitPolicy: this.directWorktreeCommitPolicy,
          projectPaneCollapsed: this.projectPaneCollapsed,
          panelWidths: this.panelWidths,
          panelVisibility: this.panelVisibility,
        }),
      );
    },
    setIncludeIgnored(value: boolean) {
      this.includeIgnored = value;
      this.persist();
    },
    setSide(side: ChangeSide) {
      this.selectedSide = side;
      this.persist();
    },
    setThemeMode(themeMode: ThemeMode) {
      this.themeMode = themeMode;
      this.persist();
    },
    setDirectWorktreeCommitPolicy(policy: DirectWorktreeCommitPolicy) {
      this.directWorktreeCommitPolicy = policy;
      this.persist();
    },
    setProjectPaneCollapsed(collapsed: boolean) {
      this.projectPaneCollapsed = collapsed;
      this.persist();
    },
    setPanelVisible(panel: LayoutPanelKey, visible: boolean) {
      this.panelVisibility[panel] = visible;
      this.persist();
    },
    setPanelWidth(panel: LayoutPanelKey, width: number) {
      this.panelWidths[panel] = clampPanelWidth(panel, width);
      this.persist();
    },
    resetLayout() {
      this.panelWidths = { ...DEFAULT_PANEL_WIDTHS };
      this.panelVisibility = { ...DEFAULT_PANEL_VISIBILITY };
      this.projectPaneCollapsed = false;
      this.persist();
    },
  },
});
