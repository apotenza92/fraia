import type { AgentTarget } from './types';
import type { ViewportNavigationProfileId } from './viewportNavigation';

export type EmptyCanvasSelectionAction = 'clear' | 'arm-window' | 'ignore';
export type ViewportSelectionOperation = 'replace' | 'add' | 'remove' | 'toggle';
export type ViewportSelectionWindowGesture = 'two-click' | 'drag' | 'shift-drag';
export type ViewportSelectionPickBehavior = 'replace' | 'add' | 'toggle';
export type ViewportSelectionModifierStyle = 'shift-add-primary-remove' | 'shift-remove-primary-toggle' | 'primary-toggle' | 'none';
export type ViewportSelectionEmptyBehavior = 'clear' | 'start-window' | 'ignore';
export type ViewportSelectionForceWindowModifier = 'shift' | 'alt' | 'none';
export type ViewportSelectionModifiers = {
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
};

export type ViewportCustomSelectionSettings = {
  pickBehavior: ViewportSelectionPickBehavior;
  modifierStyle: ViewportSelectionModifierStyle;
  emptyBehavior: ViewportSelectionEmptyBehavior;
  windowGesture: Exclude<ViewportSelectionWindowGesture, 'shift-drag'>;
  windowBehavior: ViewportSelectionPickBehavior;
  forceWindowModifier: ViewportSelectionForceWindowModifier;
};

export const VIEWPORT_CUSTOM_SELECTION_STORAGE_KEY = 'fraia.viewport.customSelection.v1';
export const DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS: ViewportCustomSelectionSettings = {
  pickBehavior: 'replace',
  modifierStyle: 'shift-add-primary-remove',
  emptyBehavior: 'clear',
  windowGesture: 'drag',
  windowBehavior: 'replace',
  forceWindowModifier: 'alt',
};

function operationWithModifiers(
  base: ViewportSelectionPickBehavior,
  modifierStyle: ViewportSelectionModifierStyle,
  modifiers: ViewportSelectionModifiers,
): ViewportSelectionOperation {
  const primary = modifiers.ctrlKey || modifiers.metaKey;
  if (modifierStyle === 'shift-add-primary-remove') {
    if (primary) return 'remove';
    if (modifiers.shiftKey) return 'add';
  } else if (modifierStyle === 'shift-remove-primary-toggle') {
    if (modifiers.shiftKey) return 'remove';
    if (primary) return 'toggle';
  } else if (modifierStyle === 'primary-toggle' && primary) {
    return 'toggle';
  }
  return base;
}

export function directSelectionOperation(
  profileId: ViewportNavigationProfileId,
  modifiers: ViewportSelectionModifiers,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
): ViewportSelectionOperation {
  if (profileId === 'spacegass' || profileId === 'strand7') return 'toggle';
  if (profileId === 'autocad') return modifiers.shiftKey ? 'remove' : 'add';
  if (profileId === 'tekla') {
    if (modifiers.ctrlKey || modifiers.metaKey) return 'toggle';
    return modifiers.shiftKey ? 'add' : 'replace';
  }
  if (profileId === 'ansys') {
    if (modifiers.ctrlKey || modifiers.metaKey) return 'toggle';
    return modifiers.shiftKey ? 'add' : 'replace';
  }
  if (profileId === 'custom') return operationWithModifiers(customSettings.pickBehavior, customSettings.modifierStyle, modifiers);
  return operationWithModifiers('replace', 'shift-add-primary-remove', modifiers);
}

export function windowSelectionOperation(
  profileId: ViewportNavigationProfileId,
  modifiers: ViewportSelectionModifiers,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
): ViewportSelectionOperation {
  if (profileId === 'spacegass') return modifiers.shiftKey ? 'add' : 'toggle';
  if (profileId === 'strand7') return 'toggle';
  if (profileId === 'autocad') return modifiers.shiftKey ? 'remove' : 'add';
  if (profileId === 'custom') return operationWithModifiers(customSettings.windowBehavior, customSettings.modifierStyle, modifiers);
  return directSelectionOperation(profileId, modifiers, customSettings);
}

export function forcesWindowFromTarget(
  profileId: ViewportNavigationProfileId,
  modifiers: ViewportSelectionModifiers,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
) {
  const forceModifier = profileId === 'spacegass'
    ? 'shift'
    : profileId === 'rhino'
      ? 'alt'
      : profileId === 'custom'
        ? customSettings.forceWindowModifier
        : 'none';
  return (forceModifier === 'shift' && modifiers.shiftKey) || (forceModifier === 'alt' && modifiers.altKey);
}

export function selectionWindowGesture(
  profileId: ViewportNavigationProfileId,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
): ViewportSelectionWindowGesture {
  if (profileId === 'spacegass' || profileId === 'autocad') return 'two-click';
  if (profileId === 'strand7') return 'shift-drag';
  if (profileId === 'custom') return customSettings.windowGesture;
  return 'drag';
}

export function emptyCanvasSelectionAction(
  profileId: ViewportNavigationProfileId,
  hasSelection: boolean,
  modifiers: ViewportSelectionModifiers,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
): EmptyCanvasSelectionAction {
  const behavior = profileId === 'spacegass' || profileId === 'autocad'
    ? 'start-window'
    : profileId === 'strand7'
      ? 'ignore'
      : profileId === 'custom'
        ? customSettings.emptyBehavior
        : 'clear';
  if (behavior === 'start-window') return 'arm-window';
  if (behavior === 'ignore') return 'ignore';
  if (behavior === 'clear') return hasSelection && !modifiers.shiftKey ? 'clear' : 'ignore';
  return 'arm-window';
}

export type ViewportSelectionDescription = {
  short: string;
  pick: string;
  blank: string;
  window: string;
  modify: string;
};

const baseOperationLabel: Record<ViewportSelectionPickBehavior, string> = {
  replace: 'replaces selection',
  add: 'adds to selection',
  toggle: 'toggles selection',
};

function customModifierLabel(style: ViewportSelectionModifierStyle) {
  if (style === 'shift-add-primary-remove') return 'Shift adds · Ctrl/Cmd removes';
  if (style === 'shift-remove-primary-toggle') return 'Shift removes · Ctrl/Cmd toggles';
  if (style === 'primary-toggle') return 'Ctrl/Cmd toggles';
  return 'No selection modifiers';
}

export function viewportSelectionDescription(
  profileId: ViewportNavigationProfileId,
  customSettings: ViewportCustomSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
): ViewportSelectionDescription {
  if (profileId === 'spacegass') return {
    short: 'Toggle picks · Two-click window',
    pick: 'Click toggles items; no modifier is needed to build a set.',
    blank: 'A blank click sets the first corner of a selection window.',
    window: 'Click the opposite corner; left-to-right encloses, right-to-left crosses.',
    modify: 'Shift forces the first corner over an item and makes the completed window add-only.',
  };
  if (profileId === 'strand7') return {
    short: 'Toggle picks · Shift-drag window',
    pick: 'Click toggles items using Strand7 invert-selection behavior.',
    blank: 'A plain blank click leaves the selection unchanged.',
    window: 'Shift-drag creates the 2D region while preserving left-drag rotation.',
    modify: 'The window toggles captured items; Shift also represents Strand7 through-selection.',
  };
  if (profileId === 'rhino') return {
    short: 'Replace picks · Shift adds',
    pick: 'Click replaces the selection; Shift-click adds.',
    blank: 'Clicking blank canvas clears the selection.',
    window: 'Drag left-to-right to enclose or right-to-left to cross.',
    modify: 'Ctrl/Cmd removes; Alt forces a window when starting near an item.',
  };
  if (profileId === 'ansys') return {
    short: 'Replace picks · Ctrl/Cmd toggles',
    pick: 'Click replaces the selection; Ctrl/Cmd-click toggles items.',
    blank: 'Clicking blank canvas clears the selection.',
    window: 'Drag a box; left-to-right encloses and right-to-left crosses.',
    modify: 'Shift adds; Ctrl/Cmd toggles captured items.',
  };
  if (profileId === 'autocad') return {
    short: 'Add picks · Two-click window',
    pick: 'Each click adds an item to the selection set.',
    blank: 'A blank click sets the first corner of a selection window.',
    window: 'Click the opposite corner; left-to-right encloses, right-to-left crosses.',
    modify: 'Shift removes picked items or items captured by a window.',
  };
  if (profileId === 'tekla') return {
    short: 'Replace picks · Shift adds',
    pick: 'Click replaces the selection; Shift-click adds.',
    blank: 'Clicking blank canvas clears the selection.',
    window: 'Drag left-to-right to enclose or right-to-left to cross.',
    modify: 'Ctrl/Cmd toggles picked or captured items.',
  };
  const gesture = customSettings.windowGesture === 'two-click' ? 'Two-click window' : 'Drag window';
  const blank = customSettings.emptyBehavior === 'clear'
    ? 'Clicking blank canvas clears the selection.'
    : customSettings.emptyBehavior === 'start-window'
      ? 'A blank click starts a selection window.'
      : 'A blank click leaves the selection unchanged.';
  return {
    short: `${baseOperationLabel[customSettings.pickBehavior].replace(' selection', '')} · ${gesture}`,
    pick: `A direct click ${baseOperationLabel[customSettings.pickBehavior]}.`,
    blank,
    window: `${gesture}; completion ${baseOperationLabel[customSettings.windowBehavior]}.`,
    modify: customModifierLabel(customSettings.modifierStyle),
  };
}

function isPickBehavior(value: unknown): value is ViewportSelectionPickBehavior {
  return value === 'replace' || value === 'add' || value === 'toggle';
}

function isModifierStyle(value: unknown): value is ViewportSelectionModifierStyle {
  return value === 'shift-add-primary-remove' || value === 'shift-remove-primary-toggle' || value === 'primary-toggle' || value === 'none';
}

function isEmptyBehavior(value: unknown): value is ViewportSelectionEmptyBehavior {
  return value === 'clear' || value === 'start-window' || value === 'ignore';
}

function isWindowGesture(value: unknown): value is ViewportCustomSelectionSettings['windowGesture'] {
  return value === 'two-click' || value === 'drag';
}

function isForceWindowModifier(value: unknown): value is ViewportSelectionForceWindowModifier {
  return value === 'shift' || value === 'alt' || value === 'none';
}

export function loadStoredViewportCustomSelectionSettings(
  storage: Pick<Storage, 'getItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
): ViewportCustomSelectionSettings {
  if (!storage) return { ...DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS };
  try {
    const value = JSON.parse(storage.getItem(VIEWPORT_CUSTOM_SELECTION_STORAGE_KEY) ?? 'null') as Partial<ViewportCustomSelectionSettings> | null;
    if (
      typeof value === 'object'
      && value !== null
      && isPickBehavior(value.pickBehavior)
      && isModifierStyle(value.modifierStyle)
      && isEmptyBehavior(value.emptyBehavior)
      && isWindowGesture(value.windowGesture)
      && isPickBehavior(value.windowBehavior)
      && isForceWindowModifier(value.forceWindowModifier)
    ) return value as ViewportCustomSelectionSettings;
  } catch {
    // Fall through to the documented safe default.
  }
  return { ...DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS };
}

export function storeViewportCustomSelectionSettings(
  settings: ViewportCustomSelectionSettings,
  storage: Pick<Storage, 'setItem'> | null = typeof window === 'undefined' ? null : window.localStorage,
) {
  if (!storage) return;
  try {
    storage.setItem(VIEWPORT_CUSTOM_SELECTION_STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Custom selection remains usable when storage is unavailable.
  }
}

export function viewportSelectionKind(startX: number, endX: number): 'window' | 'crossing' {
  return endX >= startX ? 'window' : 'crossing';
}

export function isViewportNodeSelectable(nodeId: string, passiveNodeIds: ReadonlySet<string>) {
  return !passiveNodeIds.has(nodeId);
}

export function prioritizeViewportPointerTarget(
  geometryTarget: AgentTarget | null,
  labelTarget: AgentTarget | null,
) {
  const geometryHasPriority = geometryTarget?.kind === 'node' || geometryTarget?.kind === 'member';
  const labelIsSecondary = labelTarget?.kind === 'support' || labelTarget?.kind === 'load';
  return geometryHasPriority && labelIsSecondary ? geometryTarget : labelTarget ?? geometryTarget;
}

function targetKey(target: AgentTarget) {
  return `${target.kind}:${target.id}`;
}

export function toggleAgentTargets(current: AgentTarget[], targets: AgentTarget[]) {
  const originallySelected = new Set(current.map(targetKey));
  const next = new Map(current.map((target) => [targetKey(target), target]));
  targets.forEach((target) => {
    const key = targetKey(target);
    if (originallySelected.has(key)) next.delete(key);
    else next.set(key, target);
  });
  return [...next.values()];
}
