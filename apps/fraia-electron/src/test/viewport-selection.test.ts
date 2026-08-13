import { describe, expect, it } from 'vitest';

import {
  DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
  directSelectionOperation,
  emptyCanvasSelectionAction,
  forcesWindowFromTarget,
  isViewportNodeSelectable,
  prioritizeViewportPointerTarget,
  selectionWindowGesture,
  toggleAgentTargets,
  viewportSelectionKind,
  windowSelectionOperation,
} from '@/lib/viewportSelection';

const noModifiers = { altKey: false, ctrlKey: false, metaKey: false, shiftKey: false };

describe('viewport selection grammar', () => {
  it('consumes a plain empty click as clear when a selection exists', () => {
    expect(emptyCanvasSelectionAction('rhino', true, noModifiers)).toBe('clear');
  });

  it('arms a window when selection is empty or Shift forces it', () => {
    expect(emptyCanvasSelectionAction('spacegass', false, noModifiers)).toBe('arm-window');
    expect(emptyCanvasSelectionAction('strand7', true, noModifiers)).toBe('ignore');
  });

  it('matches each profile selection modifier contract', () => {
    const shift = { ...noModifiers, shiftKey: true };
    const primary = { ...noModifiers, ctrlKey: true };
    expect(directSelectionOperation('spacegass', noModifiers)).toBe('toggle');
    expect(windowSelectionOperation('spacegass', shift)).toBe('add');
    expect(directSelectionOperation('rhino', noModifiers)).toBe('replace');
    expect(directSelectionOperation('rhino', shift)).toBe('add');
    expect(directSelectionOperation('rhino', primary)).toBe('remove');
    expect(directSelectionOperation('ansys', shift)).toBe('add');
    expect(directSelectionOperation('ansys', primary)).toBe('toggle');
    expect(directSelectionOperation('autocad', noModifiers)).toBe('add');
    expect(directSelectionOperation('autocad', shift)).toBe('remove');
    expect(directSelectionOperation('tekla', primary)).toBe('toggle');
  });

  it('uses profile-specific window gestures and force modifiers', () => {
    expect(selectionWindowGesture('spacegass')).toBe('two-click');
    expect(selectionWindowGesture('strand7')).toBe('shift-drag');
    expect(selectionWindowGesture('rhino')).toBe('drag');
    expect(forcesWindowFromTarget('spacegass', { ...noModifiers, shiftKey: true })).toBe(true);
    expect(forcesWindowFromTarget('rhino', { ...noModifiers, altKey: true })).toBe(true);
    expect(selectionWindowGesture('custom', { ...DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS, windowGesture: 'two-click' })).toBe('two-click');
  });

  it('uses window and crossing semantics based on horizontal direction', () => {
    expect(viewportSelectionKind(10, 40)).toBe('window');
    expect(viewportSelectionKind(40, 10)).toBe('crossing');
  });

  it('toggles mixed captured targets against the original selection', () => {
    const node = { kind: 'node' as const, id: 'N1' };
    const member = { kind: 'member' as const, id: 'M1' };
    const load = { kind: 'load' as const, id: 'L1' };
    expect(toggleAgentTargets([node, load], [node, member])).toEqual([load, member]);
  });

  it('excludes nodes that exist only to anchor proposed supports', () => {
    const proposedSupportNodeIds = new Set(['N-proposed']);
    expect(isViewportNodeSelectable('N1', proposedSupportNodeIds)).toBe(true);
    expect(isViewportNodeSelectable('N-proposed', proposedSupportNodeIds)).toBe(false);
  });

  it('keeps direct node and member hits ahead of load and support labels', () => {
    const node = { kind: 'node', id: 'N1' };
    const member = { kind: 'member', id: 'M1' };
    const load = { kind: 'load', id: 'L1' };
    const support = { kind: 'support', id: 'S1' };
    expect(prioritizeViewportPointerTarget(node, load)).toEqual(node);
    expect(prioritizeViewportPointerTarget(member, support)).toEqual(member);
    expect(prioritizeViewportPointerTarget(load, node)).toEqual(node);
  });
});
