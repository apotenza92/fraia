import { beforeEach, describe, expect, it } from 'vitest';

import {
  DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID,
  DEFAULT_VIEWPORT_MOUSE_HANDEDNESS,
  DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS,
  VIEWPORT_CUSTOM_NAVIGATION_STORAGE_KEY,
  VIEWPORT_MOUSE_HANDEDNESS_STORAGE_KEY,
  VIEWPORT_NAVIGATION_STORAGE_KEY,
  handedMouseButton,
  handedViewportNavigationLabel,
  loadStoredViewportMouseHandedness,
  loadStoredViewportCustomNavigationSettings,
  loadStoredViewportNavigationProfile,
  resolveViewportNavigationGesture,
  storeViewportMouseHandedness,
  storeViewportCustomNavigationSettings,
  storeViewportNavigationProfile,
  viewportNavigationProfile,
  viewportZoomSpeedForGesture,
  type ViewportNavigationProfileId,
} from '@/lib/viewportNavigation';

function gesture(button: number, options: Partial<{ buttons: number; shiftKey: boolean; ctrlKey: boolean; metaKey: boolean }> = {}) {
  return {
    button,
    buttons: options.buttons ?? (button === 0 ? 1 : button === 1 ? 4 : 2),
    shiftKey: options.shiftKey ?? false,
    ctrlKey: options.ctrlKey ?? false,
    metaKey: options.metaKey ?? false,
  };
}

describe('viewport navigation profiles', () => {
  beforeEach(() => window.localStorage.clear());

  it.each<[
    ViewportNavigationProfileId,
    ReturnType<typeof gesture>,
    'rotate' | 'pan' | 'zoom' | 'none',
  ]>([
    ['spacegass', gesture(0), 'rotate'],
    ['spacegass', gesture(2), 'pan'],
    ['spacegass', gesture(1), 'zoom'],
    ['strand7', gesture(0), 'rotate'],
    ['strand7', gesture(2), 'zoom'],
    ['strand7', gesture(2, { buttons: 3 }), 'pan'],
    ['rhino', gesture(2), 'rotate'],
    ['rhino', gesture(2, { shiftKey: true }), 'pan'],
    ['rhino', gesture(2, { metaKey: true }), 'zoom'],
    ['ansys', gesture(1), 'rotate'],
    ['ansys', gesture(1, { ctrlKey: true }), 'pan'],
    ['ansys', gesture(1, { shiftKey: true }), 'zoom'],
    ['autocad', gesture(1), 'pan'],
    ['autocad', gesture(1, { shiftKey: true }), 'rotate'],
    ['tekla', gesture(1), 'pan'],
    ['tekla', gesture(1, { metaKey: true }), 'rotate'],
    ['custom', gesture(0), 'rotate'],
    ['custom', gesture(1), 'pan'],
    ['custom', gesture(2), 'none'],
  ])('%s resolves its documented mouse gesture', (profileId, input, expected) => {
    expect(resolveViewportNavigationGesture(profileId, input)).toBe(expected);
  });

  it('rejects ambiguous primary-plus-shift gestures', () => {
    expect(resolveViewportNavigationGesture('rhino', gesture(2, { shiftKey: true, metaKey: true }))).toBe('none');
    expect(resolveViewportNavigationGesture('ansys', gesture(1, { shiftKey: true, ctrlKey: true }))).toBe('none');
  });

  it('makes drag zoom meaningful without changing other camera gestures', () => {
    expect(viewportZoomSpeedForGesture('zoom')).toBe(6);
    expect(viewportZoomSpeedForGesture('rotate')).toBe(1);
    expect(viewportZoomSpeedForGesture('pan')).toBe(1);
    expect(viewportZoomSpeedForGesture('none')).toBe(1);
  });

  it('uses SPACE GASS-style as the default and invalid-storage fallback', () => {
    expect(DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID).toBe('spacegass');
    expect(viewportNavigationProfile(DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID).essentials).toEqual({
      rotate: 'Left drag',
      pan: 'Right drag',
      zoom: 'Wheel / middle drag',
    });
    window.localStorage.setItem(VIEWPORT_NAVIGATION_STORAGE_KEY, 'unknown');
    expect(loadStoredViewportNavigationProfile()).toBe('spacegass');
  });

  it('persists one app-wide profile preference', () => {
    storeViewportNavigationProfile('rhino');
    expect(loadStoredViewportNavigationProfile()).toBe('rhino');
  });

  it('resolves and describes a custom mouse mapping without changing SPACE GASS', () => {
    const custom = { left: 'pan', middle: 'zoom', right: 'rotate' } as const;
    expect(resolveViewportNavigationGesture('custom', gesture(0), custom)).toBe('pan');
    expect(resolveViewportNavigationGesture('custom', gesture(1), custom)).toBe('zoom');
    expect(resolveViewportNavigationGesture('custom', gesture(2), custom)).toBe('rotate');
    expect(viewportNavigationProfile('custom', custom).essentials).toEqual({
      rotate: 'Right drag',
      pan: 'Left drag',
      zoom: 'Wheel / Middle drag',
    });
    expect(viewportNavigationProfile('spacegass', custom).essentials).toEqual({
      rotate: 'Left drag',
      pan: 'Right drag',
      zoom: 'Wheel / middle drag',
    });
  });

  it('persists valid custom mappings and falls back safely for invalid data', () => {
    expect(loadStoredViewportCustomNavigationSettings()).toEqual(DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS);
    const custom = { left: 'none', middle: 'pan', right: 'rotate' } as const;
    storeViewportCustomNavigationSettings(custom);
    expect(loadStoredViewportCustomNavigationSettings()).toEqual(custom);
    window.localStorage.setItem(VIEWPORT_CUSTOM_NAVIGATION_STORAGE_KEY, JSON.stringify({ left: 'fly', middle: 'pan', right: 'rotate' }));
    expect(loadStoredViewportCustomNavigationSettings()).toEqual(DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS);
  });

  it('swaps physical left and right help for a left-handed mouse', () => {
    expect(handedMouseButton('left', 'left')).toBe('right');
    expect(handedMouseButton('right', 'left')).toBe('left');
    expect(handedMouseButton('middle', 'left')).toBe('middle');
    expect(handedViewportNavigationLabel('Wheel / Ctrl/Cmd + right drag', 'left')).toBe('Wheel / Ctrl/Cmd + left drag');
    expect(handedViewportNavigationLabel('Left + right drag', 'left')).toBe('Left + right drag');
  });

  it('persists mouse handedness with a right-handed fallback', () => {
    expect(DEFAULT_VIEWPORT_MOUSE_HANDEDNESS).toBe('right');
    expect(loadStoredViewportMouseHandedness()).toBe('right');
    storeViewportMouseHandedness('left');
    expect(loadStoredViewportMouseHandedness()).toBe('left');
    window.localStorage.setItem(VIEWPORT_MOUSE_HANDEDNESS_STORAGE_KEY, 'unknown');
    expect(loadStoredViewportMouseHandedness()).toBe('right');
  });
});
