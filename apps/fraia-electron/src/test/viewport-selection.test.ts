import { describe, expect, it } from 'vitest';

import {
  emptyCanvasSelectionAction,
  isViewportNodeSelectable,
  prioritizeViewportPointerTarget,
  toggleAgentTargets,
  viewportSelectionKind,
} from '@/lib/viewportSelection';

describe('viewport selection grammar', () => {
  it('consumes a plain empty click as clear when a selection exists', () => {
    expect(emptyCanvasSelectionAction(true, false)).toBe('clear');
  });

  it('arms a window when selection is empty or Shift forces it', () => {
    expect(emptyCanvasSelectionAction(false, false)).toBe('arm-window');
    expect(emptyCanvasSelectionAction(true, true)).toBe('arm-window');
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
