export const CHROME = {
  menuHeight: 32,
  tabHeight: 42,
  workspaceToolbarHeight: 48,
  railWidth: 44,
  bottomToolbarHeight: 44,
  splitLineWidth: 1,
  splitHitZoneWidth: 8,
  iconSize: 'lg',
  iconGlyph: 18,
  smallIconGlyph: 16,
  labelIconGlyph: 17,
  panelGap: 8,
} as const;

export const APP_HEADER_HEIGHT = CHROME.menuHeight + CHROME.tabHeight;
