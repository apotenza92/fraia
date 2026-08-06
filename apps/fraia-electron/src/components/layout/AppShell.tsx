import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { PointerEvent as ReactPointerEvent, ReactNode } from 'react';
import { ArrowDown, ArrowLeft, ChevronDown, Circle, Eye, FileSearch, Layers, Magnet, MousePointer2, Move, PanelRightOpen, PencilLine, Play, Scissors, Sparkles, Triangle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { ButtonGroup } from '@/components/ui/button-group';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { Field, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle, PopoverTrigger } from '@/components/ui/popover';
import { Separator } from '@/components/ui/separator';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { Spinner } from '@/components/ui/spinner';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { Toggle } from '@/components/ui/toggle';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { cn } from '@/lib/utils';
import type { AgentTarget, RenderMember, RenderScene, WorkbenchState } from '../../lib/types';
import { buildSchemeWorkspace } from '../../lib/scene';
import { Viewport3D, type ViewportEditOverlay, type ViewportLabelVisibility, type ViewportPointerInfo, type ViewportSelectionGesture } from '../viewport/Viewport3D';
import { ModelWorkspaceChrome } from './ModelWorkspaceChrome';
import { DesignOptionsPanel, type ActiveView, type WorkspacePanel } from './ModelWorkspaceSidebar';
import { BaseChatPanel } from '../panels/BaseChatPanel';
import { sceneHasSchemeGroups, SchemeGroupsPanelContent } from '../viewport/SchemeGroupsOverlay';
import { LegendDialog } from '../viewport/ViewportLegendOverlay';
import { ResultsWorkspace } from '../results/ResultsWorkspace';
import { AppMenuBar } from './AppMenuBar';
import { normalizeWorkbenchState, projectDirOf } from '../../lib/defaultProject';
import { APP_HEADER_HEIGHT, CHROME } from './chromeMetrics';
import { activeBatchFrom, activeDevelopmentPathFrom, decisionStateFrom, developmentPathsFrom, latestComparisonFrom, optionIdForPath, optionRevisions, revisionForOption } from '../../lib/designOptionDecisions';
import { DesignOptionInspector } from '../options/DesignOptionInspector';
import { DesignOptionAgentPanel } from '../options/DesignOptionAgentPanel';
import { DevelopmentPanel } from '../options/DevelopmentPanel';
import { WorkflowStageBar } from './WorkflowStageBar';
import { initialWorkflowStage, runtimeWorkflowStage, workflowJourneyFrom, type WorkflowStage } from '../../lib/workflowJourney';
import { ResizeHandle } from '../domain-ui/ResizeHandle';
import { DocumentTabBar, documentTabTriggerId, type DocumentTab } from '../domain-ui/DocumentTabBar';
import { SplitButtonSegment } from '../domain-ui/SplitButtonSegment';
import { ViewportHelpBar, type ViewportHelpShortcut } from './ViewportHelpBar';
import {
  loadStoredViewportCustomNavigationSettings,
  loadStoredViewportNavigationProfile,
  loadStoredViewportMouseHandedness,
  storeViewportCustomNavigationSettings,
  storeViewportNavigationProfile,
  storeViewportMouseHandedness,
  type ViewportCustomNavigationSettings,
  type ViewportMouseHandedness,
  type ViewportNavigationProfileId,
} from '../../lib/viewportNavigation';
import { toggleAgentTargets } from '../../lib/viewportSelection';

const WORKSPACE_PANEL_MIN_RATIO = 0.22;
const WORKSPACE_PANEL_MAX_RATIO = 0.4;
const WORKSPACE_PANEL_DEFAULT_RATIO = 0.35;
const WORKSPACE_PANEL_MIN = 300;
const WORKSPACE_PANEL_MAX = 640;
const WORKSPACE_PANEL_FALLBACK_WIDTH = 430;
const GROUPS_PANEL_MIN = 320;
const GROUPS_PANEL_MAX = 860;
const GROUPS_PANEL_MAX_RATIO = 0.4;
const VIEWPORT_LABEL_VISIBILITY_STORAGE_KEY = 'fraia.viewport.labelVisibility.v1';
const VIEWPORT_LABEL_VISIBILITY_TOGGLE_MEMORY_STORAGE_KEY = 'fraia.viewport.labelVisibilityToggleMemory.v1';
const VIEWPORT_SNAP_TOGGLE_MEMORY_STORAGE_KEY = 'fraia.viewport.snapToggleMemory.v1';
const AUTO_COLLAPSE_WIDTH = 1180;
const DEFAULT_CHAT_WIDTH_RATIO = 0.4;
const START_GEOMETRY_ONLY = import.meta.env.VITE_FRAIA_GEOMETRY_ONLY === '1';
const OPTION_INSPECTOR_WIDTH = 340;
const DOCUMENT_PANEL_ID = 'fraia-current-model-panel';

const DEFAULT_LABEL_VISIBILITY: ViewportLabelVisibility = {
  node: true,
  member: true,
  support: true,
  load: true,
};

type RenderPanel = 'groups' | null;
type BaseEditTool = 'select' | 'node' | 'member' | 'move' | 'split';
type ViewportViewMode = 'base' | 'scheme';
type ToolbarMenuId = 'member-settings' | 'snap-settings' | 'label-settings';
type Point3 = { x: number; y: number; z: number };
type AxisId = 'x' | 'y' | 'z';
type WorldPlane = 'xy' | 'xz' | 'yz';
type ViewportRay = { origin: Point3; direction: Point3 };
type SnapGuideSegment = { start: Point3; end: Point3; axis: AxisId | 'angle' };
type SnapInferenceMode = 'auto' | 'plane' | '3d';
type SnapLock = (
  | { kind: 'axis'; axis: AxisId; source: 'keyboard' | 'shift' }
  | { kind: 'direction'; direction: Point3; label: string; source: 'keyboard' | 'shift' }
  | { kind: 'plane'; plane: WorldPlane; source: 'keyboard' | 'shift' }
);
type SnapOptions = {
  endpoints: boolean;
  midpoints: boolean;
  nearest: boolean;
  grid: boolean;
  angles: boolean;
  axes: boolean;
  gridSize: number;
  angleIncrement: number;
};
type MemberDrawingOptions = {
  polygonMode: boolean;
};
type SnapEnablement = Pick<SnapOptions, 'endpoints' | 'midpoints' | 'nearest' | 'grid' | 'angles'>;
type SnapResult = {
  point: Point3;
  label: string;
  nodeId?: string;
  axis?: 'x' | 'y' | 'z' | 'angle';
  axes?: AxisId[];
  primaryAxis?: AxisId;
  angled?: boolean;
  direction?: Point3;
  inferredPlane?: WorldPlane;
  locked?: boolean;
  inferenceMode?: SnapInferenceMode;
  guideSegments?: SnapGuideSegment[];
  snapLabel?: string;
};

type SnapDisplay = {
  kind: 'end' | 'mid' | 'near' | 'axis' | 'angle';
};
type PendingMemberStart = {
  nodeId: string;
  point: Point3;
  source?: { kind: 'free' } | { kind: 'node'; id: string } | { kind: 'member'; id: string };
};
type MemberPreviewTopology = Pick<ViewportEditOverlay, 'previewMemberSegments' | 'previewNodes' | 'previewSplitMemberSegments' | 'memberSplitDimensions'>;
type SnapContext = {
  start?: Point3 | null;
  snapLock?: SnapLock | null;
  snapTarget?: ViewportPointerInfo['snapTarget'] | null;
  ray?: ViewportRay | null;
  inferenceMode?: SnapInferenceMode;
  inferencePlane?: WorldPlane | null;
  disabled?: boolean;
};

function baseBriefReady(state: WorkbenchState | null) {
  const brief = state?.baseModelBrief ?? state?.base_model_brief;
  return Boolean(brief?.readiness?.readyForSchemas ?? brief?.readiness?.ready_for_schemas);
}

function pendingDesignOptionIntentActions(state: WorkbenchState | null) {
  const agentState = state?.agentState ?? state?.agent_state;
  const session = agentState?.sessions?.find((candidate) => candidate.surface === 'pre_solve');
  for (const message of [...(session?.messages ?? [])].reverse()) {
    const actions = (message.proposedActions ?? message.proposed_actions ?? []).filter((action: any) => {
      const kind = action?.actionKind ?? action?.action_kind;
      return kind === 'update_planning_draft' && action?.field === 'coordination.designOptionIntents';
    });
    if (actions.length) return actions;
  }
  return [];
}

function targetKey(target: AgentTarget) {
  return `${target.kind}:${target.id}`;
}

function sameTarget(a: AgentTarget | null, b: AgentTarget | null) {
  return Boolean(a && b && a.kind === b.kind && a.id === b.id);
}

function addTargets(current: AgentTarget[], targets: AgentTarget[]) {
  const next = new Map(current.map((target) => [targetKey(target), target]));
  targets.forEach((target) => next.set(targetKey(target), target));
  return [...next.values()];
}

function removeTargets(current: AgentTarget[], targets: AgentTarget[]) {
  const removeKeys = new Set(targets.map(targetKey));
  return current.filter((target) => !removeKeys.has(targetKey(target)));
}

function attachedNodeTargetsForMember(scene: RenderScene, memberId: string): AgentTarget[] {
  const member = scene.members.find((item) => item.id === memberId);
  if (!member) return [];
  const nodeIds = new Set(scene.nodes.map((node) => node.id));
  return [memberStartId(member), memberEndId(member)]
    .filter((id): id is string => Boolean(id && nodeIds.has(id)))
    .map((id) => ({ kind: 'node', id }));
}

function expandMemberTargets(scene: RenderScene, targets: AgentTarget[]) {
  const expanded = targets.flatMap((target) => (
    target.kind === 'member'
      ? [target, ...attachedNodeTargetsForMember(scene, target.id)]
      : [target]
  ));
  return addTargets([], expanded);
}

function toggleExpandedTarget(scene: RenderScene, current: AgentTarget[], target: AgentTarget) {
  const expanded = expandMemberTargets(scene, [target]);
  if (!current.some((item) => sameTarget(item, target))) return addTargets(current, expanded);
  return expandMemberTargets(scene, removeTargets(current, expanded));
}

function toggleExpandedTargets(scene: RenderScene, current: AgentTarget[], targets: AgentTarget[]) {
  return expandMemberTargets(scene, toggleAgentTargets(current, expandMemberTargets(scene, targets)));
}

function loadStoredLabelVisibility(): ViewportLabelVisibility {
  if (typeof window === 'undefined') return DEFAULT_LABEL_VISIBILITY;
  try {
    const raw = window.localStorage.getItem(VIEWPORT_LABEL_VISIBILITY_STORAGE_KEY);
    if (!raw) return DEFAULT_LABEL_VISIBILITY;
    const parsed = JSON.parse(raw) as Partial<ViewportLabelVisibility>;
    return {
      node: Boolean(parsed.node),
      member: Boolean(parsed.member),
      support: Boolean(parsed.support),
      load: Boolean(parsed.load),
    };
  } catch {
    return DEFAULT_LABEL_VISIBILITY;
  }
}

function storeLabelVisibility(visibility: ViewportLabelVisibility) {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(VIEWPORT_LABEL_VISIBILITY_STORAGE_KEY, JSON.stringify(visibility));
}

function labelVisibilityActive(visibility: ViewportLabelVisibility) {
  return Object.values(visibility).some(Boolean);
}

function loadStoredLabelVisibilityToggleMemory(): ViewportLabelVisibility | null {
  if (typeof window === 'undefined') return null;
  try {
    const raw = window.localStorage.getItem(VIEWPORT_LABEL_VISIBILITY_TOGGLE_MEMORY_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<ViewportLabelVisibility>;
    const memory = {
      node: Boolean(parsed.node),
      member: Boolean(parsed.member),
      support: Boolean(parsed.support),
      load: Boolean(parsed.load),
    };
    return labelVisibilityActive(memory) ? memory : null;
  } catch {
    return null;
  }
}

function storeLabelVisibilityToggleMemory(visibility: ViewportLabelVisibility) {
  if (typeof window === 'undefined') return;
  if (!labelVisibilityActive(visibility)) return;
  window.localStorage.setItem(VIEWPORT_LABEL_VISIBILITY_TOGGLE_MEMORY_STORAGE_KEY, JSON.stringify(visibility));
}

function snapEnablement(options: SnapOptions): SnapEnablement {
  return {
    endpoints: options.endpoints,
    midpoints: options.midpoints,
    nearest: options.nearest,
    grid: options.grid,
    angles: options.angles,
  };
}

function snapEnablementActive(enablement: SnapEnablement) {
  return Object.values(enablement).some(Boolean);
}

function snapOptionsWithEnablement(options: SnapOptions, enablement: SnapEnablement): SnapOptions {
  return { ...options, ...enablement };
}

function loadStoredSnapToggleMemory(): SnapEnablement | null {
  if (typeof window === 'undefined') return null;
  try {
    const raw = window.localStorage.getItem(VIEWPORT_SNAP_TOGGLE_MEMORY_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<SnapEnablement>;
    const memory = {
      endpoints: Boolean(parsed.endpoints),
      midpoints: Boolean(parsed.midpoints),
      nearest: Boolean(parsed.nearest),
      grid: Boolean(parsed.grid),
      angles: Boolean(parsed.angles),
    };
    return snapEnablementActive(memory) ? memory : null;
  } catch {
    return null;
  }
}

function storeSnapToggleMemory(enablement: SnapEnablement) {
  if (typeof window === 'undefined') return;
  if (!snapEnablementActive(enablement)) return;
  window.localStorage.setItem(VIEWPORT_SNAP_TOGGLE_MEMORY_STORAGE_KEY, JSON.stringify(enablement));
}

const DEFAULT_SNAP_OPTIONS: SnapOptions = {
  endpoints: true,
  midpoints: true,
  nearest: true,
  grid: true,
  angles: true,
  axes: true,
  gridSize: 0.1,
  angleIncrement: 45,
};
const DEFAULT_MEMBER_DRAWING_OPTIONS: MemberDrawingOptions = {
  polygonMode: true,
};
const ALL_DISABLED_SNAP_ENABLEMENT: SnapEnablement = {
  endpoints: false,
  midpoints: false,
  nearest: false,
  grid: false,
  angles: false,
};
const ALL_VISIBLE_LABEL_VISIBILITY: ViewportLabelVisibility = {
  node: true,
  member: true,
  support: true,
  load: true,
};
const ALL_HIDDEN_LABEL_VISIBILITY: ViewportLabelVisibility = {
  node: false,
  member: false,
  support: false,
  load: false,
};

type WorkspaceSidebarProps = {
  width: number;
  onResizeStart: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onResizeValue: (width: number) => void;
  resizeMin: number;
  resizeMax: number;
  children: ReactNode;
};

function pointDistance(a: Point3, b: Point3) {
  return Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
}

function nextId(prefix: string, ids: string[]) {
  let next = 1;
  ids.forEach((id) => {
    if (!id.startsWith(prefix)) return;
    const value = Number(id.slice(prefix.length));
    if (Number.isInteger(value)) next = Math.max(next, value + 1);
  });
  return `${prefix}${next}`;
}

function normalizePoint(point: Point3): Point3 {
  const clean = (value: number) => Number(value.toFixed(6));
  return { x: clean(point.x), y: clean(point.y), z: clean(point.z) };
}

function memberEndpoints(scene: RenderScene, member: RenderMember) {
  const nodes = new Map(scene.nodes.map((node) => [node.id, node]));
  const start = nodes.get(memberStartId(member) ?? '');
  const end = nodes.get(memberEndId(member) ?? '');
  return start && end ? { start, end } : null;
}

function deleteOperationsForTargets(scene: RenderScene, targets: AgentTarget[]) {
  const memberIds = new Set(targets.filter((target) => target.kind === 'member').map((target) => target.id));
  const memberEndpointNodeIds = new Set<string>();
  scene.members.forEach((member) => {
    if (!memberIds.has(member.id)) return;
    const start = memberStartId(member);
    const end = memberEndId(member);
    if (start) memberEndpointNodeIds.add(start);
    if (end) memberEndpointNodeIds.add(end);
  });

  const nodeIds = new Set(
    targets
      .filter((target) => target.kind === 'node' && !memberEndpointNodeIds.has(target.id))
      .map((target) => target.id)
  );
  const supportIds = new Set(targets.filter((target) => target.kind === 'support').map((target) => target.id));
  const loadIds = new Set(targets.filter((target) => target.kind === 'load').map((target) => target.id));

  return [
    ...[...loadIds].map((id) => ({ kind: 'remove_load', id })),
    ...[...supportIds].map((id) => ({ kind: 'remove_support', id })),
    ...[...memberIds].map((id) => ({ kind: 'delete_member', id })),
    ...[...nodeIds].map((id) => ({ kind: 'delete_node', id })),
  ];
}

function targetExistsInScene(scene: RenderScene, target: AgentTarget) {
  if (target.kind === 'node') return scene.nodes.some((node) => node.id === target.id);
  if (target.kind === 'member') return scene.members.some((member) => member.id === target.id);
  if (target.kind === 'support') return scene.supports.some((support) => support.id === target.id);
  if (target.kind === 'load') return scene.loads.some((load) => load.id === target.id);
  if (target.kind === 'release') return scene.releases?.some((release) => release.id === target.id) ?? false;
  return true;
}

function reconciledSelectionAfterBaseEdit(
  previousScene: RenderScene,
  nextScene: RenderScene,
  currentTargets: AgentTarget[],
  operations: any[],
) {
  const removedTargets = new Set<string>();

  operations.forEach((operation) => {
    const kind = typeof operation?.kind === 'string' ? operation.kind : '';
    const id = typeof operation?.id === 'string' ? operation.id : '';
    if (!id) return;

    if (kind === 'delete_member') {
      removedTargets.add(targetKey({ kind: 'member', id }));
      attachedNodeTargetsForMember(previousScene, id).forEach((target) => {
        removedTargets.add(targetKey(target));
      });
    } else if (kind === 'delete_node') {
      removedTargets.add(targetKey({ kind: 'node', id }));
    } else if (kind === 'remove_support') {
      removedTargets.add(targetKey({ kind: 'support', id }));
    } else if (kind === 'remove_load') {
      removedTargets.add(targetKey({ kind: 'load', id }));
    } else if (kind === 'remove_release' || kind === 'delete_release') {
      removedTargets.add(targetKey({ kind: 'release', id }));
    }
  });

  const survivingTargets = currentTargets
    .filter((target) => !removedTargets.has(targetKey(target)))
    .filter((target) => targetExistsInScene(nextScene, target));
  return expandMemberTargets(nextScene, survivingTargets);
}

const AXIS_VECTORS: Record<AxisId, Point3> = {
  x: { x: 1, y: 0, z: 0 },
  y: { x: 0, y: 1, z: 0 },
  z: { x: 0, y: 0, z: 1 },
};

const PLANE_AXES: Record<WorldPlane, [AxisId, AxisId]> = {
  xy: ['x', 'y'],
  xz: ['x', 'z'],
  yz: ['y', 'z'],
};

const SNAP_GUIDE_LENGTH = 1.2;
const INFERENCE_COMPONENT_EPS = 0.01;
const CANONICAL_AXIS_SNAP_SCORE_BONUS = 0.1;
const CANONICAL_AXIS_CAPTURE_ANGLE_DEG = 5;
const CANONICAL_AXIS_CAPTURE_DISTANCE = 0.035;
const CANONICAL_AXIS_CAPTURE_RAY_DISTANCE = 0.1;
const CANONICAL_AXIS_GRID_OVERRIDE_MARGIN_RATIO = 0.55;
const MEMBER_INTERSECTION_EPS = 1e-7;

function dot(a: Point3, b: Point3) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

function sub(a: Point3, b: Point3): Point3 {
  return { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z };
}

function cross2(ax: number, ay: number, bx: number, by: number) {
  return ax * by - ay * bx;
}

function addScaled(point: Point3, direction: Point3, scale: number): Point3 {
  return {
    x: point.x + direction.x * scale,
    y: point.y + direction.y * scale,
    z: point.z + direction.z * scale,
  };
}

function normalizeVector(vector: Point3): Point3 | null {
  const length = Math.hypot(vector.x, vector.y, vector.z);
  if (length <= 1e-9) return null;
  return { x: vector.x / length, y: vector.y / length, z: vector.z / length };
}

function canonicalDirection(direction: Point3): Point3 | null {
  const normalized = normalizeVector(direction);
  if (!normalized) return null;
  const values = [normalized.x, normalized.y, normalized.z];
  const first = values.find((value) => Math.abs(value) > 1e-6) ?? 1;
  return first < 0
    ? { x: -normalized.x, y: -normalized.y, z: -normalized.z }
    : normalized;
}

function directionKey(direction: Point3) {
  const canonical = canonicalDirection(direction);
  if (!canonical) return '';
  return [canonical.x, canonical.y, canonical.z].map((value) => value.toFixed(4)).join(':');
}

function addDirectionCandidate(map: Map<string, Point3>, direction: Point3) {
  const canonical = canonicalDirection(direction);
  if (!canonical) return;
  const key = directionKey(canonical);
  if (key && !map.has(key)) map.set(key, canonical);
}

function closestCameraPlane(ray?: ViewportRay | null): WorldPlane {
  const direction = ray ? normalizeVector(ray.direction) : null;
  if (!direction) return 'xy';
  const ax = Math.abs(direction.x);
  const ay = Math.abs(direction.y);
  const az = Math.abs(direction.z);
  if (az >= ax && az >= ay) return 'xy';
  if (ay >= ax && ay >= az) return 'xz';
  return 'yz';
}

function planeSnapLabel(plane: WorldPlane) {
  if (plane === 'xy') return 'XY Frame';
  if (plane === 'xz') return 'XZ Plan';
  return 'YZ Side';
}

function planeForAxes(axes: AxisId[]): WorldPlane | null {
  const key = [...axes].sort().join('');
  if (key === 'xy') return 'xy';
  if (key === 'xz') return 'xz';
  if (key === 'yz') return 'yz';
  return null;
}

function outOfPlaneAxis(plane: WorldPlane): AxisId {
  if (plane === 'xy') return 'z';
  if (plane === 'xz') return 'y';
  return 'x';
}

function planeDirection(plane: WorldPlane, a: number, b: number): Point3 {
  if (plane === 'xy') return { x: a, y: b, z: 0 };
  if (plane === 'xz') return { x: a, y: 0, z: b };
  return { x: 0, y: a, z: b };
}

function snapDirectionCandidates(incrementDeg: number, plane?: WorldPlane): Point3[] {
  const directions = new Map<string, Point3>();
  const increment = Math.max(1, Math.min(90, Math.abs(incrementDeg) || 45));
  const radians = (deg: number) => (deg * Math.PI) / 180;

  if (plane) {
    PLANE_AXES[plane].forEach((axis) => addDirectionCandidate(directions, AXIS_VECTORS[axis]));
    for (let angle = 0; angle < 180; angle += increment) {
      addDirectionCandidate(directions, planeDirection(plane, Math.cos(radians(angle)), Math.sin(radians(angle))));
    }
    return [...directions.values()];
  }

  Object.values(AXIS_VECTORS).forEach((axis) => addDirectionCandidate(directions, axis));
  [-1, 0, 1].forEach((x) => {
    [-1, 0, 1].forEach((y) => {
      [-1, 0, 1].forEach((z) => {
        if (x || y || z) addDirectionCandidate(directions, { x, y, z });
      });
    });
  });
  for (let angle = 0; angle < 180; angle += increment) {
    const c = Math.cos(radians(angle));
    const s = Math.sin(radians(angle));
    addDirectionCandidate(directions, { x: c, y: s, z: 0 });
    addDirectionCandidate(directions, { x: c, y: 0, z: s });
    addDirectionCandidate(directions, { x: 0, y: c, z: s });
  }
  for (let elevation = -90; elevation <= 90; elevation += increment) {
    const elev = radians(elevation);
    const ce = Math.cos(elev);
    const se = Math.sin(elev);
    for (let azimuth = 0; azimuth < 360; azimuth += increment) {
      const az = radians(azimuth);
      addDirectionCandidate(directions, {
        x: ce * Math.cos(az),
        y: ce * Math.sin(az),
        z: se,
      });
    }
  }
  return [...directions.values()];
}

function isCameraParallelDirection(direction: Point3, ray?: ViewportRay | null) {
  const candidate = normalizeVector(direction);
  const rayDirection = ray ? normalizeVector(ray.direction) : null;
  if (!candidate || !rayDirection) return false;
  return Math.abs(dot(candidate, rayDirection)) > 0.985;
}

function pointOnCameraPlane(start: Point3, raw: Point3, ray?: ViewportRay | null): Point3 {
  if (!ray) return raw;
  const normal = normalizeVector(ray.direction);
  if (!normal) return raw;
  const denom = dot(ray.direction, normal);
  if (Math.abs(denom) <= 1e-9) return raw;
  const t = dot(sub(start, ray.origin), normal) / denom;
  return addScaled(ray.origin, ray.direction, t);
}

function planeNormalVector(plane: WorldPlane): Point3 {
  return AXIS_VECTORS[outOfPlaneAxis(plane)];
}

function pointOnWorldPlane(start: Point3, raw: Point3, plane: WorldPlane, ray?: ViewportRay | null): Point3 | null {
  if (!ray) return raw;
  const normal = planeNormalVector(plane);
  const denom = dot(ray.direction, normal);
  if (Math.abs(denom) <= 1e-8) return null;
  const t = dot(sub(start, ray.origin), normal) / denom;
  return addScaled(ray.origin, ray.direction, t);
}

function pointDistanceToRay(point: Point3, ray?: ViewportRay | null) {
  const direction = ray ? normalizeVector(ray.direction) : null;
  if (!ray || !direction) return Number.POSITIVE_INFINITY;
  const offset = sub(point, ray.origin);
  const t = dot(offset, direction);
  const closest = addScaled(ray.origin, direction, t);
  return pointDistance(point, closest);
}

function closestPointOnDirection(start: Point3, raw: Point3, direction: Point3, ray?: ViewportRay | null) {
  const candidate = normalizeVector(direction);
  if (!candidate) return { point: raw, distance: Number.POSITIVE_INFINITY, t: 0 };
  if (!ray) {
    const t = dot(sub(raw, start), candidate);
    return { point: addScaled(start, candidate, t), distance: 0, t };
  }
  const rayDirection = normalizeVector(ray.direction);
  if (!rayDirection) return { point: raw, distance: Number.POSITIVE_INFINITY, t: 0 };
  const w0 = sub(start, ray.origin);
  const b = dot(candidate, rayDirection);
  const d = dot(candidate, w0);
  const e = dot(rayDirection, w0);
  const denom = 1 - b * b;
  if (Math.abs(denom) <= 1e-8) {
    const t = dot(sub(raw, start), candidate);
    return { point: addScaled(start, candidate, t), distance: 0, t };
  }
  const t = (b * e - d) / denom;
  const rayT = (e - b * d) / denom;
  const point = addScaled(start, candidate, t);
  const pointOnRay = addScaled(ray.origin, rayDirection, rayT);
  return { point, distance: pointDistance(point, pointOnRay), t };
}

function axesForDirection(direction: Point3): AxisId[] {
  const normalized = normalizeVector(direction);
  if (!normalized) return [];
  return (['x', 'y', 'z'] as AxisId[]).filter((axis) => Math.abs(normalized[axis]) > 0.18);
}

function primaryAxisForDirection(direction: Point3): AxisId {
  const normalized = normalizeVector(direction) ?? AXIS_VECTORS.x;
  return (['x', 'y', 'z'] as AxisId[]).reduce((best, axis) => (
    Math.abs(normalized[axis]) > Math.abs(normalized[best]) ? axis : best
  ), 'x');
}

function signedAxisVector(axis: AxisId, sign: number): Point3 {
  return {
    x: axis === 'x' ? sign : 0,
    y: axis === 'y' ? sign : 0,
    z: axis === 'z' ? sign : 0,
  };
}

function guideSegmentsFor(start: Point3, point: Point3, direction: Point3, angled: boolean): SnapGuideSegment[] {
  const segments: SnapGuideSegment[] = [];
  if (pointDistance(start, point) <= 1e-6) return segments;
  const normalized = normalizeVector(direction);
  if (!normalized) return segments;
  axesForDirection(direction).forEach((axis) => {
    const sign = normalized[axis] >= 0 ? 1 : -1;
    const cue = signedAxisVector(axis, sign);
    segments.push({ start: addScaled(point, cue, -SNAP_GUIDE_LENGTH), end: point, axis });
  });
  if (angled) segments.push({ start: addScaled(point, normalized, -SNAP_GUIDE_LENGTH), end: point, axis: 'angle' });
  return segments;
}

function snapTextFor(direction: Point3, axes: AxisId[], angled: boolean, incrementDeg: number, inferredPlane?: WorldPlane) {
  const axisText = (axes.length ? axes : [primaryAxisForDirection(direction)]).map((axis) => axis.toUpperCase()).join('+');
  if (!angled) return axisText;
  const normalized = normalizeVector(direction);
  if (!normalized) return axisText;
  if (axes.length === 3) {
    const plane = inferredPlane ?? 'xy';
    return `${planeSnapLabel(plane)} + ${outOfPlaneAxis(plane).toUpperCase()}`;
  }
  if (axes.length !== 2) return axisText;
  const primary = primaryAxisForDirection(direction);
  const angle = Math.round((Math.acos(Math.min(1, Math.abs(normalized[primary]))) * 180) / Math.PI / Math.max(1, incrementDeg)) * Math.max(1, incrementDeg);
  return `${planeSnapLabel(planeForAxes(axes) ?? inferredPlane ?? 'xy')} ${angle}°`;
}

function snapLabelForMode(label: string, mode: SnapInferenceMode | undefined, plane: WorldPlane | undefined, locked: boolean) {
  if (locked && mode === 'plane' && plane) return `Locked ${label} - ${planeSnapLabel(plane)}`;
  if (locked) return `Locked ${label}`;
  if (mode === 'plane' && plane) return `${label} - ${planeSnapLabel(plane)}`;
  if (mode === '3d') return `${label} - 3D`;
  return label;
}

function directionSnapResult(start: Point3, direction: Point3, point: Point3, incrementDeg: number, options: { locked?: boolean; inferredPlane?: WorldPlane; inferenceMode?: SnapInferenceMode } = {}): SnapResult {
  const axes = axesForDirection(direction);
  const primaryAxis = primaryAxisForDirection(direction);
  const angled = axes.length !== 1;
  const cleanPoint = normalizePoint(point);
  const canonical = canonicalDirection(direction) ?? direction;
  const baseLabel = snapTextFor(direction, axes, angled, incrementDeg, options.inferredPlane);
  const locked = Boolean(options.locked);
  const snapLabel = snapLabelForMode(baseLabel, options.inferenceMode, options.inferredPlane, locked);
  return {
    point: cleanPoint,
    label: locked ? snapLabel : angled ? baseLabel : `${primaryAxis.toUpperCase()} direction`,
    axis: angled ? 'angle' : primaryAxis,
    axes,
    primaryAxis,
    angled,
    direction: canonical,
    inferredPlane: options.inferredPlane,
    locked,
    inferenceMode: options.inferenceMode,
    guideSegments: guideSegmentsFor(start, cleanPoint, canonical, angled),
    snapLabel,
  };
}

function formatDimensionSnapDistance(distance: number) {
  const abs = Math.abs(distance);
  if (abs < 1) return `${Math.round(abs * 1000)} mm`;
  return `${Number(abs.toFixed(3))} m`;
}

function applyDimensionLengthSnap(start: Point3, result: SnapResult, increment: number): SnapResult {
  if (!(increment > 0)) return result;
  if (result.nodeId || result.label === 'Endpoint' || result.label === 'Midpoint' || result.label === 'On Member') return result;

  const inferredDirection = result.direction ?? normalizeVector(sub(result.point, start));
  const direction = inferredDirection ? normalizeVector(inferredDirection) : null;
  if (!direction) return result;

  const signedLength = dot(sub(result.point, start), direction);
  if (!Number.isFinite(signedLength)) return result;

  const snappedLength = Math.round(signedLength / increment) * increment;
  const snappedPoint = normalizePoint(addScaled(start, direction, snappedLength));
  const distanceLabel = formatDimensionSnapDistance(snappedLength);
  const snapLabel = result.snapLabel ? `${result.snapLabel} / ${distanceLabel}` : `Dimension ${distanceLabel}`;
  const guideDirection = result.direction ?? direction;
  const angled = result.axis === 'angle' || axesForDirection(guideDirection).length > 1;

  return {
    ...result,
    point: snappedPoint,
    label: result.axis ? result.label : 'Dimension',
    direction: guideDirection,
    guideSegments: guideSegmentsFor(start, snappedPoint, guideDirection, angled),
    snapLabel,
  };
}

function dimensionGridResult(start: Point3, planePoint: Point3, plane: WorldPlane, increment: number): SnapResult {
  const snapped = { ...planePoint };
  PLANE_AXES[plane].forEach((axis) => {
    snapped[axis] = start[axis] + Math.round((planePoint[axis] - start[axis]) / increment) * increment;
  });
  snapped[outOfPlaneAxis(plane)] = start[outOfPlaneAxis(plane)];

  const point = normalizePoint(snapped);
  const vector = sub(point, start);
  const direction = normalizeVector(vector) ?? undefined;
  const axes = coordinateComponentAxes(vector);
  const axis = axes.length === 1 ? axes[0] : direction ? 'angle' as const : undefined;
  const angled = axis === 'angle';
  const distance = pointDistance(start, point);

  return {
    point,
    label: axis && !angled ? `${axis.toUpperCase()} direction` : 'Dimension Grid',
    axis,
    axes,
    primaryAxis: direction ? primaryAxisForDirection(direction) : undefined,
    angled,
    direction,
    inferredPlane: plane,
    guideSegments: direction ? guideSegmentsFor(start, point, direction, angled) : undefined,
    snapLabel: `Dimension ${formatDimensionSnapDistance(distance)}`,
  };
}

function snapToDimensionGrid(start: Point3, raw: Point3, increment: number, ray?: ViewportRay | null): SnapResult {
  if (!(increment > 0)) return { point: raw, label: 'Point' };
  const preferredPlane = closestCameraPlane(ray);
  const candidates = (Object.keys(PLANE_AXES) as WorldPlane[]).flatMap((plane) => {
    const planePoint = pointOnWorldPlane(start, raw, plane, ray);
    if (!planePoint) return [];
    const result = dimensionGridResult(start, planePoint, plane, increment);
    const pointerDistance = ray ? pointDistanceToRay(result.point, ray) : pointDistance(raw, result.point);
    const gridDistance = pointDistance(planePoint, result.point);
    const planePenalty = plane === preferredPlane ? 0 : increment * 0.02;
    return [{ result, score: pointerDistance + gridDistance * 0.15 + planePenalty }];
  });
  return candidates.sort((a, b) => a.score - b.score)[0]?.result ?? {
    point: normalizePoint(raw),
    label: 'Point',
    direction: normalizeVector(sub(raw, start)) ?? undefined,
  };
}

function snapToDimensionGridOnPlane(start: Point3, raw: Point3, increment: number, plane: WorldPlane, ray?: ViewportRay | null): SnapResult {
  if (!(increment > 0)) return { point: raw, label: 'Point' };
  const planePoint = pointOnWorldPlane(start, raw, plane, ray) ?? projectionPoint(start, pointOnCameraPlane(start, raw, ray), plane);
  return dimensionGridResult(start, planePoint, plane, increment);
}

function chooseDimensionSnap(start: Point3, raw: Point3, directed: SnapResult, grid: SnapResult, increment: number, ray?: ViewportRay | null, preferCanonicalAxes = true): SnapResult {
  if (!(increment > 0) || pointDistance(start, grid.point) <= 1e-6) return directed;
  if (preferCanonicalAxes && (directed.axis === 'x' || directed.axis === 'y' || directed.axis === 'z')) return directed;
  const intent = pointOnCameraPlane(start, raw, ray);
  const directedDistance = ray ? pointDistanceToRay(directed.point, ray) : pointDistance(intent, directed.point);
  const gridDistance = ray ? pointDistanceToRay(grid.point, ray) : pointDistance(intent, grid.point);
  const gridOverrideMargin = preferCanonicalAxes && (directed.axis === 'x' || directed.axis === 'y' || directed.axis === 'z')
    ? increment * CANONICAL_AXIS_GRID_OVERRIDE_MARGIN_RATIO
    : increment * 0.04;
  if (gridDistance + gridOverrideMargin < directedDistance) return grid;
  return directed;
}

function snapToAxis(start: Point3, raw: Point3, axis: AxisId, ray?: ViewportRay | null, locked = false): SnapResult {
  const snapped = closestPointOnDirection(start, raw, AXIS_VECTORS[axis], ray);
  return directionSnapResult(start, AXIS_VECTORS[axis], snapped.point, 90, { locked });
}

function snapToDirection(start: Point3, raw: Point3, direction: Point3, incrementDeg: number, ray?: ViewportRay | null, locked = false): SnapResult {
  const snapped = closestPointOnDirection(start, raw, direction, ray);
  return directionSnapResult(start, direction, snapped.point, incrementDeg, { locked });
}

function directionComponentAxes(direction: Point3) {
  const normalized = normalizeVector(direction);
  if (!normalized) return [];
  return (['x', 'y', 'z'] as AxisId[]).filter((axis) => Math.abs(normalized[axis]) > INFERENCE_COMPONENT_EPS);
}

function canonicalAxisForDirection(direction: Point3): AxisId | null {
  const axes = directionComponentAxes(direction);
  return axes.length === 1 ? axes[0] : null;
}

function coordinateComponentAxes(delta: Point3) {
  return (['x', 'y', 'z'] as AxisId[]).filter((axis) => Math.abs(delta[axis]) > 1e-6);
}

function directionComplexity(direction: Point3, preferredPlane: WorldPlane) {
  const componentAxes = directionComponentAxes(direction);
  if (componentAxes.length <= 1) return 0;
  if (componentAxes.length >= 3) return 3;
  return planeForAxes(componentAxes) === preferredPlane ? 1 : 2;
}

function autoSimplicityPenalty(complexity: number) {
  if (complexity <= 0) return 0;
  if (complexity === 1) return 0.025;
  if (complexity === 2) return 0.06;
  return 0.14;
}

function projectDirectionToCameraPlane(direction: Point3, ray?: ViewportRay | null) {
  const normalized = normalizeVector(direction);
  if (!normalized) return null;
  const normal = ray ? normalizeVector(ray.direction) : null;
  if (!normal) return normalized;
  return normalizeVector(addScaled(normalized, normal, -dot(normalized, normal)));
}

function screenAngleDegBetween(a: Point3, b: Point3) {
  const normalizedA = normalizeVector(a);
  const normalizedB = normalizeVector(b);
  if (!normalizedA || !normalizedB) return Number.POSITIVE_INFINITY;
  const lineDot = Math.min(1, Math.max(0, Math.abs(dot(normalizedA, normalizedB))));
  return (Math.acos(lineDot) * 180) / Math.PI;
}

function snapToPlaneDirection(start: Point3, raw: Point3, incrementDeg: number, ray?: ViewportRay | null, inferenceMode: SnapInferenceMode = 'auto', planeOverride?: WorldPlane, locked = false, preferCanonicalAxes = true): SnapResult {
  const plane = planeOverride ?? closestCameraPlane(ray);
  const candidateDirections = inferenceMode === 'plane'
    ? snapDirectionCandidates(incrementDeg, plane)
    : snapDirectionCandidates(incrementDeg);
  const intentPoint = pointOnCameraPlane(start, raw, ray);
  const intentVector = sub(intentPoint, start);
  const intentLength = pointDistance(start, intentPoint);
  const intentDirection = normalizeVector(intentVector);
  const rayDirection = ray ? normalizeVector(ray.direction) : null;
	let scored: Array<{
    direction: Point3;
    point: Point3;
    score: number;
    screenAngleDeg: number;
    axisDistance: number;
    normalizedRayDistance: number;
    complexity: number;
  }> = [];
  for (const direction of candidateDirections) {
    if (!locked && inferenceMode !== '3d' && isCameraParallelDirection(direction, ray)) continue;
    const snapped = closestPointOnDirection(start, raw, direction, ray);
    const projectedDirection = projectDirectionToCameraPlane(direction, ray);
    if (!projectedDirection || !intentDirection) continue;
    const screenAngleDeg = screenAngleDegBetween(intentDirection, projectedDirection);
    if (!Number.isFinite(screenAngleDeg)) continue;
    const axisDistance = snapped.distance;
    const normalizedRayDistance = snapped.distance / Math.max(intentLength, 1);
    const complexity = directionComplexity(direction, plane);
    const hiddenDepthPenalty = inferenceMode === 'auto' && rayDirection
      ? 0.06 * Math.abs(dot(normalizeVector(direction) ?? direction, rayDirection))
      : 0;
    const simplicityPenalty = inferenceMode === 'auto'
      ? autoSimplicityPenalty(complexity)
      : axesForDirection(direction).length * 0.001;
    const axisPreference = preferCanonicalAxes && canonicalAxisForDirection(direction)
      ? -CANONICAL_AXIS_SNAP_SCORE_BONUS
      : 0;
    const score =
      3 * (screenAngleDeg / 90) +
      0.35 * normalizedRayDistance +
      simplicityPenalty +
      hiddenDepthPenalty +
      axisPreference;
    scored.push({ direction, point: snapped.point, score, screenAngleDeg, axisDistance, normalizedRayDistance, complexity });
  }
  if (!scored.length) return { point: raw, label: 'Point' };
  scored = scored.sort((a, b) => a.score - b.score);
  let best = scored[0];
  let capturedAxis: (typeof scored)[number] | undefined;
  if (preferCanonicalAxes && !locked) {
    capturedAxis = scored
      .filter((candidate) => (
        Boolean(canonicalAxisForDirection(candidate.direction)) &&
        (
          candidate.screenAngleDeg <= CANONICAL_AXIS_CAPTURE_ANGLE_DEG ||
          candidate.axisDistance <= CANONICAL_AXIS_CAPTURE_DISTANCE
        ) &&
        candidate.normalizedRayDistance <= CANONICAL_AXIS_CAPTURE_RAY_DISTANCE
      ))
      .sort((a, b) => a.axisDistance - b.axisDistance || a.screenAngleDeg - b.screenAngleDeg || a.normalizedRayDistance - b.normalizedRayDistance)[0];
    if (capturedAxis) best = capturedAxis;
  }
  if (!capturedAxis && !locked) {
    const nonAxisBest = scored.find((candidate) => !canonicalAxisForDirection(candidate.direction));
    if (nonAxisBest) best = nonAxisBest;
  }
  if (inferenceMode === 'auto') {
    const simpler = scored
      .filter((candidate) => (
        (!canonicalAxisForDirection(candidate.direction) || candidate === capturedAxis || locked) &&
        candidate.complexity < best.complexity &&
        candidate.screenAngleDeg <= best.screenAngleDeg + 5 &&
        candidate.normalizedRayDistance <= best.normalizedRayDistance + 0.08
      ))
      .sort((a, b) => a.complexity - b.complexity || a.score - b.score)[0];
    if (simpler) best = simpler;
  }
  if (!best) return { point: raw, label: 'Point' };
  return directionSnapResult(start, best.direction, best.point, incrementDeg, { inferredPlane: plane, inferenceMode, locked });
}

function planarCursorPoint(start: Point3, raw: Point3, ray?: ViewportRay | null, plane = closestCameraPlane(ray)): SnapResult {
  const cameraPlanePoint = pointOnCameraPlane(start, raw, ray);
  const point = normalizePoint(projectionPoint(start, cameraPlanePoint, plane));
  const direction = normalizeVector(sub(point, start)) ?? undefined;
  const axes = coordinateComponentAxes(sub(point, start));
  return {
    point,
    label: 'Point',
    axis: axes.length === 1 ? axes[0] : direction ? 'angle' : undefined,
    axes,
    primaryAxis: direction ? primaryAxisForDirection(direction) : undefined,
    angled: axes.length > 1,
    direction,
    inferredPlane: plane,
  };
}

function memberTargetFitsTwoAxes(start: Point3, point: Point3) {
  return coordinateComponentAxes(sub(point, start)).length <= 2;
}

function memberObjectSnapResult(start: Point3, point: Point3, label: string, nodeId?: string): SnapResult {
  const vector = sub(point, start);
  const axes = coordinateComponentAxes(vector);
  return {
    point,
    label,
    nodeId,
    axis: axes.length === 1 ? axes[0] : axes.length > 1 ? 'angle' : undefined,
    axes,
    primaryAxis: axes.length ? primaryAxisForDirection(vector) : undefined,
    angled: axes.length > 1,
    inferredPlane: axes.length === 2 ? planeForAxes(axes) ?? undefined : undefined,
  };
}

function snapPoint(scene: RenderScene, rawPoint: Point3, options: SnapOptions, context: SnapContext): SnapResult {
  const raw = normalizePoint(rawPoint);
  if (context.disabled) return { point: raw, label: 'Point' };
  const explicitLock = Boolean(context.start && context.snapLock);
  const snapLength = (result: SnapResult) => (
    context.start && options.grid ? applyDimensionLengthSnap(context.start, result, options.gridSize) : result
  );

  if (context.start && context.snapLock?.kind === 'axis') {
    return snapLength(snapToAxis(context.start, raw, context.snapLock.axis, context.ray, true));
  }

  if (context.start && context.snapLock?.kind === 'direction') {
    return snapLength(snapToDirection(context.start, raw, context.snapLock.direction, options.angleIncrement, context.ray, true));
  }

  if (context.start && context.snapLock?.kind === 'plane') {
    return snapLength(snapToPlaneDirection(context.start, raw, options.angleIncrement, context.ray, 'plane', context.snapLock.plane, true, options.axes));
  }

  if (!explicitLock && options.endpoints && context.snapTarget?.kind === 'node') {
    const node = scene.nodes.find((item) => item.id === context.snapTarget?.id);
    if (node) return { point: normalizePoint({ x: node.x, y: node.y, z: node.z }), label: 'Endpoint', nodeId: node.id };
  }

  if (!explicitLock && options.midpoints && context.snapTarget?.kind === 'memberMidpoint') {
    const member = scene.members.find((item) => item.id === context.snapTarget?.id);
    const endpoints = member ? memberEndpoints(scene, member) : null;
    if (endpoints) {
      return {
        point: normalizePoint({
          x: (endpoints.start.x + endpoints.end.x) / 2,
          y: (endpoints.start.y + endpoints.end.y) / 2,
          z: (endpoints.start.z + endpoints.end.z) / 2,
        }),
        label: 'Midpoint',
      };
    }
  }

  if (!explicitLock && options.nearest && context.snapTarget?.kind === 'member') {
    const member = scene.members.find((item) => item.id === context.snapTarget?.id);
    const endpoints = member ? memberEndpoints(scene, member) : null;
    if (endpoints) {
      const ax = endpoints.start.x;
      const ay = endpoints.start.y;
      const az = endpoints.start.z;
      const bx = endpoints.end.x;
      const by = endpoints.end.y;
      const bz = endpoints.end.z;
      const vx = bx - ax;
      const vy = by - ay;
      const vz = bz - az;
      const lengthSquared = vx * vx + vy * vy + vz * vz;
      if (lengthSquared > 1e-9) {
        const t = Math.max(0, Math.min(1, ((raw.x - ax) * vx + (raw.y - ay) * vy + (raw.z - az) * vz) / lengthSquared));
        const point = { x: ax + vx * t, y: ay + vy * t, z: az + vz * t };
        return { point: normalizePoint(point), label: 'On Member' };
      }
    }
  }

  if (context.start && options.angles) {
    const directed = snapLength(snapToPlaneDirection(context.start, raw, options.angleIncrement, context.ray, context.inferenceMode ?? 'auto', undefined, false, options.axes));
    if (!options.grid) return directed;
    const grid = snapToDimensionGrid(context.start, raw, options.gridSize, context.ray);
    return chooseDimensionSnap(context.start, raw, directed, grid, options.gridSize, context.ray, options.axes);
  }

  if (context.start) {
    const point = normalizePoint(pointOnCameraPlane(context.start, raw, context.ray));
    if (options.grid) return snapToDimensionGrid(context.start, raw, options.gridSize, context.ray);
    return { point, label: 'Point', direction: normalizeVector(sub(point, context.start)) ?? undefined };
  }

  if (options.grid && options.gridSize > 0) {
    return {
      point: normalizePoint({
        x: Math.round(raw.x / options.gridSize) * options.gridSize,
        y: Math.round(raw.y / options.gridSize) * options.gridSize,
        z: Math.round(raw.z / options.gridSize) * options.gridSize,
      }),
      label: 'Grid',
    };
  }

  return { point: raw, label: 'Point' };
}

function snapMemberPoint(scene: RenderScene, rawPoint: Point3, options: SnapOptions, context: SnapContext): SnapResult {
  const raw = normalizePoint(rawPoint);
  const start = context.start ?? null;
  if (!start) return snapPoint(scene, rawPoint, options, context);

  const explicitPlane = context.inferencePlane ?? null;
  const plane = explicitPlane ?? closestCameraPlane(context.ray);
  const planarFallback = () => (
    !context.disabled && options.grid
      ? snapToDimensionGridOnPlane(start, raw, options.gridSize, plane, context.ray)
      : planarCursorPoint(start, raw, context.ray, plane)
  );
  const snapLength = (result: SnapResult) => (
    options.grid ? applyDimensionLengthSnap(start, result, options.gridSize) : result
  );
  const explicitLock = Boolean(context.snapLock);

  if (context.snapLock?.kind === 'axis') {
    return snapLength(snapToAxis(start, raw, context.snapLock.axis, context.ray, true));
  }

  if (context.snapLock?.kind === 'direction') {
    if (coordinateComponentAxes(context.snapLock.direction).length <= 2) {
      return snapLength(snapToDirection(start, raw, context.snapLock.direction, options.angleIncrement, context.ray, true));
    }
    return planarFallback();
  }

  if (context.snapLock?.kind === 'plane') {
    return snapLength(snapToPlaneDirection(start, raw, options.angleIncrement, context.ray, 'plane', context.snapLock.plane, true, options.axes));
  }

  if (!context.disabled && !explicitLock && options.endpoints && context.snapTarget?.kind === 'node') {
    const node = scene.nodes.find((item) => item.id === context.snapTarget?.id);
    if (node) {
      const point = normalizePoint({ x: node.x, y: node.y, z: node.z });
      if (memberTargetFitsTwoAxes(start, point)) return memberObjectSnapResult(start, point, 'Endpoint', node.id);
    }
  }

  if (!context.disabled && !explicitLock && options.midpoints && context.snapTarget?.kind === 'memberMidpoint') {
    const member = scene.members.find((item) => item.id === context.snapTarget?.id);
    const endpoints = member ? memberEndpoints(scene, member) : null;
    if (endpoints) {
      const point = normalizePoint({
        x: (endpoints.start.x + endpoints.end.x) / 2,
        y: (endpoints.start.y + endpoints.end.y) / 2,
        z: (endpoints.start.z + endpoints.end.z) / 2,
      });
      if (memberTargetFitsTwoAxes(start, point)) return memberObjectSnapResult(start, point, 'Midpoint');
    }
  }

  if (!context.disabled && !explicitLock && options.nearest && context.snapTarget?.kind === 'member') {
    const member = scene.members.find((item) => item.id === context.snapTarget?.id);
    const endpoints = member ? memberEndpoints(scene, member) : null;
    if (endpoints) {
      const ax = endpoints.start.x;
      const ay = endpoints.start.y;
      const az = endpoints.start.z;
      const bx = endpoints.end.x;
      const by = endpoints.end.y;
      const bz = endpoints.end.z;
      const vx = bx - ax;
      const vy = by - ay;
      const vz = bz - az;
      const lengthSquared = vx * vx + vy * vy + vz * vz;
      if (lengthSquared > 1e-9) {
        const t = Math.max(0, Math.min(1, ((raw.x - ax) * vx + (raw.y - ay) * vy + (raw.z - az) * vz) / lengthSquared));
        const point = normalizePoint({ x: ax + vx * t, y: ay + vy * t, z: az + vz * t });
        if (memberTargetFitsTwoAxes(start, point)) return memberObjectSnapResult(start, point, 'On Member');
      }
    }
  }

  if (options.angles && !context.disabled) {
    const directed = snapLength(snapToPlaneDirection(
      start,
      raw,
      options.angleIncrement,
      context.ray,
      explicitPlane ? 'plane' : 'auto',
      explicitPlane ?? undefined,
      false,
      options.axes,
    ));
    if (!options.grid) return directed;
    const grid = snapToDimensionGridOnPlane(start, raw, options.gridSize, plane, context.ray);
    return chooseDimensionSnap(start, raw, directed, grid, options.gridSize, context.ray, options.axes);
  }

  return planarFallback();
}

function snapDisplayFor(result: SnapResult | null): SnapDisplay | null {
  if (!result) return null;
  if (result.label === 'Endpoint') return { kind: 'end' };
  if (result.label === 'Midpoint') return { kind: 'mid' };
  if (result.label === 'On Member') return { kind: 'near' };
  if (result.axis === 'x' || result.axis === 'y' || result.axis === 'z') return { kind: 'axis' };
  if (result.axis === 'angle') return { kind: 'angle' };
  return null;
}

function memberInferenceLabel(start: Point3, result: SnapResult | null): ViewportEditOverlay['inferenceLabel'] | undefined {
  if (!result?.direction || pointDistance(start, result.point) <= 1e-6) return undefined;
  if (result.axis === 'x' || result.axis === 'y' || result.axis === 'z') {
    const axis = result.axis;
    return {
      kind: 'axis',
      anchor: start,
      axis,
      label: axis.toUpperCase(),
    };
  }
  return undefined;
}

function projectionPoint(start: Point3, end: Point3, plane: WorldPlane): Point3 {
  if (plane === 'xy') return { x: end.x, y: end.y, z: start.z };
  if (plane === 'xz') return { x: end.x, y: start.y, z: end.z };
  return { x: start.x, y: end.y, z: end.z };
}

function projectionAngles(vector: Point3, plane: WorldPlane): Array<{ axis: AxisId; angleDeg: number }> {
  const [a, b] = PLANE_AXES[plane];
  const absA = Math.abs(vector[a]);
  const absB = Math.abs(vector[b]);
  const length = Math.hypot(absA, absB);
  if (length <= 1e-9) return [];
  return [a, b].map((axis) => ({
    axis,
    angleDeg: Math.round((Math.acos(Math.min(1, Math.abs(vector[axis]) / length)) * 180) / Math.PI),
  }));
}

function memberProjectionGuide(start: Point3, result: SnapResult | null): ViewportEditOverlay['projectionGuide'] | undefined {
  if (!result || pointDistance(start, result.point) <= 1e-6) return undefined;
  if (result.axis === 'x' || result.axis === 'y' || result.axis === 'z') return undefined;
  const vector = sub(result.point, start);
  const normalized = normalizeVector(result.direction ?? vector);
  if (!normalized) return undefined;
  const componentAxes = coordinateComponentAxes(vector);
  if (componentAxes.length !== 2) return undefined;
  const plane = result.inferredPlane ?? planeForAxes(componentAxes);
  if (!plane) return undefined;
  const projectedEnd = projectionPoint(start, result.point, plane);
  if (pointDistance(start, projectedEnd) <= 1e-6) return undefined;
  const angles = projectionAngles(sub(projectedEnd, start), plane);
  if (angles.length !== 2) return undefined;
  const firstAxis = PLANE_AXES[plane].find((axis) => Math.abs(vector[axis]) > 1e-6);
  const firstAngle = angles.find((entry) => entry.axis === firstAxis);
  const offsetAxis = outOfPlaneAxis(plane);
  const outOfPlaneDelta = Math.abs(result.point[offsetAxis] - start[offsetAxis]);
  return {
    guide: {
      plane,
      start,
      projectedEnd,
      realEnd: result.point,
      angles,
      outOfPlaneAxis: outOfPlaneDelta > 1e-6 ? offsetAxis : undefined,
      angle: firstAxis && firstAngle ? { axis: firstAxis, angleDeg: firstAngle.angleDeg } : undefined,
    },
  };
}

function sameZ(a: Point3, b: Point3) {
  return Math.abs(a.z - b.z) <= MEMBER_INTERSECTION_EPS;
}

function isSegmentParameter(value: number) {
  return value >= -MEMBER_INTERSECTION_EPS && value <= 1 + MEMBER_INTERSECTION_EPS;
}

function isSegmentInterior(value: number) {
  return value > MEMBER_INTERSECTION_EPS && value < 1 - MEMBER_INTERSECTION_EPS;
}

function clampUnit(value: number) {
  return Math.min(1, Math.max(0, value));
}

function segmentIntersectionXY(a: Point3, b: Point3, c: Point3, d: Point3) {
  if (!sameZ(a, b) || !sameZ(c, d) || Math.abs(a.z - c.z) > MEMBER_INTERSECTION_EPS) return null;
  const abx = b.x - a.x;
  const aby = b.y - a.y;
  const cdx = d.x - c.x;
  const cdy = d.y - c.y;
  const denom = cross2(abx, aby, cdx, cdy);
  if (Math.abs(denom) <= MEMBER_INTERSECTION_EPS) return null;
  const acx = c.x - a.x;
  const acy = c.y - a.y;
  const newT = cross2(acx, acy, cdx, cdy) / denom;
  const existingT = cross2(acx, acy, abx, aby) / denom;
  if (!isSegmentParameter(newT) || !isSegmentParameter(existingT)) return null;
  const cleanT = clampUnit(newT);
  return {
    newT: cleanT,
    existingT: clampUnit(existingT),
    point: normalizePoint({
      x: a.x + abx * cleanT,
      y: a.y + aby * cleanT,
      z: a.z,
    }),
  };
}

function pointKey(point: Point3) {
  return [point.x, point.y, point.z].map((value) => value.toFixed(6)).join(':');
}

function memberPreviewTopology(scene: RenderScene, start: Point3 | null, end: Point3 | null): MemberPreviewTopology {
  if (!start || !end || pointDistance(start, end) <= 1e-6) return {};
  const splitPoints: Array<{ t: number; point: Point3 }> = [
    { t: 0, point: start },
    { t: 1, point: end },
  ];
  const previewNodes = new Map<string, Point3>();
  const previewSplitMemberSegments: NonNullable<ViewportEditOverlay['previewSplitMemberSegments']> = [];
  const memberSplitDimensions: NonNullable<ViewportEditOverlay['memberSplitDimensions']> = [];
  previewNodes.set(pointKey(end), end);

  scene.members.forEach((member) => {
    const endpoints = memberEndpoints(scene, member);
    if (!endpoints) return;
    const intersection = segmentIntersectionXY(start, end, endpoints.start, endpoints.end);
    if (!intersection) return;
    if (isSegmentInterior(intersection.newT)) {
      splitPoints.push({ t: intersection.newT, point: intersection.point });
      previewNodes.set(pointKey(intersection.point), intersection.point);
    }
    if (isSegmentInterior(intersection.existingT)) {
      previewNodes.set(pointKey(intersection.point), intersection.point);
      previewSplitMemberSegments.push(
        { memberId: member.id, start: endpoints.start, end: intersection.point },
        { memberId: member.id, start: intersection.point, end: endpoints.end },
      );
      memberSplitDimensions.push(
        { memberId: member.id, start: endpoints.start, end: intersection.point, distance: pointDistance(endpoints.start, intersection.point) },
        { memberId: member.id, start: intersection.point, end: endpoints.end, distance: pointDistance(intersection.point, endpoints.end) },
      );
    }
  });

  const sorted = splitPoints
    .sort((a, b) => a.t - b.t)
    .filter((item, index, items) => index === 0 || Math.abs(item.t - items[index - 1].t) > MEMBER_INTERSECTION_EPS);
  const previewMemberSegments = sorted.slice(0, -1).map((point, index) => ({
    start: point.point,
    end: sorted[index + 1].point,
  }));
  return {
    previewMemberSegments,
    previewNodes: [...previewNodes.values()],
    previewSplitMemberSegments,
    memberSplitDimensions,
  };
}

function WorkspaceBody({ children }: { children: ReactNode }) {
  return <div className="flex min-h-0 min-w-0 flex-1 items-stretch gap-0">{children}</div>;
}

function defaultGroupsPanelWidth(renderWidth?: number) {
  if (typeof window === 'undefined') return 400;
  const availableWidth = renderWidth ?? Math.max(0, window.innerWidth);
  return Math.min(GROUPS_PANEL_MAX, Math.max(GROUPS_PANEL_MIN, Math.round(availableWidth * DEFAULT_CHAT_WIDTH_RATIO)));
}

function groupsPanelSizingBase(renderAreaWidth?: number) {
  const availableWidth = renderAreaWidth ?? (typeof window === 'undefined' ? 1280 : window.innerWidth);
  return Math.max(0, availableWidth);
}

function groupsPanelWidthForRatio(ratio: number, renderAreaWidth?: number) {
  const width = Math.round(groupsPanelSizingBase(renderAreaWidth) * ratio);
  return Math.min(groupsPanelMaxWidth(renderAreaWidth), Math.max(GROUPS_PANEL_MIN, width));
}

function groupsPanelRatioForWidth(width: number, renderAreaWidth?: number) {
  const sizingBase = groupsPanelSizingBase(renderAreaWidth);
  if (sizingBase <= 0) return DEFAULT_CHAT_WIDTH_RATIO;
  return width / sizingBase;
}

function groupsPanelMaxWidth(renderAreaWidth?: number) {
  const ratioMax = Math.round(groupsPanelSizingBase(renderAreaWidth) * GROUPS_PANEL_MAX_RATIO);
  return Math.max(GROUPS_PANEL_MIN, Math.min(GROUPS_PANEL_MAX, ratioMax));
}

function measuredWorkspaceWidthFallback() {
  if (typeof window === 'undefined') return 1280;
  return window.innerWidth;
}

function workspacePanelSizingBase(workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  return workspaceAreaWidth > 0 ? workspaceAreaWidth : measuredWorkspaceWidthFallback();
}

function workspacePanelWidthForRatio(ratio: number, workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  return Math.round(workspacePanelSizingBase(workspaceAreaWidth) * ratio);
}

function clampWorkspacePanelRatio(ratio: number) {
  return Math.min(WORKSPACE_PANEL_MAX_RATIO, Math.max(WORKSPACE_PANEL_MIN_RATIO, ratio));
}

function workspacePanelRatioForWidth(width: number, workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  const sizingBase = workspacePanelSizingBase(workspaceAreaWidth);
  if (sizingBase <= 0) return WORKSPACE_PANEL_DEFAULT_RATIO;
  return clampWorkspacePanelRatio(width / sizingBase);
}

function workspacePanelBounds(workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  const sizingBase = workspacePanelSizingBase(workspaceAreaWidth);
  const ratioMin = workspacePanelWidthForRatio(WORKSPACE_PANEL_MIN_RATIO, sizingBase);
  const ratioMax = workspacePanelWidthForRatio(WORKSPACE_PANEL_MAX_RATIO, sizingBase);
  const min = Math.min(WORKSPACE_PANEL_MIN, Math.max(0, sizingBase - 320));
  const max = Math.min(WORKSPACE_PANEL_MAX, Math.max(min, ratioMax));
  return {
    min: Math.max(min, Math.min(WORKSPACE_PANEL_MIN, ratioMin)),
    max,
  };
}

function defaultWorkspacePanelWidth(workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  const bounds = workspacePanelBounds(workspaceAreaWidth);
  const ratioWidth = workspacePanelWidthForRatio(WORKSPACE_PANEL_DEFAULT_RATIO, workspaceAreaWidth);
  return Math.min(bounds.max, Math.max(bounds.min, ratioWidth || WORKSPACE_PANEL_FALLBACK_WIDTH));
}

function clampWorkspacePanelWidth(width: number, workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  const bounds = workspacePanelBounds(workspaceAreaWidth);
  return Math.min(bounds.max, Math.max(bounds.min, width));
}

function workspacePanelWidthForClampedRatio(ratio: number, workspaceAreaWidth = measuredWorkspaceWidthFallback()) {
  return clampWorkspacePanelWidth(workspacePanelWidthForRatio(clampWorkspacePanelRatio(ratio), workspaceAreaWidth), workspaceAreaWidth);
}

function DockedSidePanel({
  width,
  side,
  resizeLabel,
  showDivider = true,
  showResizeHandle = true,
  onResizeStart,
  onResizeValue,
  resizeMin,
  resizeMax,
  children,
}: WorkspaceSidebarProps & { side: 'left' | 'right'; resizeLabel: string; showDivider?: boolean; showResizeHandle?: boolean }) {
  return (
    <aside
      className={cn('relative h-full min-h-0 shrink-0', showDivider && (side === 'left' ? 'border-r' : 'border-l'))}
      style={{ width, minWidth: width, maxWidth: width }}
      data-render-sidebar
    >
      <div className="flex h-full min-h-0 flex-col gap-0">
        <div className="min-h-0 flex-1">{children}</div>
      </div>
      {showResizeHandle ? (
        <ResizeHandle
          label={resizeLabel}
          min={resizeMin}
          max={resizeMax}
          value={width}
          onPointerDown={onResizeStart}
          onValueChange={onResizeValue}
          handleStyle={{
            width: CHROME.splitHitZoneWidth,
            right: side === 'left' ? -CHROME.splitHitZoneWidth : undefined,
            left: side === 'right' ? -CHROME.splitHitZoneWidth : undefined,
          }}
        />
      ) : null}
    </aside>
  );
}

function RenderRailButton({
  active,
  disabled,
  label,
  tooltipDisabled,
  onClick,
  children,
}: {
  active: boolean;
  disabled?: boolean;
  label: string;
  tooltipDisabled?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={(
          <Button
            aria-label={label}
            aria-pressed={active}
            disabled={disabled}
            onClick={onClick}
            variant={active ? 'secondary' : 'ghost'}
            size="icon"
          >
            {children}
          </Button>
        )}
      />
      {!tooltipDisabled ? <TooltipContent side="bottom">{label}</TooltipContent> : null}
    </Tooltip>
  );
}

function SettingsCheckboxRow({
  id,
  label,
  checked,
  disabled,
  icon,
  onCheckedChange,
}: {
  id: string;
  label: string;
  checked: boolean;
  disabled?: boolean;
  icon?: ReactNode;
  onCheckedChange: (checked: boolean) => void;
}) {
  return (
    <Field
      orientation="horizontal"
      data-disabled={disabled || undefined}
      className="h-8 gap-2 px-2"
    >
      <Checkbox
        id={id}
        checked={checked}
        disabled={disabled}
        onCheckedChange={(value) => onCheckedChange(value === true)}
      />
      <FieldLabel htmlFor={id} className="min-w-0 cursor-default items-center">
        {icon}
        <span className="truncate">{label}</span>
      </FieldLabel>
    </Field>
  );
}

function SnapSettingInput({
  label,
  value,
  disabled,
  min,
  step,
  unit,
  onValue,
}: {
  label: string;
  value: number;
  disabled?: boolean;
  min: number;
  step: number;
  unit: string;
  onValue: (value: number) => void;
}) {
  return (
    <div className="flex min-w-0 items-center gap-1.5">
      <Input
        aria-label={label}
        disabled={disabled}
        type="number"
        min={min}
        step={step}
        value={value}
        onFocus={(event) => event.currentTarget.select()}
        onChange={(event) => onValue(Number(event.currentTarget.value))}
        className="h-7 w-16"
      />
      <span className="w-7 text-xs text-muted-foreground">{unit}</span>
    </div>
  );
}

function ToolbarToggle({
  label,
  pressed,
  disabled,
  onPressedChange,
  children,
}: {
  label: string;
  pressed: boolean;
  disabled?: boolean;
  onPressedChange: (pressed: boolean) => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={(
          <Toggle
            aria-label={label}
            pressed={pressed}
            disabled={disabled}
            onPressedChange={onPressedChange}
            variant="outline"
            size="default"
          >
            {children}
          </Toggle>
        )}
      />
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}

type ToolbarSettingsSection = 'member' | 'snap' | 'label';

function ToolbarSettingsMenu({
  section,
  open,
  selected,
  snapOptions,
  drawingOptions,
  visibility,
  disabled,
  onSnapOptions,
  onDrawingOptions,
  onVisibility,
  onOpenChange,
}: {
  section: ToolbarSettingsSection;
  open: boolean;
  selected: boolean;
  snapOptions: SnapOptions;
  drawingOptions: MemberDrawingOptions;
  visibility: ViewportLabelVisibility;
  disabled?: boolean;
  onSnapOptions: (options: SnapOptions) => void;
  onDrawingOptions: (options: MemberDrawingOptions) => void;
  onVisibility: (visibility: ViewportLabelVisibility) => void;
  onOpenChange: (open: boolean) => void;
}) {
  const label = section === 'member' ? 'Member' : section === 'snap' ? 'Snap' : 'Label';
  const setSnap = (patch: Partial<SnapOptions>) => onSnapOptions({ ...snapOptions, ...patch });
  const setVisibility = (key: keyof ViewportLabelVisibility, checked: boolean) => {
    onVisibility({ ...visibility, [key]: checked });
  };

  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <Tooltip>
        <TooltipTrigger
          render={(
            <PopoverTrigger
              render={(
                <SplitButtonSegment
                  aria-label={`${label} settings`}
                  aria-expanded={open}
                  disabled={disabled}
                  selected={selected}
                  variant="outline"
                  size="icon"
                >
                  <ChevronDown />
                </SplitButtonSegment>
              )}
            />
          )}
        />
        <TooltipContent side="bottom">{label} settings</TooltipContent>
      </Tooltip>
      <PopoverContent align="start" className="max-h-[min(36rem,calc(100vh-8rem))] w-72 overflow-y-auto p-3">
        <PopoverHeader>
          <PopoverTitle>{label} settings</PopoverTitle>
          <PopoverDescription>
            {section === 'member'
              ? 'Configure continuous member drawing.'
              : section === 'snap'
                ? 'Choose model snapping targets and increments.'
                : 'Choose model annotations shown in the viewport.'}
          </PopoverDescription>
        </PopoverHeader>
        <FieldGroup className="gap-3">
          {section === 'member' ? (
            <FieldSet className="gap-1">
              <FieldLegend variant="label" className="mb-0 px-2 py-1">Member drawing</FieldLegend>
              <FieldGroup className="gap-1">
              <SettingsCheckboxRow
                id="member-polygon-mode"
                label="Continuous drawing"
                checked={drawingOptions.polygonMode}
                disabled={disabled}
                onCheckedChange={(checked) => onDrawingOptions({ ...drawingOptions, polygonMode: checked })}
              />
              </FieldGroup>
            </FieldSet>
          ) : null}
          {section === 'snap' ? (
            <FieldSet className="gap-1">
              <FieldLegend variant="label" className="mb-0 px-2 py-1">Snapping</FieldLegend>
              <FieldGroup className="gap-1">
              <SettingsCheckboxRow id="snap-endpoints" label="Endpoints" checked={snapOptions.endpoints} disabled={disabled} onCheckedChange={(checked) => setSnap({ endpoints: checked })} />
              <SettingsCheckboxRow id="snap-midpoints" label="Midpoints" checked={snapOptions.midpoints} disabled={disabled} onCheckedChange={(checked) => setSnap({ midpoints: checked })} />
              <SettingsCheckboxRow id="snap-nearest" label="Nearest on member" checked={snapOptions.nearest} disabled={disabled} onCheckedChange={(checked) => setSnap({ nearest: checked })} />
              <Field orientation="horizontal" data-disabled={disabled || undefined} className="h-8 gap-2 px-2">
                <Checkbox id="snap-angles" checked={snapOptions.angles} disabled={disabled} onCheckedChange={(checked) => setSnap({ angles: checked === true })} />
                <FieldLabel htmlFor="snap-angles" className="min-w-0 truncate">Angle snap</FieldLabel>
                <SnapSettingInput label="Angle increment" disabled={disabled} min={1} step={1} unit="deg" value={snapOptions.angleIncrement} onValue={(value) => setSnap({ angleIncrement: Math.max(1, Number.isFinite(value) ? value : 15) })} />
              </Field>
              <SettingsCheckboxRow id="snap-axes" label="Axis snap" checked={snapOptions.axes} disabled={disabled || !snapOptions.angles} onCheckedChange={(checked) => setSnap({ axes: checked })} />
              <Field orientation="horizontal" data-disabled={disabled || undefined} className="h-8 gap-2 px-2">
                <Checkbox id="snap-dimension" checked={snapOptions.grid} disabled={disabled} onCheckedChange={(checked) => setSnap({ grid: checked === true })} />
                <FieldLabel htmlFor="snap-dimension" className="min-w-0 truncate">Dimension snap</FieldLabel>
                <SnapSettingInput label="Dimension increment" disabled={disabled} min={1} step={10} unit="mm" value={Math.round(snapOptions.gridSize * 1000)} onValue={(value) => setSnap({ gridSize: Math.max(0.001, Number.isFinite(value) ? value / 1000 : 0.1) })} />
              </Field>
              </FieldGroup>
            </FieldSet>
          ) : null}
          {section === 'label' ? (
            <FieldSet className="gap-1">
              <FieldLegend variant="label" className="mb-0 px-2 py-1">Labels</FieldLegend>
              <FieldGroup className="gap-1">
                <SettingsCheckboxRow id="visibility-node-labels" label="Node labels" icon={<Circle />} checked={visibility.node} onCheckedChange={(checked) => setVisibility('node', checked)} />
                <SettingsCheckboxRow id="visibility-member-labels" label="Member labels" icon={<PencilLine />} checked={visibility.member} onCheckedChange={(checked) => setVisibility('member', checked)} />
                <SettingsCheckboxRow id="visibility-support-labels" label="Support labels" icon={<Triangle />} checked={visibility.support} onCheckedChange={(checked) => setVisibility('support', checked)} />
                <SettingsCheckboxRow id="visibility-load-labels" label="Load labels" icon={<ArrowDown />} checked={visibility.load} onCheckedChange={(checked) => setVisibility('load', checked)} />
              </FieldGroup>
            </FieldSet>
          ) : null}
        </FieldGroup>
      </PopoverContent>
    </Popover>
  );
}

export function ContextualWorkspaceToolbar({
  viewMode,
  activePanel,
  activeTool,
  pendingMemberStart,
  editPending,
  snapOptions,
  memberDrawingOptions,
  labelVisibility,
  groupsAvailable,
  openToolbarMenu,
  onTool,
  onSnapOptions,
  onToggleSnap,
  onMemberDrawingOptions,
  onLabelVisibility,
  onToggleLabelVisibility,
  onToolbarMenuOpen,
  onTogglePanel,
}: {
  viewMode: ViewportViewMode;
  activePanel: RenderPanel;
  activeTool: BaseEditTool;
  pendingMemberStart: string | null;
  editPending: boolean;
  snapOptions: SnapOptions;
  memberDrawingOptions: MemberDrawingOptions;
  labelVisibility: ViewportLabelVisibility;
  groupsAvailable: boolean;
  openToolbarMenu: ToolbarMenuId | null;
  onTool: (tool: BaseEditTool) => void;
  onSnapOptions: (options: SnapOptions) => void;
  onToggleSnap: () => void;
  onMemberDrawingOptions: (options: MemberDrawingOptions) => void;
  onLabelVisibility: (visibility: ViewportLabelVisibility) => void;
  onToggleLabelVisibility: () => void;
  onToolbarMenuOpen: (menu: ToolbarMenuId | null) => void;
  onTogglePanel: (panel: 'groups') => void;
}) {
  const baseTools: Array<{ id: BaseEditTool; label: string; icon: ReactNode }> = [
    { id: 'select', label: 'Select', icon: <MousePointer2 data-icon="inline-start" /> },
    { id: 'node', label: 'Joint', icon: <Circle data-icon="inline-start" /> },
    { id: 'member', label: 'Member', icon: <PencilLine data-icon="inline-start" /> },
    { id: 'move', label: 'Move', icon: <Move data-icon="inline-start" /> },
    { id: 'split', label: 'Split', icon: <Scissors data-icon="inline-start" /> },
  ];
  const memberToolLabel = pendingMemberStart ? `Member from ${pendingMemberStart}` : 'Member';
  const snapsActive = snapEnablementActive(snapEnablement(snapOptions));
  const labelsActive = labelVisibilityActive(labelVisibility);

  return (
    <div
      aria-label={viewMode === 'base' ? 'Base model edit tools' : 'Render tools'}
      className="flex min-w-0 flex-nowrap items-center gap-1"
    >
      {viewMode === 'base' ? (
        <>
          <ToggleGroup
            aria-label="Editing mode"
            value={[activeTool]}
            onValueChange={(value) => {
              const nextTool = value[0] as BaseEditTool | undefined;
              if (nextTool && nextTool !== activeTool) onTool(nextTool);
            }}
            disabled={editPending}
            variant="outline"
            size="default"
            spacing={2}
          >
            {baseTools.map((tool) => {
              const tooltipLabel = tool.id === 'member' ? memberToolLabel : tool.label;
              const toolToggle = (
                <Tooltip key={tool.id}>
                  <TooltipTrigger
                    render={(
                      <ToggleGroupItem aria-label={tool.label} value={tool.id}>
                        {tool.icon}
                      </ToggleGroupItem>
                    )}
                  />
                  <TooltipContent side="bottom">{tooltipLabel}</TooltipContent>
                </Tooltip>
              );
              if (tool.id !== 'member') return toolToggle;

              return (
                <ButtonGroup key={tool.id} aria-label="Member controls">
                  {toolToggle}
                  <ToolbarSettingsMenu
                    section="member"
                    open={openToolbarMenu === 'member-settings'}
                    selected={activeTool === 'member'}
                    snapOptions={snapOptions}
                    drawingOptions={memberDrawingOptions}
                    visibility={labelVisibility}
                    disabled={editPending}
                    onSnapOptions={onSnapOptions}
                    onDrawingOptions={onMemberDrawingOptions}
                    onVisibility={onLabelVisibility}
                    onOpenChange={(open) => onToolbarMenuOpen(open ? 'member-settings' : null)}
                  />
                </ButtonGroup>
              );
            })}
          </ToggleGroup>
          <Separator orientation="vertical" className="mx-1 h-6" />
          <ButtonGroup aria-label="Snap controls">
            <ToolbarToggle label="Snaps" pressed={snapsActive} disabled={editPending} onPressedChange={(pressed) => {
              if (pressed !== snapsActive) onToggleSnap();
            }}>
              <Magnet data-icon="inline-start" />
            </ToolbarToggle>
            <ToolbarSettingsMenu
              section="snap"
              open={openToolbarMenu === 'snap-settings'}
              selected={snapsActive}
              snapOptions={snapOptions}
              drawingOptions={memberDrawingOptions}
              visibility={labelVisibility}
              disabled={editPending}
              onSnapOptions={onSnapOptions}
              onDrawingOptions={onMemberDrawingOptions}
              onVisibility={onLabelVisibility}
              onOpenChange={(open) => onToolbarMenuOpen(open ? 'snap-settings' : null)}
            />
          </ButtonGroup>
          <ButtonGroup aria-label="Label controls">
            <ToolbarToggle label="Labels" pressed={labelsActive} onPressedChange={(pressed) => {
              if (pressed !== labelsActive) onToggleLabelVisibility();
            }}>
              <Eye data-icon="inline-start" />
            </ToolbarToggle>
            <ToolbarSettingsMenu
              section="label"
              open={openToolbarMenu === 'label-settings'}
              selected={labelsActive}
              snapOptions={snapOptions}
              drawingOptions={memberDrawingOptions}
              visibility={labelVisibility}
              onSnapOptions={onSnapOptions}
              onDrawingOptions={onMemberDrawingOptions}
              onVisibility={onLabelVisibility}
              onOpenChange={(open) => onToolbarMenuOpen(open ? 'label-settings' : null)}
            />
          </ButtonGroup>
        </>
      ) : (
        <>
          {groupsAvailable ? (
            <RenderRailButton active={activePanel === 'groups'} label="Analysis Groups" onClick={() => onTogglePanel('groups')}>
              <Layers />
            </RenderRailButton>
          ) : null}
          <ButtonGroup aria-label="Label controls">
            <ToolbarToggle label="Labels" pressed={labelsActive} onPressedChange={(pressed) => {
              if (pressed !== labelsActive) onToggleLabelVisibility();
            }}>
              <Eye data-icon="inline-start" />
            </ToolbarToggle>
            <ToolbarSettingsMenu
              section="label"
              open={openToolbarMenu === 'label-settings'}
              selected={labelsActive}
              snapOptions={snapOptions}
              drawingOptions={memberDrawingOptions}
              visibility={labelVisibility}
              onSnapOptions={onSnapOptions}
              onDrawingOptions={onMemberDrawingOptions}
              onVisibility={onLabelVisibility}
              onOpenChange={(open) => onToolbarMenuOpen(open ? 'label-settings' : null)}
            />
          </ButtonGroup>
        </>
      )}
    </div>
  );
}

function memberStartId(member: RenderMember) {
  return member.start;
}

function memberEndId(member: RenderMember) {
  return member.end;
}

function ViewportRegion({
  scene,
  viewMode,
  groupsAvailable,
  leftInset,
  labelVisibility,
  focusedTargets,
  activeTool,
  snapOptions,
  memberDrawingOptions,
  pendingMemberStart,
  activePanel,
  cameraScopeKey,
  navigationProfileId,
  customNavigationSettings,
  mouseHandedness,
  menuDismissOverlayActive,
  onSelectTarget,
  onSelectionGesture,
  onNavigationProfileId,
  onCustomNavigationSettings,
  onMouseHandedness,
  onPendingMemberStart,
  onActivePanel,
  onTool,
  onEdit,
  onDismissToolbarMenu,
  editPending,
}: {
  scene: RenderScene;
  viewMode: ViewportViewMode;
  groupsAvailable: boolean;
  leftInset: number;
  labelVisibility: ViewportLabelVisibility;
  focusedTargets: AgentTarget[];
  activeTool: BaseEditTool;
  snapOptions: SnapOptions;
  memberDrawingOptions: MemberDrawingOptions;
  pendingMemberStart: PendingMemberStart | null;
  activePanel: RenderPanel;
  cameraScopeKey: string;
  navigationProfileId: ViewportNavigationProfileId;
  customNavigationSettings: ViewportCustomNavigationSettings;
  mouseHandedness: ViewportMouseHandedness;
  menuDismissOverlayActive: boolean;
  onSelectTarget: (target: AgentTarget | null) => void;
  onSelectionGesture: (gesture: ViewportSelectionGesture) => void;
  onNavigationProfileId: (profileId: ViewportNavigationProfileId) => void;
  onCustomNavigationSettings: (settings: ViewportCustomNavigationSettings) => void;
  onMouseHandedness: (handedness: ViewportMouseHandedness) => void;
  onPendingMemberStart: (start: PendingMemberStart | null) => void;
  onActivePanel: (panel: RenderPanel) => void;
  onTool: (tool: BaseEditTool) => void;
  onEdit: (operations: any[]) => Promise<void>;
  onDismissToolbarMenu: () => void;
  editPending: boolean;
}) {
  const [moveStart, setMoveStart] = useState<{ nodeId: string; point: Point3 } | null>(null);
  const [pointerInfo, setPointerInfo] = useState<ViewportPointerInfo | null>(null);
  const [snapLock, setSnapLock] = useState<SnapLock | null>(null);
  const [snapTemporarilyDisabled, setSnapTemporarilyDisabled] = useState(false);
  const [inferenceMode, setInferenceMode] = useState<SnapInferenceMode>('auto');
  const [memberDrawingPlane, setMemberDrawingPlane] = useState<WorldPlane | null>(null);
  const [renderAreaWidth, setRenderAreaWidth] = useState(measuredWorkspaceWidthFallback);
  const [panelWidth, setPanelWidth] = useState(() => defaultGroupsPanelWidth(groupsPanelSizingBase()));
  const [panelWidthMode, setPanelWidthMode] = useState<'default' | 'manual'>('default');
  const resizeCleanupRef = useRef<(() => void) | null>(null);
  const renderAreaRef = useRef<HTMLDivElement | null>(null);
  const panelWidthRatioRef = useRef(DEFAULT_CHAT_WIDTH_RATIO);
  const panelOpen = activePanel !== null;
  const activePreviewStart = pendingMemberStart?.point ?? moveStart?.point ?? null;
  const pointerPoint = pointerInfo?.point ?? (activePreviewStart && pointerInfo?.ray ? activePreviewStart : null);
  const currentSnapContext = {
    start: activePreviewStart,
    snapLock: snapTemporarilyDisabled ? null : snapLock,
    snapTarget: snapTemporarilyDisabled ? null : pointerInfo?.snapTarget ?? null,
    ray: pointerInfo?.ray ?? null,
    inferenceMode,
    inferencePlane: activeTool === 'member' ? memberDrawingPlane : null,
    disabled: snapTemporarilyDisabled,
  };
  const snappedPointer = pointerPoint
    ? activeTool === 'member' && activePreviewStart
      ? snapMemberPoint(scene, pointerPoint, snapOptions, currentSnapContext)
      : snapPoint(scene, pointerPoint, snapOptions, currentSnapContext)
    : null;
  const snapDisplay = snapDisplayFor(snappedPointer);
  const objectSnapActive = snapDisplay?.kind === 'end' || snapDisplay?.kind === 'mid' || snapDisplay?.kind === 'near';
  const showCursorSnapGlyph = activeTool !== 'member' || !activePreviewStart || objectSnapActive;
  const exactPreviewAxis = activeTool === 'member' && (snappedPointer?.axis === 'x' || snappedPointer?.axis === 'y' || snappedPointer?.axis === 'z')
    ? snappedPointer.axis
    : undefined;
  const toolStatus = activeTool === 'select'
    ? 'Select'
    : activeTool === 'node'
      ? 'Node'
      : activeTool === 'member'
        ? 'Member'
        : activeTool === 'move'
          ? 'Move'
          : 'Split';
  const viewportStatus = [
    toolStatus,
    ...(activeTool === 'member' ? [`Plane ${memberDrawingPlane ? memberDrawingPlane.toUpperCase() : 'Auto'}`] : []),
    ...(activePreviewStart && snappedPointer && snapLock?.kind === 'axis'
      ? [`Axis lock ${snapLock.axis.toUpperCase()}`]
      : exactPreviewAxis && snapOptions.axes && !objectSnapActive
        ? [`Axis snap ${exactPreviewAxis.toUpperCase()}`]
        : []),
  ].join(' · ');
  const viewportContextualShortcuts: ViewportHelpShortcut[] = activeTool === 'member' && activePreviewStart
    ? [
        { id: 'axis-lock', keys: ['1', '2', '3'], label: 'Toggle axis lock' },
        { id: 'snap-off', keys: ['Shift'], label: 'Temporarily disable snaps' },
      ]
    : [];
  const inferenceLabel = activeTool === 'member' && activePreviewStart && !objectSnapActive && !exactPreviewAxis
    ? memberInferenceLabel(activePreviewStart, snappedPointer)
    : undefined;
  const projectionGuide = activeTool === 'member' && activePreviewStart && snappedPointer && !exactPreviewAxis
    ? memberProjectionGuide(activePreviewStart, snappedPointer)
    : undefined;
  const previewTopology = useMemo(() => (
    activeTool === 'member' && activePreviewStart && snappedPointer
      ? memberPreviewTopology(scene, activePreviewStart, snappedPointer.point)
      : {}
  ), [activePreviewStart, activeTool, scene, snappedPointer]);
  const memberEndpointSnapTarget = activeTool === 'member' && activePreviewStart && objectSnapActive
    ? pointerInfo?.snapTarget ?? null
    : null;
  const generatedNodeId = useCallback((reservedIds: string[] = []) => nextId('node.N', [
    ...scene.nodes.map((node) => node.id),
    ...reservedIds,
  ]), [scene.nodes]);
  const generatedMemberId = useCallback(() => nextId('member.M', scene.members.map((member) => member.id)), [scene.members]);
  const memberEndpointIsFree = Boolean(activeTool === 'member' && activePreviewStart && snappedPointer && !objectSnapActive);
  const memberEndNodeId = activeTool === 'member' && activePreviewStart && snappedPointer
    ? snappedPointer.nodeId ?? (memberEndpointIsFree ? generatedNodeId(pendingMemberStart?.nodeId ? [pendingMemberStart.nodeId] : []) : null)
    : null;
  const memberEndLabel = memberEndNodeId
    ? { kind: 'node' as const, id: memberEndNodeId, point: snappedPointer!.point }
    : undefined;
  const memberSnapLabel = memberEndpointSnapTarget?.kind === 'member' || memberEndpointSnapTarget?.kind === 'memberMidpoint'
    ? { memberId: memberEndpointSnapTarget.id, point: snappedPointer!.point, showCoordinates: true }
    : undefined;
  const memberStartLabel = activeTool === 'member' && pendingMemberStart
    ? {
        kind: 'node' as const,
        id: pendingMemberStart.nodeId,
        point: pendingMemberStart.point,
      }
    : undefined;
  const editOverlay = useMemo<ViewportEditOverlay>(() => ({
    grid: { visible: false, size: snapOptions.gridSize },
    previewLine: activePreviewStart && snappedPointer ? { start: activePreviewStart, end: snappedPointer.point, tone: moveStart ? 'move' : activeTool === 'split' ? 'split' : 'member', axis: exactPreviewAxis } : undefined,
    previewMemberSegments: previewTopology.previewMemberSegments,
    previewNodes: previewTopology.previewNodes,
    previewSplitMemberSegments: previewTopology.previewSplitMemberSegments,
    memberSplitDimensions: previewTopology.memberSplitDimensions,
    snapPoint: activeTool !== 'select' && showCursorSnapGlyph && snappedPointer && snapDisplay ? { ...snappedPointer.point, ...snapDisplay, axis: snappedPointer.axis === 'x' || snappedPointer.axis === 'y' || snappedPointer.axis === 'z' ? snappedPointer.axis : undefined } : undefined,
    guideLines: activeTool !== 'member' && activePreviewStart && snappedPointer?.guideSegments ? snappedPointer.guideSegments : undefined,
    projectionGuide,
    memberStartLabel,
    memberEndLabel,
    memberSnapLabel,
    snapLabel: activeTool !== 'member' && activePreviewStart && snappedPointer?.snapLabel ? { point: snappedPointer.point, text: snappedPointer.snapLabel } : undefined,
    inferenceLabel,
  }), [activePreviewStart, activeTool, exactPreviewAxis, inferenceLabel, memberEndLabel, memberSnapLabel, memberStartLabel, moveStart, previewTopology.memberSplitDimensions, previewTopology.previewMemberSegments, previewTopology.previewNodes, previewTopology.previewSplitMemberSegments, projectionGuide, showCursorSnapGlyph, snapDisplay, snapOptions.gridSize, snappedPointer]);

  function sourceForSnap(result: SnapResult, snapTarget: ViewportPointerInfo['snapTarget'] | null): PendingMemberStart['source'] {
    if (result.nodeId) return { kind: 'node', id: result.nodeId };
    if (
      (result.label === 'Midpoint' || result.label === 'On Member') &&
      (snapTarget?.kind === 'member' || snapTarget?.kind === 'memberMidpoint')
    ) return { kind: 'member', id: snapTarget.id };
    return { kind: 'free' };
  }

  async function nodeForSnap(result: SnapResult, reservedNodeIds: string[] = []) {
    if (result.nodeId) return { nodeId: result.nodeId, point: result.point, operations: [] as any[] };
    const nodeId = generatedNodeId(reservedNodeIds);
    return {
      nodeId,
      point: result.point,
      operations: [{ kind: 'create_node', id: nodeId, x: result.point.x, y: result.point.y, z: result.point.z }],
    };
  }

  const stopResize = useCallback(() => {
    resizeCleanupRef.current?.();
    resizeCleanupRef.current = null;
  }, []);

  function startPanelResize(event: ReactPointerEvent<HTMLDivElement>) {
    event.preventDefault();
    panelWidthRatioRef.current = groupsPanelRatioForWidth(panelWidth, renderAreaWidth);
    setPanelWidthMode('manual');
    stopResize();
    const startX = event.clientX;
    const startWidth = panelWidth;
    const maxWidth = groupsPanelMaxWidth(renderAreaWidth);
    function onMove(moveEvent: PointerEvent) {
      const delta = moveEvent.clientX - startX;
      const next = Math.min(maxWidth, Math.max(GROUPS_PANEL_MIN, startWidth - delta));
      panelWidthRatioRef.current = groupsPanelRatioForWidth(next, renderAreaWidth);
      setPanelWidth(next);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', stopResize);
    window.addEventListener('pointercancel', stopResize);
    window.addEventListener('blur', stopResize);
    resizeCleanupRef.current = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', stopResize);
      window.removeEventListener('pointercancel', stopResize);
      window.removeEventListener('blur', stopResize);
    };
  }

  function setPanelWidthFromKeyboard(nextWidth: number) {
    stopResize();
    panelWidthRatioRef.current = groupsPanelRatioForWidth(nextWidth, renderAreaWidth);
    setPanelWidthMode('manual');
    setPanelWidth(nextWidth);
  }

  function togglePanel(panel: 'groups') {
    if (viewMode === 'base' && panel === 'groups') return;
    if (panel === 'groups' && !groupsAvailable) return;
    onActivePanel(activePanel === panel ? null : panel);
  }

  const handleSelectTarget = useCallback((target: AgentTarget | null) => {
    if (activeTool !== 'select') return;
    onSelectTarget(target);
  }, [activeTool, onSelectTarget]);

  const cancelToolSession = useCallback(() => {
    onPendingMemberStart(null);
    setMoveStart(null);
    setSnapLock(null);
    setSnapTemporarilyDisabled(false);
    setInferenceMode('auto');
    setMemberDrawingPlane(null);
  }, [onPendingMemberStart]);

  const commitMemberTo = useCallback(async (result: SnapResult, snapTarget: ViewportPointerInfo['snapTarget'] | null) => {
    if (!pendingMemberStart) return;
    if (pointDistance(pendingMemberStart.point, result.point) <= 1e-6) return;
    const end = await nodeForSnap(result, [pendingMemberStart.nodeId]);
    const memberId = generatedMemberId();
    await onEdit([
      ...end.operations,
      { kind: 'create_member', id: memberId, start_node: pendingMemberStart.nodeId, end_node: end.nodeId, role: 'member' },
    ]);
    if (memberDrawingOptions.polygonMode) {
      onPendingMemberStart({ nodeId: end.nodeId, point: end.point, source: sourceForSnap(result, snapTarget) });
    } else {
      onPendingMemberStart(null);
    }
    setSnapLock(null);
    setInferenceMode('auto');
  }, [generatedMemberId, memberDrawingOptions.polygonMode, nodeForSnap, onEdit, onPendingMemberStart, pendingMemberStart]);

  const handleViewportClick = useCallback(async (info: ViewportPointerInfo) => {
    if (viewMode !== 'base') return;
    if (editPending) return;
    const target = info.target;
    const point = info.point;
    if (activeTool === 'select') return;
    if (activeTool === 'node') {
      if (!point) return;
      const result = snapPoint(scene, point, snapOptions, { snapLock: snapTemporarilyDisabled ? null : snapLock, snapTarget: snapTemporarilyDisabled ? null : info.snapTarget, ray: info.ray, inferenceMode, disabled: snapTemporarilyDisabled });
      if (result.nodeId) return;
      await onEdit([{ kind: 'create_node', x: result.point.x, y: result.point.y, z: result.point.z }]);
      return;
    }
    if (activeTool === 'member') {
      if (!point && !(pendingMemberStart && info.ray)) return;
      const rawPoint = point ?? pendingMemberStart!.point;
      const context = {
        start: pendingMemberStart?.point ?? null,
        snapLock: snapTemporarilyDisabled ? null : snapLock,
        snapTarget: snapTemporarilyDisabled ? null : info.snapTarget,
        ray: info.ray,
        inferenceMode,
        inferencePlane: memberDrawingPlane,
        disabled: snapTemporarilyDisabled,
      };
      const result = pendingMemberStart
        ? snapMemberPoint(scene, rawPoint, snapOptions, context)
        : snapPoint(scene, rawPoint, snapOptions, context);
      if (!pendingMemberStart) {
        const start = await nodeForSnap(result);
        if (start.operations.length) await onEdit(start.operations);
        onPendingMemberStart({ nodeId: start.nodeId, point: start.point, source: sourceForSnap(result, info.snapTarget) });
        return;
      }
      await commitMemberTo(result, info.snapTarget);
      return;
    }
    if (activeTool === 'move') {
      if (!moveStart) {
        const supportNodeId = target?.kind === 'support'
          ? (() => {
              const support = scene.supports.find((item) => item.id === target.id);
              return support?.targetNode ?? support?.target_node;
            })()
          : null;
        const nodeId = target?.kind === 'node' ? target.id : supportNodeId;
        if (!nodeId) return;
        const node = scene.nodes.find((item) => item.id === nodeId);
        if (!node) return;
        setMoveStart({ nodeId: node.id, point: { x: node.x, y: node.y, z: node.z } });
        return;
      }
      if (!point && !info.ray) return;
      const rawPoint = point ?? moveStart.point;
      const result = snapPoint(scene, rawPoint, snapOptions, { start: moveStart.point, snapLock: snapTemporarilyDisabled ? null : snapLock, snapTarget: snapTemporarilyDisabled ? null : info.snapTarget, ray: info.ray, inferenceMode, disabled: snapTemporarilyDisabled });
      await onEdit([{ kind: 'update_node', id: moveStart.nodeId, x: result.point.x, y: result.point.y, z: result.point.z }]);
      setMoveStart(null);
      setSnapLock(null);
      setInferenceMode('auto');
      return;
    }
    if (activeTool === 'split') {
      if (target?.kind !== 'member' || !point) return;
      const result = snapPoint(scene, point, snapOptions, { snapLock: snapTemporarilyDisabled ? null : snapLock, snapTarget: snapTemporarilyDisabled ? null : info.snapTarget, ray: info.ray, inferenceMode, disabled: snapTemporarilyDisabled });
      await onEdit([{ kind: 'split_member', id: target.id, x: result.point.x, y: result.point.y, z: result.point.z }]);
      return;
    }
  }, [activeTool, commitMemberTo, editPending, inferenceMode, memberDrawingPlane, moveStart, nodeForSnap, onEdit, onPendingMemberStart, pendingMemberStart, scene, snapLock, snapOptions, snapTemporarilyDisabled, viewMode]);

  const handleViewportMove = useCallback((info: ViewportPointerInfo) => {
    setPointerInfo(info);
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const editingText = event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement || event.target instanceof HTMLSelectElement;
      const toggleKeyboardAxisLock = (axis: AxisId) => {
        setSnapLock((current) => (
          current?.kind === 'axis' && current.axis === axis && current.source === 'keyboard'
            ? null
            : { kind: 'axis', axis, source: 'keyboard' }
        ));
      };
      if (!editingText && event.key === 'Shift') {
        event.preventDefault();
        setSnapTemporarilyDisabled(true);
        setSnapLock((current) => current?.source === 'shift' ? null : current);
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        cancelToolSession();
        onSelectTarget(null);
        if (activeTool !== 'select') onTool('select');
        return;
      }
      if (!editingText && (event.key === 'Delete' || event.key === 'Backspace') && focusedTargets.length && !editPending) {
        const operations = deleteOperationsForTargets(scene, focusedTargets);
        if (operations.length) {
          event.preventDefault();
          void onEdit(operations);
        }
        return;
      }
      if (editingText || !activePreviewStart) return;
      if (event.key === '1') {
        event.preventDefault();
        toggleKeyboardAxisLock('x');
        return;
      }
      if (event.key === '2') {
        event.preventDefault();
        toggleKeyboardAxisLock('y');
        return;
      }
      if (event.key === '3') {
        event.preventDefault();
        toggleKeyboardAxisLock('z');
        return;
      }
    }
    function onKeyUp(event: KeyboardEvent) {
      if (event.key === 'Shift') {
        setSnapTemporarilyDisabled(false);
        setSnapLock((current) => current?.source === 'shift' ? null : current);
      }
    }
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [activePreviewStart, activeTool, cancelToolSession, editPending, focusedTargets, onEdit, onSelectTarget, onTool, scene]);

  useEffect(() => {
    setSnapLock(null);
    setSnapTemporarilyDisabled(false);
    setInferenceMode('auto');
    setMemberDrawingPlane(null);
  }, [activeTool, viewMode]);

  useEffect(() => stopResize, [stopResize]);

  useEffect(() => {
    if (activePanel === 'groups' && (viewMode === 'base' || !groupsAvailable)) onActivePanel(null);
  }, [activePanel, groupsAvailable, onActivePanel, viewMode]);

  useEffect(() => {
    function collapseForNarrowWindow() {
      if (window.innerWidth < AUTO_COLLAPSE_WIDTH) {
        onActivePanel(null);
      }
    }
    collapseForNarrowWindow();
    window.addEventListener('resize', collapseForNarrowWindow);
    return () => window.removeEventListener('resize', collapseForNarrowWindow);
  }, [onActivePanel]);

  useEffect(() => {
    const element = renderAreaRef.current;
    if (!element) return;
    const observedElement = element;
    function syncRenderAreaWidth() {
      const nextWidth = observedElement.clientWidth;
      setRenderAreaWidth(nextWidth);
      setPanelWidth((current) => {
        if (panelWidthMode === 'default') return defaultGroupsPanelWidth(groupsPanelSizingBase(nextWidth));
        const ratio = panelWidthRatioRef.current || groupsPanelRatioForWidth(current, nextWidth);
        return groupsPanelWidthForRatio(ratio, nextWidth);
      });
    }
    syncRenderAreaWidth();
    const observer = new ResizeObserver(syncRenderAreaWidth);
    observer.observe(observedElement);
    window.addEventListener('resize', syncRenderAreaWidth);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', syncRenderAreaWidth);
    };
  }, [panelWidthMode]);

  const availableCanvasWidth = Math.max(0, renderAreaWidth - leftInset - (panelOpen ? panelWidth : 0));

  return (
    <div ref={renderAreaRef} className="flex h-full min-w-70 flex-1 flex-col bg-background">
      <div className="relative min-h-0 flex-1">
        <Viewport3D
          scene={scene}
          focusedTargets={focusedTargets}
          labelVisibility={labelVisibility}
          editOverlay={editOverlay}
          selectionEnabled={activeTool === 'select'}
          cameraScopeKey={cameraScopeKey}
          navigationProfileId={navigationProfileId}
          customNavigationSettings={customNavigationSettings}
          fitInsets={{
            left: leftInset,
            right: panelOpen ? panelWidth : 0,
            bottom: 0,
          }}
          onSelectTarget={handleSelectTarget}
          onSelectionGesture={onSelectionGesture}
          onViewportClick={handleViewportClick}
          onViewportPointerMove={handleViewportMove}
        />
        {menuDismissOverlayActive ? (
          <div
            aria-hidden="true"
            className="absolute inset-y-0 left-0 z-30"
            style={{ right: panelOpen ? panelWidth : 0 }}
            onPointerDownCapture={(event) => {
              event.preventDefault();
              event.stopPropagation();
              onDismissToolbarMenu();
            }}
          />
        ) : null}
        {panelOpen ? (
          <div className="absolute inset-y-0 right-0">
            <DockedSidePanel
              side="right"
              width={panelWidth}
              onResizeStart={startPanelResize}
              onResizeValue={setPanelWidthFromKeyboard}
              resizeMin={GROUPS_PANEL_MIN}
              resizeMax={groupsPanelMaxWidth(renderAreaWidth)}
              resizeLabel="Resize render sidebar"
            >
              <SchemeGroupsPanelContent scene={scene} />
            </DockedSidePanel>
          </div>
        ) : null}
      </div>
      <ViewportHelpBar
        availableWidth={availableCanvasWidth}
        status={viewportStatus}
        navigationProfileId={navigationProfileId}
        customNavigationSettings={customNavigationSettings}
        mouseHandedness={mouseHandedness}
        contextualShortcuts={viewportContextualShortcuts}
        onNavigationProfileId={onNavigationProfileId}
        onCustomNavigationSettings={onCustomNavigationSettings}
        onMouseHandedness={onMouseHandedness}
      />
    </div>
  );
}

export function AppShell({
  state,
  onState,
  documentTabs,
  activeDocumentId,
  onDocumentSelect,
  onDocumentClose,
  onDocumentReorder,
  onOpenDocument,
  onNewBlankModel,
  documentActionPending,
  documentError,
}: {
  state: WorkbenchState | null;
  onState: (s: WorkbenchState) => void;
  documentTabs: DocumentTab[];
  activeDocumentId: string;
  onDocumentSelect: (id: string) => void;
  onDocumentClose: (id: string) => void;
  onDocumentReorder: (orderedIds: string[]) => void;
  onOpenDocument: () => void;
  onNewBlankModel: () => void;
  documentActionPending: boolean;
  documentError: string | null;
}) {
  const workspace = useMemo(() => buildSchemeWorkspace(state), [state]);
  const [active, setActive] = useState<ActiveView>({ kind: 'base' });
  const [workspacePanel, setWorkspacePanel] = useState<WorkspacePanel>(START_GEOMETRY_ONLY ? null : 'base-chat');
  const [workflowStage, setWorkflowStage] = useState<WorkflowStage>(() => initialWorkflowStage(state));
  const [optionAgentOpen, setOptionAgentOpen] = useState(false);
  const [narrowWorkflowInspector, setNarrowWorkflowInspector] = useState(() => window.innerWidth < AUTO_COLLAPSE_WIDTH);
  const [inspectorSheetOpen, setInspectorSheetOpen] = useState(false);
  const [selectedTargets, setSelectedTargets] = useState<AgentTarget[]>([]);
  const [editPending, setEditPending] = useState(false);
  const [legendOpen, setLegendOpen] = useState(false);
  const [labelVisibility, setLabelVisibility] = useState<ViewportLabelVisibility>(() => loadStoredLabelVisibility());
  const [navigationProfileId, setNavigationProfileId] = useState<ViewportNavigationProfileId>(() => loadStoredViewportNavigationProfile());
  const [customNavigationSettings, setCustomNavigationSettings] = useState<ViewportCustomNavigationSettings>(() => loadStoredViewportCustomNavigationSettings());
  const [mouseHandedness, setMouseHandedness] = useState<ViewportMouseHandedness>(() => loadStoredViewportMouseHandedness());
  const [activeTool, setActiveTool] = useState<BaseEditTool>('select');
  const [snapOptions, setSnapOptions] = useState<SnapOptions>(DEFAULT_SNAP_OPTIONS);
  const [memberDrawingOptions, setMemberDrawingOptions] = useState<MemberDrawingOptions>(DEFAULT_MEMBER_DRAWING_OPTIONS);
  const [pendingMemberStart, setPendingMemberStart] = useState<PendingMemberStart | null>(null);
  const [activeRenderPanel, setActiveRenderPanel] = useState<RenderPanel>(null);
  const [openToolbarMenu, setOpenToolbarMenu] = useState<ToolbarMenuId | null>(null);
  const [workspaceAreaWidth, setWorkspaceAreaWidth] = useState(measuredWorkspaceWidthFallback);
  const [workspacePanelWidth, setWorkspacePanelWidth] = useState(defaultWorkspacePanelWidth);
  const [workspacePanelWidthMode, setWorkspacePanelWidthMode] = useState<'default' | 'manual'>('default');
  const [generatingOptions, setGeneratingOptions] = useState(false);
  const [generationError, setGenerationError] = useState<string | null>(null);
  const [decisionBusy, setDecisionBusy] = useState(false);
  const [analysisBusy, setAnalysisBusy] = useState(false);
  const [workflowError, setWorkflowError] = useState<string | null>(null);
  const resizeCleanupRef = useRef<(() => void) | null>(null);
  const workspaceAreaRef = useRef<HTMLDivElement | null>(null);
  const workspacePanelRatioRef = useRef(WORKSPACE_PANEL_DEFAULT_RATIO);
  const workflowProjectRef = useRef<string | null>(state ? projectDirOf(state) : null);
  const workflowSurfaceInitializedRef = useRef(false);
  const evidenceReturnRef = useRef<{ active: ActiveView; panel: WorkspacePanel } | null>(null);
  const lastActiveSnapEnablementRef = useRef<SnapEnablement | null>(
    loadStoredSnapToggleMemory() ?? snapEnablement(DEFAULT_SNAP_OPTIONS)
  );
  const lastActiveLabelVisibilityRef = useRef<ViewportLabelVisibility | null>(
    loadStoredLabelVisibilityToggleMemory() ?? (labelVisibilityActive(labelVisibility) ? labelVisibility : null)
  );

  const activeBatch = activeBatchFrom(state);
  const journey = useMemo(() => workflowJourneyFrom(state, workflowStage), [state, workflowStage]);
  const activePath = activeDevelopmentPathFrom(state);
  const developmentPaths = developmentPathsFrom(state);
  const displayedDevelopmentPath = active.kind === 'development'
    ? developmentPaths.find((path) => path.id === active.pathId) ?? activePath
    : activePath;
  const activeOptionId = active.kind === 'scheme'
    ? active.id
    : active.kind === 'development'
      ? active.optionId
      : active.kind === 'evidence'
        ? active.optionId
        : null;
  const activeSchemeIndex = activeOptionId ? workspace.schemes.findIndex((scheme) => scheme.id === activeOptionId) : -1;
  const activeScheme = activeSchemeIndex >= 0 ? workspace.schemes[activeSchemeIndex] : null;
  const evidenceActive = active.kind === 'evidence' || active.kind === 'results';
  const developmentActive = active.kind === 'development';
  const activeScene = activeScheme ? activeScheme.scene : workspace.baseScene;
  const designOptionsEnabled = workspace.schemes.length > 0;
  const workspacePanelOpen = Boolean(workspacePanel && !evidenceActive);
  const activeSceneHasSchemeGroups = sceneHasSchemeGroups(activeScene);
  const viewportMode: ViewportViewMode = activeScheme || workspacePanel === 'design-options' || developmentActive ? 'scheme' : 'base';
  const cameraScopeKey = activeDocumentId;
  const showWorkspaceToolbar = !evidenceActive;

  useEffect(() => {
    const sync = () => setNarrowWorkflowInspector(window.innerWidth < AUTO_COLLAPSE_WIDTH);
    sync();
    window.addEventListener('resize', sync);
    return () => window.removeEventListener('resize', sync);
  }, []);

  useEffect(() => {
    setInspectorSheetOpen(false);
  }, [activeOptionId, evidenceActive, workflowStage]);

  useEffect(() => {
    if (!state) return;
    const projectKey = projectDirOf(state);
    if (workflowSurfaceInitializedRef.current && workflowProjectRef.current === projectKey) return;
    workflowSurfaceInitializedRef.current = true;
    workflowProjectRef.current = projectKey;
    const initialStage = initialWorkflowStage(state);
    setWorkflowStage(initialStage);
    if (initialStage === 'base') {
      setActive({ kind: 'base' });
      setWorkspacePanel('base-chat');
      setOptionAgentOpen(false);
      return;
    }
    const revisions = optionRevisions(activeBatch);
    const firstIncluded = revisions.find((revision) => revision.included);
    const optionId = firstIncluded?.optionId ?? firstIncluded?.option_id ?? workspace.schemes[0]?.id;
    if (initialStage === 'analysis' && activePath && journey.hasEligibleActivePath) {
      setActive({ kind: 'development', pathId: activePath.id, optionId: optionIdForPath(activePath) });
      setWorkspacePanel('development');
      return;
    }
    if (optionId) setActive({ kind: 'scheme', id: optionId });
    setWorkspacePanel('design-options');
    setOptionAgentOpen(false);
  }, [activeBatch, activePath, journey.hasEligibleActivePath, state, workspace.schemes]);

  useEffect(() => {
    if (!state) return;
    const resolvedStage = runtimeWorkflowStage(workflowStage, state);
    if (resolvedStage === workflowStage) return;
    setWorkflowStage(resolvedStage);
    evidenceReturnRef.current = null;
    if (resolvedStage === 'base') {
      setActive({ kind: 'base' });
      setWorkspacePanel('base-chat');
      setOptionAgentOpen(false);
      return;
    }
    const firstIncluded = optionRevisions(activeBatch).find((revision) => revision.included);
    const optionId = firstIncluded?.optionId ?? firstIncluded?.option_id ?? workspace.schemes[0]?.id;
    if (optionId) setActive({ kind: 'scheme', id: optionId });
    setWorkspacePanel('design-options');
    setOptionAgentOpen(false);
  }, [activeBatch, state, workflowStage, workspace.schemes]);

  const handleWorkspaceTool = useCallback((tool: BaseEditTool) => {
    setOpenToolbarMenu(null);
    setActiveTool(tool);
    setPendingMemberStart(null);
    if (tool !== 'select') setSelectedTargets([]);
  }, []);

  useEffect(() => {
    if (activeOptionId && !workspace.schemes.some((scheme) => scheme.id === activeOptionId)) {
      setActive({ kind: 'base' });
      setWorkspacePanel('base-chat');
      setWorkflowStage('base');
      setOptionAgentOpen(false);
    }
    if (!designOptionsEnabled && workspacePanel === 'design-options') {
      setActive({ kind: 'base' });
      setWorkspacePanel('base-chat');
      setWorkflowStage('base');
      setOptionAgentOpen(false);
    }
  }, [activeOptionId, designOptionsEnabled, workspace.schemes, workspacePanel]);

  useEffect(() => {
    setSelectedTargets([]);
    setOpenToolbarMenu(null);
  }, [active.kind, active.kind === 'scheme' ? active.id : 'base']);

  function selectScheme(id: string) {
    setActive({ kind: 'scheme', id });
    setWorkspacePanel('design-options');
    setOptionAgentOpen(workflowStage === 'options');
  }

  function focusWorkflowHeading() {
    window.requestAnimationFrame(() => {
      document.getElementById('fraia-workflow-stage-heading')?.focus();
    });
  }

  function openBaseSurface() {
    setActive({ kind: 'base' });
    setWorkspacePanel('base-chat');
    setOptionAgentOpen(false);
  }

  function openDesignOptionsSurface() {
    if (!designOptionsEnabled) return;
    const revisions = optionRevisions(activeBatch);
    const currentOptionId = activeOptionId && workspace.schemes.some((scheme) => scheme.id === activeOptionId)
      ? activeOptionId
      : null;
    const firstIncluded = revisions.find((revision) => revision.included);
    const optionId = currentOptionId ?? firstIncluded?.optionId ?? firstIncluded?.option_id ?? workspace.schemes[0]?.id;
    if (optionId) setActive({ kind: 'scheme', id: optionId });
    setActiveTool('select');
    setPendingMemberStart(null);
    setActiveRenderPanel(null);
    setWorkspacePanel('design-options');
    setOptionAgentOpen(false);
  }

  function openAnalysisSurface() {
    if (!journey.stages.find((stage) => stage.stage === 'analysis')?.available) return;
    setOptionAgentOpen(false);
    const activePathOptionId = activePath ? optionIdForPath(activePath) : '';
    const activePathIsIncluded = journey.includedOptionIds.includes(activePathOptionId);
    if (activePath && activePathIsIncluded && journey.hasEligibleActivePath) {
      setActive({ kind: 'development', pathId: activePath.id, optionId: activePathOptionId });
      setWorkspacePanel('development');
      return;
    }
    const optionId = activeOptionId && journey.includedOptionIds.includes(activeOptionId)
      ? activeOptionId
      : journey.includedOptionIds[0] ?? workspace.schemes[0]?.id;
    if (optionId) setActive({ kind: 'scheme', id: optionId });
    setWorkspacePanel('design-options');
  }

  function navigateToWorkflowStage(stage: WorkflowStage) {
    const target = journey.stages.find((candidate) => candidate.stage === stage);
    if (!target?.available) return;
    setWorkflowStage(stage);
    if (stage === 'base') openBaseSurface();
    if (stage === 'options') openDesignOptionsSurface();
    if (stage === 'analysis') openAnalysisSurface();
    focusWorkflowHeading();
  }

  function returnFromEvidence() {
    const previous = evidenceReturnRef.current;
    evidenceReturnRef.current = null;
    if (previous) {
      setActive(previous.active);
      setWorkspacePanel(previous.panel);
      focusWorkflowHeading();
      return;
    }
    if (workflowStage === 'analysis') openAnalysisSurface();
    else if (workflowStage === 'options') openDesignOptionsSurface();
    else openBaseSurface();
    focusWorkflowHeading();
  }

  function openEvidence(optionId = activeOptionId ?? workspace.schemes[0]?.id ?? '') {
    if (!optionId) return;
    evidenceReturnRef.current = { active, panel: workspacePanel };
    const revision = revisionForOption(state, optionId);
    const revisionRunId = revision?.latestAnalysisRunId ?? revision?.latest_analysis_run_id ?? null;
    const pathRunId = active.kind === 'development' && active.optionId === optionId
      ? displayedDevelopmentPath?.sourceAnalysisRunId ?? displayedDevelopmentPath?.source_analysis_run_id ?? null
      : null;
    setActive({ kind: 'evidence', optionId, runId: pathRunId ?? revisionRunId });
    setWorkspacePanel(null);
  }

  const handleViewportSelectTarget = useCallback((target: AgentTarget | null) => {
    setSelectedTargets((current) => (target ? toggleExpandedTarget(activeScene, current, target) : []));
  }, [activeScene]);

  const handleViewportSelectionGesture = useCallback((gesture: ViewportSelectionGesture) => {
    setSelectedTargets((current) => toggleExpandedTargets(activeScene, current, gesture.targets));
  }, [activeScene]);

  const handleNavigationProfileId = useCallback((profileId: ViewportNavigationProfileId) => {
    setNavigationProfileId(profileId);
    storeViewportNavigationProfile(profileId);
  }, []);

  const handleCustomNavigationSettings = useCallback((settings: ViewportCustomNavigationSettings) => {
    setCustomNavigationSettings(settings);
    storeViewportCustomNavigationSettings(settings);
  }, []);

  const handleMouseHandedness = useCallback((handedness: ViewportMouseHandedness) => {
    setMouseHandedness(handedness);
    storeViewportMouseHandedness(handedness);
  }, []);

  const handleLabelVisibility = useCallback((visibility: ViewportLabelVisibility) => {
    setLabelVisibility(visibility);
    storeLabelVisibility(visibility);
  }, []);

  const handleToggleSnap = useCallback(() => {
    setSnapOptions((current) => {
      const currentEnablement = snapEnablement(current);
      if (snapEnablementActive(currentEnablement)) {
        lastActiveSnapEnablementRef.current = currentEnablement;
        storeSnapToggleMemory(currentEnablement);
        return snapOptionsWithEnablement(current, ALL_DISABLED_SNAP_ENABLEMENT);
      }
      const restored = lastActiveSnapEnablementRef.current ?? snapEnablement(DEFAULT_SNAP_OPTIONS);
      return snapOptionsWithEnablement(current, restored);
    });
  }, []);

  const handleToggleLabelVisibility = useCallback(() => {
    setLabelVisibility((current) => {
      let next: ViewportLabelVisibility;
      if (labelVisibilityActive(current)) {
        lastActiveLabelVisibilityRef.current = current;
        storeLabelVisibilityToggleMemory(current);
        next = ALL_HIDDEN_LABEL_VISIBILITY;
      } else {
        next = lastActiveLabelVisibilityRef.current ?? ALL_VISIBLE_LABEL_VISIBILITY;
      }
      storeLabelVisibility(next);
      return next;
    });
  }, []);

  const editBaseModel = useCallback(async (operations: any[]) => {
    if (!state || !operations.length || active.kind !== 'base') return;
    setEditPending(true);
    try {
      const response = await window.fraia.editBaseModel({
        projectDir: projectDirOf(state),
        operations,
      });
      const nextState = normalizeWorkbenchState(response);
      if (nextState) {
        const nextScene = buildSchemeWorkspace(nextState).baseScene;
        setSelectedTargets((current) => reconciledSelectionAfterBaseEdit(activeScene, nextScene, current, operations));
        onState(nextState);
      }
    } catch (error) {
      console.error('Base model edit failed', error);
    } finally {
      setEditPending(false);
    }
  }, [active.kind, activeScene, onState, state]);

  async function generateDesignOptions() {
    if (!state || generatingOptions || !baseBriefReady(state)) return;
    if (workspace.schemes.length) {
      const confirmed = window.confirm('Generate a new design-option set from the current Base Model? The current set, chats, and analysis evidence will remain available in history.');
      if (!confirmed) return;
    }
    const projectDir = projectDirOf(state);
    setGeneratingOptions(true);
    setGenerationError(null);
    try {
      const response = await window.fraia.generateDesignOptions({
        projectDir,
        commentId: 'design-option-intents',
        proposedActions: pendingDesignOptionIntentActions(state),
      });
      const nextState = normalizeWorkbenchState(response);
      if (nextState) {
        onState(nextState);
        const nextSchemes = nextState.designSchemes ?? nextState.design_schemes ?? [];
        if (!nextSchemes.length) {
          const diagnostics = nextState.analysisReadiness?.diagnostics ?? nextState.analysis_readiness?.diagnostics ?? [];
          const diagnostic = diagnostics.find((item: any) => String(item?.code ?? '').startsWith('agent.design_options.')) ?? diagnostics[diagnostics.length - 1];
          const detail = diagnostic?.detail ? ` ${diagnostic.detail}` : '';
          throw new Error(diagnostic?.message ? `${diagnostic.message}${detail}` : response?.message ?? 'Design options were not generated.');
        }
        setActive({ kind: 'scheme', id: nextSchemes[0].id });
        setOptionAgentOpen(false);
      }
      setWorkflowStage('options');
      setWorkspacePanel('design-options');
      focusWorkflowHeading();
    } catch (error: any) {
      setGenerationError(error?.message || 'Could not generate design options.');
    } finally {
      setGeneratingOptions(false);
    }
  }

  async function setOptionIncluded(optionId: string, included: boolean) {
    if (!state || decisionBusy) return;
    setDecisionBusy(true);
    setWorkflowError(null);
    try {
      const response = await window.fraia.updateDesignOptionDecision({ projectDir: projectDirOf(state), action: 'set_included', optionId, included });
      const nextState = normalizeWorkbenchState(response);
      if (nextState) onState(nextState);
    } catch (error: any) {
      setWorkflowError(error?.message || 'Could not update the comparison set.');
    } finally {
      setDecisionBusy(false);
    }
  }

  async function analyseIncludedOptions() {
    if (!state || analysisBusy || activeBatch?.status !== 'active') return;
    const optionIds = journey.missingOrStaleOptionIds;
    if (!journey.includedOptionIds.length || journey.hasExactCurrentAnalysis) return;
    setAnalysisBusy(true);
    setWorkflowError(null);
    try {
      const response = optionIds.length
        ? await window.fraia.analyseDesignOptions({
          projectDir: projectDirOf(state),
          scope: { kind: 'selected_design_options', optionIds },
          candidatePolicy: 'all_candidates',
          checkProfile: 'preliminary_conservative_steel',
        })
        : await window.fraia.updateDesignOptionDecision({
          projectDir: projectDirOf(state),
          action: 'refresh_comparison',
        });
      const nextState = normalizeWorkbenchState(response);
      if (nextState) onState(nextState);
    } catch (error: any) {
      setWorkflowError(error?.message || 'Could not analyse the included design options.');
    } finally {
      setAnalysisBusy(false);
    }
  }

  async function developOption(optionId: string) {
    if (!state || decisionBusy) return;
    setDecisionBusy(true);
    setWorkflowError(null);
    try {
      const response = await window.fraia.updateDesignOptionDecision({ projectDir: projectDirOf(state), action: 'develop', optionId });
      const nextState = normalizeWorkbenchState(response);
      if (nextState) {
        onState(nextState);
        const nextPath = activeDevelopmentPathFrom(nextState);
        if (nextPath) {
          setWorkflowStage('analysis');
          setActive({ kind: 'development', pathId: nextPath.id, optionId: optionIdForPath(nextPath) });
          setWorkspacePanel('development');
        }
      }
    } catch (error: any) {
      setWorkflowError(error?.message || 'Could not open a work path for this option.');
    } finally {
      setDecisionBusy(false);
    }
  }

  const stopResize = useCallback(() => {
    resizeCleanupRef.current?.();
    resizeCleanupRef.current = null;
  }, []);

  function startResize(
    event: ReactPointerEvent<HTMLDivElement>,
    startWidth: number,
    setWidth: (width: number) => void,
    min: number,
    max: number,
    onStart?: () => void,
    onWidthChange?: (width: number) => void,
  ) {
    event.preventDefault();
    onStart?.();
    stopResize();
    const startX = event.clientX;
    function onMove(moveEvent: PointerEvent) {
      const delta = moveEvent.clientX - startX;
      const next = Math.min(max, Math.max(min, startWidth + delta));
      setWidth(next);
      onWidthChange?.(next);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', stopResize);
    window.addEventListener('pointercancel', stopResize);
    window.addEventListener('blur', stopResize);
    resizeCleanupRef.current = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', stopResize);
      window.removeEventListener('pointercancel', stopResize);
      window.removeEventListener('blur', stopResize);
    };
  }

  const startWorkspacePanelResize = (event: ReactPointerEvent<HTMLDivElement>) => {
    const bounds = workspacePanelBounds(workspaceAreaWidth);
    startResize(
      event,
      workspacePanelWidth,
      setWorkspacePanelWidth,
      bounds.min,
      bounds.max,
      () => {
        workspacePanelRatioRef.current = workspacePanelRatioForWidth(workspacePanelWidth, workspaceAreaWidth);
        setWorkspacePanelWidthMode('manual');
      },
      (nextWidth) => {
        workspacePanelRatioRef.current = workspacePanelRatioForWidth(nextWidth, workspaceAreaWidth);
      },
    );
  };

  const setWorkspacePanelWidthFromKeyboard = (nextWidth: number) => {
    stopResize();
    workspacePanelRatioRef.current = workspacePanelRatioForWidth(nextWidth, workspaceAreaWidth);
    setWorkspacePanelWidthMode('manual');
    setWorkspacePanelWidth(nextWidth);
  };

  function workspacePanelTitle() {
    if (workflowStage === 'analysis') return 'Analysis & Comparison';
    if (workflowStage === 'options') return optionAgentOpen && activeScheme ? activeScheme.name : 'Design Options';
    return 'Base Model';
  }

  function workspacePanelHeaderLeading() {
    if (workflowStage === 'options' && optionAgentOpen) {
      return (
        <Button
          aria-label="Back to design options"
          title="Back to design options"
          onClick={() => {
            setOptionAgentOpen(false);
            focusWorkflowHeading();
          }}
          variant="ghost"
          size="icon-sm"
        >
          <ArrowLeft />
        </Button>
      );
    }
    return null;
  }

  function workspacePanelContent() {
    if (workspacePanel === 'base-chat') {
      return (
        <BaseChatPanel
          state={state}
          onState={(nextState) => {
            setGenerationError(null);
            onState(nextState);
          }}
          onGenerateOptions={generateDesignOptions}
          generatingOptions={generatingOptions}
          hasDesignOptions={workspace.schemes.length > 0}
          generationError={generationError}
        />
      );
    }
    if (workspacePanel === 'development' && activeScheme && displayedDevelopmentPath) {
      return <DevelopmentPanel scheme={activeScheme} path={displayedDevelopmentPath} onOpenEvidence={() => openEvidence(activeScheme.id)} onBackToComparison={() => { setActive({ kind: 'scheme', id: activeScheme.id }); setWorkspacePanel('design-options'); }} />;
    }
    if (workflowStage === 'options' && optionAgentOpen && activeScheme) {
      return (
        <DesignOptionAgentPanel
          state={state}
          scheme={activeScheme}
          revision={revisionForOption(state, activeScheme.id)}
          busy={decisionBusy}
          onState={onState}
          onIncludedChange={(included) => setOptionIncluded(activeScheme.id, included)}
        />
      );
    }
    return (
      <DesignOptionsPanel
        active={active}
        schemes={workspace.schemes}
        batch={activeBatch}
        batches={decisionStateFrom(state).batches}
        stage={workflowStage === 'analysis' ? 'analysis' : 'options'}
        busy={decisionBusy}
        onSelectScheme={selectScheme}
        onIncludedChange={setOptionIncluded}
        onCompare={() => navigateToWorkflowStage('analysis')}
      />
    );
  }

  function optionInspector() {
    if (workflowStage !== 'analysis' || !activeScheme) return null;
    return (
      <DesignOptionInspector
        state={state}
        scheme={activeScheme}
        revision={revisionForOption(state, activeScheme.id)}
        stage="analysis"
        comparisonCurrent={journey.hasExactCurrentAnalysis}
        developmentPaths={developmentPaths}
        activePathId={activePath?.id ?? null}
        onState={onState}
        onDevelop={() => developOption(activeScheme.id)}
        onOpenPath={developOption}
        onOpenEvidence={() => openEvidence(activeScheme.id)}
      />
    );
  }

  useEffect(() => stopResize, [stopResize]);

  useEffect(() => {
    const element = workspaceAreaRef.current;
    if (!element) return;
    const observedElement = element;
    function syncWorkspaceAreaWidth() {
      const nextWidth = observedElement.clientWidth;
      setWorkspaceAreaWidth(nextWidth);
      setWorkspacePanelWidth((current) => {
        if (workspacePanelWidthMode === 'default') return defaultWorkspacePanelWidth(nextWidth);
        const ratio = workspacePanelRatioRef.current || workspacePanelRatioForWidth(current, nextWidth);
        return workspacePanelWidthForClampedRatio(ratio, nextWidth);
      });
    }
    syncWorkspaceAreaWidth();
    const observer = new ResizeObserver(syncWorkspaceAreaWidth);
    observer.observe(observedElement);
    window.addEventListener('resize', syncWorkspaceAreaWidth);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', syncWorkspaceAreaWidth);
    };
  }, [workspacePanelWidthMode]);

  useEffect(() => {
    if (!workspacePanelOpen) return;
    const bounds = workspacePanelBounds(workspaceAreaWidth);
    setWorkspacePanelWidth((current) => {
      const next = Math.min(bounds.max, Math.max(bounds.min, current || defaultWorkspacePanelWidth(workspaceAreaWidth)));
      if (next !== current) workspacePanelRatioRef.current = workspacePanelRatioForWidth(next, workspaceAreaWidth);
      return next;
    });
  }, [workspaceAreaWidth, workspacePanelOpen]);

  return (
    <div className="grid h-screen w-screen grid-rows-[auto_auto_minmax(0,1fr)] overflow-hidden bg-background text-foreground">
      <header style={{ height: APP_HEADER_HEIGHT }}>
        <AppMenuBar />
        <div className="shrink-0" style={{ height: CHROME.tabHeight }}>
          <DocumentTabBar
            tabs={documentTabs}
            value={activeDocumentId}
            panelId={DOCUMENT_PANEL_ID}
            onValueChange={onDocumentSelect}
            onClose={onDocumentClose}
            onReorder={onDocumentReorder}
            onOpen={onOpenDocument}
            openDisabled={documentActionPending}
            onNewBlankModel={onNewBlankModel}
            newBlankModelDisabled={documentActionPending}
          />
        </div>
      </header>
      <WorkflowStageBar
        currentStage={journey.currentStage}
        stages={journey.stages}
        onNavigate={navigateToWorkflowStage}
      />
      <main className="min-h-0 min-w-0">
        <div
          id={DOCUMENT_PANEL_ID}
          role="tabpanel"
          aria-labelledby={documentTabTriggerId(activeDocumentId)}
          ref={workspaceAreaRef}
          className="relative flex h-full min-h-0 min-w-0 flex-col gap-0"
        >
          {workspacePanelOpen ? (
            <ResizeHandle
              label="Resize workspace split"
              min={workspacePanelBounds(workspaceAreaWidth).min}
              max={workspacePanelBounds(workspaceAreaWidth).max}
              value={workspacePanelWidth}
              separatorStyle={{
                left: workspacePanelWidth,
              }}
              handleStyle={{
                  width: CHROME.splitHitZoneWidth,
                  left: workspacePanelWidth,
              }}
              onPointerDown={startWorkspacePanelResize}
              onValueChange={setWorkspacePanelWidthFromKeyboard}
            />
          ) : null}
          <ModelWorkspaceChrome
            title={evidenceActive ? 'Engineering evidence' : workspacePanelTitle()}
            leading={workspacePanelHeaderLeading()}
            panelWidth={workspacePanelWidth}
            panelOpen={workspacePanelOpen}
            trailing={evidenceActive ? (
              <Button aria-label="Back to current work" title="Back to current work" onClick={returnFromEvidence} variant="outline" size="sm">
                Back to current work
              </Button>
            ) : workflowStage === 'analysis' ? (
              <div className="flex items-center gap-2">
                {narrowWorkflowInspector && activeScheme ? (
                  <Button onClick={() => setInspectorSheetOpen(true)} variant="outline" size="sm">
                    <PanelRightOpen data-icon="inline-start" /> Option details
                  </Button>
                ) : null}
                {journey.hasExactCurrentAnalysis && (latestComparisonFrom(state)?.recommendedOptionId || latestComparisonFrom(state)?.recommended_option_id) ? <Badge variant="secondary"><Sparkles /> Recommendation ready</Badge> : null}
                <Button
                  onClick={analyseIncludedOptions}
                  disabled={analysisBusy || activeBatch?.status !== 'active' || !journey.includedOptionIds.length || journey.hasExactCurrentAnalysis}
                  size="sm"
                >
                  {analysisBusy ? <Spinner data-icon="inline-start" /> : <Play data-icon="inline-start" />}
                  {analysisBusy ? 'Analysing…' : journey.hasExactCurrentAnalysis ? 'Analysis current' : journey.missingOrStaleOptionIds.length ? 'Analyse options' : 'Refresh comparison'}
                </Button>
              </div>
            ) : null}
          >
            {showWorkspaceToolbar ? (
              <ContextualWorkspaceToolbar
                viewMode={viewportMode}
                activePanel={activeRenderPanel}
                activeTool={activeTool}
                pendingMemberStart={pendingMemberStart?.nodeId ?? null}
                editPending={editPending}
                snapOptions={snapOptions}
                memberDrawingOptions={memberDrawingOptions}
                labelVisibility={labelVisibility}
                groupsAvailable={Boolean(activeScheme) && activeSceneHasSchemeGroups}
                openToolbarMenu={openToolbarMenu}
                onTool={handleWorkspaceTool}
                onSnapOptions={setSnapOptions}
                onToggleSnap={handleToggleSnap}
                onMemberDrawingOptions={setMemberDrawingOptions}
                onLabelVisibility={handleLabelVisibility}
                onToggleLabelVisibility={handleToggleLabelVisibility}
                onToolbarMenuOpen={setOpenToolbarMenu}
                onTogglePanel={(panel) => setActiveRenderPanel((current) => (current === panel ? null : panel))}
              />
            ) : null}
          </ModelWorkspaceChrome>
          {workflowError ? (
            <div className="shrink-0 px-3 py-2">
              <Alert variant="destructive">
                <AlertDescription>{workflowError}</AlertDescription>
              </Alert>
            </div>
          ) : null}
          {documentError ? (
            <div className="shrink-0 px-3 py-2">
              <Alert variant="destructive">
                <AlertDescription>{documentError}</AlertDescription>
              </Alert>
            </div>
          ) : null}
          <WorkspaceBody>
            {workspacePanelOpen ? (
              <DockedSidePanel
                side="left"
                width={workspacePanelWidth}
                showDivider={false}
                showResizeHandle={false}
                onResizeStart={startWorkspacePanelResize}
                onResizeValue={setWorkspacePanelWidthFromKeyboard}
                resizeMin={workspacePanelBounds(workspaceAreaWidth).min}
                resizeMax={workspacePanelBounds(workspaceAreaWidth).max}
                resizeLabel="Resize workspace panel"
              >
                {workspacePanelContent()}
              </DockedSidePanel>
            ) : null}
            <div className="relative min-h-0 min-w-0 flex-1">
              {evidenceActive ? (
                <ResultsWorkspace state={state} requestedRunId={active.kind === 'evidence' ? active.runId : null} />
              ) : (
                <div className="flex h-full min-h-0 min-w-0">
                  <ViewportRegion
                    scene={activeScene}
                    viewMode={viewportMode}
                    leftInset={0}
                    labelVisibility={labelVisibility}
                    groupsAvailable={activeSceneHasSchemeGroups}
                    focusedTargets={selectedTargets}
                    activeTool={activeTool}
                    snapOptions={snapOptions}
                    memberDrawingOptions={memberDrawingOptions}
                    pendingMemberStart={pendingMemberStart}
                    activePanel={activeRenderPanel}
                    cameraScopeKey={cameraScopeKey}
                    navigationProfileId={navigationProfileId}
                    customNavigationSettings={customNavigationSettings}
                    mouseHandedness={mouseHandedness}
                    menuDismissOverlayActive={openToolbarMenu !== null}
                    onSelectTarget={handleViewportSelectTarget}
                    onSelectionGesture={handleViewportSelectionGesture}
                    onNavigationProfileId={handleNavigationProfileId}
                    onCustomNavigationSettings={handleCustomNavigationSettings}
                    onMouseHandedness={handleMouseHandedness}
                    onPendingMemberStart={setPendingMemberStart}
                    onActivePanel={setActiveRenderPanel}
                    onTool={handleWorkspaceTool}
                    onEdit={editBaseModel}
                    onDismissToolbarMenu={() => setOpenToolbarMenu(null)}
                    editPending={editPending}
                  />
                  {!narrowWorkflowInspector && workflowStage === 'analysis' && activeScheme ? (
                    <div className="h-full shrink-0" style={{ width: OPTION_INSPECTOR_WIDTH }}>
                      {optionInspector()}
                    </div>
                  ) : null}
                </div>
              )}
            </div>
          </WorkspaceBody>
          <Sheet open={inspectorSheetOpen && narrowWorkflowInspector && Boolean(activeScheme) && workflowStage === 'analysis'} onOpenChange={setInspectorSheetOpen}>
            <SheetContent side="right" className="w-[min(90vw,420px)] p-0" showCloseButton>
              <SheetHeader className="sr-only">
                <SheetTitle>Option details</SheetTitle>
                <SheetDescription>Inspect the current design option and its stage-specific information.</SheetDescription>
              </SheetHeader>
              <div className="min-h-0 flex-1">{optionInspector()}</div>
            </SheetContent>
          </Sheet>
        </div>
      </main>
      <LegendDialog open={legendOpen} onClose={() => setLegendOpen(false)} />
    </div>
  );
}
