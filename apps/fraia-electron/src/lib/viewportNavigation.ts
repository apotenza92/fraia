export const VIEWPORT_NAVIGATION_PROFILE_IDS = [
  'spacegass',
  'strand7',
  'rhino',
  'ansys',
  'autocad',
  'tekla',
  'custom',
] as const;

export type ViewportNavigationProfileId = (typeof VIEWPORT_NAVIGATION_PROFILE_IDS)[number];
export type ViewportNavigationAction = 'rotate' | 'pan' | 'zoom' | 'none';
export type ViewportMouseButton = 'left' | 'middle' | 'right';
export type ViewportMouseHandedness = 'right' | 'left';
export type ViewportCustomNavigationSettings = Record<ViewportMouseButton, ViewportNavigationAction>;

export type ViewportNavigationBinding = {
  action: Exclude<ViewportNavigationAction, 'none'>;
  button?: ViewportMouseButton;
  chord?: 'left+right';
  modifier?: 'shift' | 'primary';
  label: string;
};

export type ViewportNavigationProfile = {
  id: ViewportNavigationProfileId;
  label: string;
  shortLabel: string;
  bindings: ViewportNavigationBinding[];
  essentials: {
    rotate: string;
    pan: string;
    zoom: string;
  };
};

export type ViewportNavigationGestureInput = {
  button: number;
  buttons: number;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
};

export const DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID: ViewportNavigationProfileId = 'spacegass';
export const VIEWPORT_NAVIGATION_STORAGE_KEY = 'fraia.viewport.navigationProfile.v1';
export const VIEWPORT_CUSTOM_NAVIGATION_STORAGE_KEY = 'fraia.viewport.customNavigation.v1';
export const DEFAULT_VIEWPORT_MOUSE_HANDEDNESS: ViewportMouseHandedness = 'right';
export const VIEWPORT_MOUSE_HANDEDNESS_STORAGE_KEY = 'fraia.viewport.mouseHandedness.v1';
export const DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS: ViewportCustomNavigationSettings = {
  left: 'rotate',
  middle: 'pan',
  right: 'none',
};

export const VIEWPORT_NAVIGATION_PROFILES: readonly ViewportNavigationProfile[] = [
  {
    id: 'spacegass',
    label: 'Fraia — SPACE GASS',
    shortLabel: 'SPACE GASS',
    bindings: [
      { action: 'rotate', button: 'left', label: 'Left drag' },
      { action: 'pan', button: 'right', label: 'Right drag' },
      { action: 'zoom', button: 'middle', label: 'Middle vertical drag' },
    ],
    essentials: { rotate: 'Left drag', pan: 'Right drag', zoom: 'Wheel / middle drag' },
  },
  {
    id: 'strand7',
    label: 'Strand7',
    shortLabel: 'Strand7',
    bindings: [
      { action: 'rotate', button: 'left', label: 'Left drag' },
      { action: 'pan', chord: 'left+right', label: 'Left + right drag' },
      { action: 'zoom', button: 'right', label: 'Right vertical drag' },
    ],
    essentials: { rotate: 'Left drag', pan: 'Left + right drag', zoom: 'Wheel / right drag' },
  },
  {
    id: 'rhino',
    label: 'Rhino',
    shortLabel: 'Rhino',
    bindings: [
      { action: 'rotate', button: 'right', label: 'Right drag' },
      { action: 'pan', button: 'right', modifier: 'shift', label: 'Shift + right drag' },
      { action: 'zoom', button: 'right', modifier: 'primary', label: 'Ctrl/Cmd + right drag' },
    ],
    essentials: { rotate: 'Right drag', pan: 'Shift + right drag', zoom: 'Wheel / Ctrl/Cmd + right drag' },
  },
  {
    id: 'ansys',
    label: 'Ansys',
    shortLabel: 'Ansys',
    bindings: [
      { action: 'rotate', button: 'middle', label: 'Middle drag' },
      { action: 'pan', button: 'middle', modifier: 'primary', label: 'Ctrl/Cmd + middle drag' },
      { action: 'zoom', button: 'middle', modifier: 'shift', label: 'Shift + middle drag' },
    ],
    essentials: { rotate: 'Middle drag', pan: 'Ctrl/Cmd + middle drag', zoom: 'Wheel / Shift + middle drag' },
  },
  {
    id: 'autocad',
    label: 'AutoCAD',
    shortLabel: 'AutoCAD',
    bindings: [
      { action: 'pan', button: 'middle', label: 'Middle drag' },
      { action: 'rotate', button: 'middle', modifier: 'shift', label: 'Shift + middle drag' },
    ],
    essentials: { rotate: 'Shift + middle drag', pan: 'Middle drag', zoom: 'Wheel' },
  },
  {
    id: 'tekla',
    label: 'Tekla Structures',
    shortLabel: 'Tekla Structures',
    bindings: [
      { action: 'pan', button: 'middle', label: 'Middle drag' },
      { action: 'rotate', button: 'middle', modifier: 'primary', label: 'Ctrl/Cmd + middle drag' },
    ],
    essentials: { rotate: 'Ctrl/Cmd + middle drag', pan: 'Middle drag', zoom: 'Wheel' },
  },
  {
    id: 'custom',
    label: 'Custom',
    shortLabel: 'Custom',
    bindings: [],
    essentials: { rotate: 'Left drag', pan: 'Middle drag', zoom: 'Wheel' },
  },
];

const profilesById = new Map(VIEWPORT_NAVIGATION_PROFILES.map((profile) => [profile.id, profile]));

function customNavigationProfile(settings: ViewportCustomNavigationSettings): ViewportNavigationProfile {
  const buttonLabel: Record<ViewportMouseButton, string> = {
    left: 'Left drag',
    middle: 'Middle drag',
    right: 'Right drag',
  };
  const bindings = (Object.entries(settings) as [ViewportMouseButton, ViewportNavigationAction][])
    .filter((entry): entry is [ViewportMouseButton, Exclude<ViewportNavigationAction, 'none'>] => entry[1] !== 'none')
    .map(([button, action]) => ({ action, button, label: buttonLabel[button] }));
  const labelsFor = (action: Exclude<ViewportNavigationAction, 'none'>) => (
    bindings.filter((binding) => binding.action === action).map((binding) => binding.label)
  );
  const rotate = labelsFor('rotate');
  const pan = labelsFor('pan');
  const zoom = labelsFor('zoom');
  return {
    id: 'custom',
    label: 'Custom',
    shortLabel: 'Custom',
    bindings,
    essentials: {
      rotate: rotate.join(' / ') || 'Not assigned',
      pan: pan.join(' / ') || 'Not assigned',
      zoom: ['Wheel', ...zoom].join(' / '),
    },
  };
}

export function viewportNavigationProfile(
  id: ViewportNavigationProfileId,
  customSettings: ViewportCustomNavigationSettings = DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS,
) {
  if (id === 'custom') return customNavigationProfile(customSettings);
  return profilesById.get(id) ?? profilesById.get(DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID)!;
}

export function isViewportNavigationProfileId(value: unknown): value is ViewportNavigationProfileId {
  return typeof value === 'string' && VIEWPORT_NAVIGATION_PROFILE_IDS.includes(value as ViewportNavigationProfileId);
}

export function isViewportMouseHandedness(value: unknown): value is ViewportMouseHandedness {
  return value === 'right' || value === 'left';
}

export function isViewportNavigationAction(value: unknown): value is ViewportNavigationAction {
  return value === 'rotate' || value === 'pan' || value === 'zoom' || value === 'none';
}

export function handedMouseButton(
  button: ViewportMouseButton,
  handedness: ViewportMouseHandedness,
): ViewportMouseButton {
  if (handedness === 'right' || button === 'middle') return button;
  return button === 'left' ? 'right' : 'left';
}

export function handedViewportNavigationLabel(
  label: string,
  handedness: ViewportMouseHandedness,
) {
  if (handedness === 'right') return label;
  return label
    .replace(/\bLeft \+ right\b/g, '__viewport_both__')
    .replace(/\bleft \+ right\b/g, '__viewport_both_lower__')
    .replace(/\bLeft\b/g, '__viewport_right__')
    .replace(/\bRight\b/g, 'Left')
    .replace(/__viewport_right__/g, 'Right')
    .replace(/\bleft\b/g, '__viewport_right__')
    .replace(/\bright\b/g, 'left')
    .replace(/__viewport_right__/g, 'right')
    .replace(/__viewport_both__/g, 'Left + right')
    .replace(/__viewport_both_lower__/g, 'left + right');
}

function inputButton(button: number): ViewportMouseButton | null {
  if (button === 0) return 'left';
  if (button === 1) return 'middle';
  if (button === 2) return 'right';
  return null;
}

export function resolveViewportNavigationGesture(
  profileId: ViewportNavigationProfileId,
  input: ViewportNavigationGestureInput,
  customSettings: ViewportCustomNavigationSettings = DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS,
): ViewportNavigationAction {
  const primary = input.ctrlKey || input.metaKey;
  if (primary && input.shiftKey) return 'none';
  const profile = viewportNavigationProfile(profileId, customSettings);
  if (input.buttons === 3) {
    return profile.bindings.find((binding) => binding.chord === 'left+right')?.action ?? 'none';
  }
  const button = inputButton(input.button);
  if (!button) return 'none';
  const modifier = input.shiftKey ? 'shift' : primary ? 'primary' : undefined;
  return profile.bindings.find((binding) => (
    binding.button === button && binding.modifier === modifier
  ))?.action ?? 'none';
}

export function loadStoredViewportNavigationProfile(storage: Pick<Storage, 'getItem'> | null = typeof window === 'undefined' ? null : window.localStorage) {
  if (!storage) return DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID;
  try {
    const value = storage.getItem(VIEWPORT_NAVIGATION_STORAGE_KEY);
    return isViewportNavigationProfileId(value) ? value : DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID;
  } catch {
    return DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID;
  }
}

export function storeViewportNavigationProfile(
  profileId: ViewportNavigationProfileId,
  storage: Pick<Storage, 'setItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
) {
  if (!storage) return;
  try {
    storage.setItem(VIEWPORT_NAVIGATION_STORAGE_KEY, profileId);
  } catch {
    // Navigation remains usable even when storage is unavailable.
  }
}

export function loadStoredViewportCustomNavigationSettings(
  storage: Pick<Storage, 'getItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
): ViewportCustomNavigationSettings {
  if (!storage) return { ...DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS };
  try {
    const value = JSON.parse(storage.getItem(VIEWPORT_CUSTOM_NAVIGATION_STORAGE_KEY) ?? 'null');
    if (
      typeof value === 'object'
      && value !== null
      && isViewportNavigationAction(value.left)
      && isViewportNavigationAction(value.middle)
      && isViewportNavigationAction(value.right)
    ) {
      return { left: value.left, middle: value.middle, right: value.right };
    }
  } catch {
    // Fall through to the safe default mapping.
  }
  return { ...DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS };
}

export function storeViewportCustomNavigationSettings(
  settings: ViewportCustomNavigationSettings,
  storage: Pick<Storage, 'setItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
) {
  if (!storage) return;
  try {
    storage.setItem(VIEWPORT_CUSTOM_NAVIGATION_STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Custom navigation remains usable even when storage is unavailable.
  }
}

export function loadStoredViewportMouseHandedness(
  storage: Pick<Storage, 'getItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
) {
  if (!storage) return DEFAULT_VIEWPORT_MOUSE_HANDEDNESS;
  try {
    const value = storage.getItem(VIEWPORT_MOUSE_HANDEDNESS_STORAGE_KEY);
    return isViewportMouseHandedness(value) ? value : DEFAULT_VIEWPORT_MOUSE_HANDEDNESS;
  } catch {
    return DEFAULT_VIEWPORT_MOUSE_HANDEDNESS;
  }
}

export function storeViewportMouseHandedness(
  handedness: ViewportMouseHandedness,
  storage: Pick<Storage, 'setItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
) {
  if (!storage) return;
  try {
    storage.setItem(VIEWPORT_MOUSE_HANDEDNESS_STORAGE_KEY, handedness);
  } catch {
    // Mouse help remains usable even when storage is unavailable.
  }
}
