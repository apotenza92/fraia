import type { AgentTarget } from './types';

export type EmptyCanvasSelectionAction = 'clear' | 'arm-window';

export function emptyCanvasSelectionAction(hasSelection: boolean, forceWindow: boolean): EmptyCanvasSelectionAction {
  return hasSelection && !forceWindow ? 'clear' : 'arm-window';
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
