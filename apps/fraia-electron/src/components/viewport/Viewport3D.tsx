import { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { loadStoredViewportCamera, saveStoredViewportCamera } from '@/lib/viewportCameraMemory';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { Line2 } from 'three/examples/jsm/lines/Line2.js';
import { LineGeometry } from 'three/examples/jsm/lines/LineGeometry.js';
import { LineMaterial } from 'three/examples/jsm/lines/LineMaterial.js';
import { LineSegments2 } from 'three/examples/jsm/lines/LineSegments2.js';
import { LineSegmentsGeometry } from 'three/examples/jsm/lines/LineSegmentsGeometry.js';
import { ViewportGizmo, type GizmoOptions } from 'three-viewport-gizmo';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { installOrthographicViewCubeCamera } from '@/lib/viewportGizmoCamera';
import type { AgentTarget, RenderLoad, RenderRelease, RenderScene, RenderSupport } from '../../lib/types';
import { displayMembersFor, type DisplayMember } from '../../lib/renderMembers';
import { formatQuantity, metricStructuralUnitProfile, unitProfileFrom } from '../../lib/units';
import { expandedLabelCenterAlongDirection, loadArrowSymbol, SUPPORT_SYMBOL_SCALE, supportLabelOffset, supportLabelOffsetCandidates, supportSymbolHitRegion, supportSymbolOffset, supportSymbolSpec, viewportLoadLeaderFractions, viewportStroke, viewportVisualProfile, type ViewportSymbolSpec } from '../../lib/viewportSymbols';
import {
  DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID,
  DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS,
  resolveViewportNavigationGesture,
  viewportZoomSpeedForGesture,
  type ViewportCustomNavigationSettings,
  type ViewportNavigationAction,
  type ViewportNavigationProfileId,
} from '../../lib/viewportNavigation';
import {
  emptyCanvasSelectionAction,
  isViewportNodeSelectable,
  prioritizeViewportPointerTarget,
  viewportSelectionKind,
} from '../../lib/viewportSelection';

type ViewportFitInsets = {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
};

type SnapGuideAxis = 'x' | 'y' | 'z' | 'angle';
type ViewportCameraView = 'front' | 'top' | 'right' | 'iso';
type InferenceAxis = 'x' | 'y' | 'z';
type ProjectionPlane = 'xy' | 'xz' | 'yz';
type InferenceLabelEntry = { axis: InferenceAxis; angleDeg: number };
type ProjectionGuideAngle = { axis: InferenceAxis; angleDeg: number };
type ProjectionGuide = {
  plane: ProjectionPlane;
  start: { x: number; y: number; z: number };
  projectedEnd: { x: number; y: number; z: number };
  realEnd: { x: number; y: number; z: number };
  angles: ProjectionGuideAngle[];
  outOfPlaneAxis?: InferenceAxis;
  angle?: { axis: InferenceAxis; angleDeg: number };
};
type ProjectionGuideOverlay = { guide: ProjectionGuide };

export type ViewportEditOverlay = {
  grid?: { visible: boolean; size: number; extent?: number };
  previewLine?: { start: { x: number; y: number; z: number }; end: { x: number; y: number; z: number }; tone?: 'member' | 'move' | 'split'; axis?: 'x' | 'y' | 'z' };
  previewMemberSegments?: Array<{ start: { x: number; y: number; z: number }; end: { x: number; y: number; z: number } }>;
  previewSplitMemberSegments?: Array<{ memberId: string; start: { x: number; y: number; z: number }; end: { x: number; y: number; z: number } }>;
  memberSplitDimensions?: Array<{ memberId: string; start: { x: number; y: number; z: number }; end: { x: number; y: number; z: number }; distance: number }>;
  previewNodes?: Array<{ x: number; y: number; z: number }>;
  snapPoint?: { x: number; y: number; z: number; kind: 'end' | 'mid' | 'near' | 'axis' | 'angle'; axis?: 'x' | 'y' | 'z' };
  guideLines?: Array<{ start: { x: number; y: number; z: number }; end: { x: number; y: number; z: number }; axis: SnapGuideAxis }>;
  projectionGuide?: ProjectionGuideOverlay;
  coordinateLabel?: { point: { x: number; y: number; z: number } };
  memberStartLabel?: (
    | { kind: 'free'; point: { x: number; y: number; z: number } }
    | { kind: 'node'; id: string; point: { x: number; y: number; z: number } }
    | { kind: 'member'; id: string; point: { x: number; y: number; z: number } }
  );
  memberEndLabel?: { kind: 'node'; id: string; point: { x: number; y: number; z: number } };
  memberSnapLabel?: { memberId: string; point: { x: number; y: number; z: number }; showCoordinates?: boolean };
  snapLabel?: { point: { x: number; y: number; z: number }; text: string };
  inferenceLabel?: (
    | { kind: 'axis'; anchor: { x: number; y: number; z: number }; axis: InferenceAxis; label: string }
    | { kind: 'angles'; anchor: { x: number; y: number; z: number }; locked?: boolean; entries: InferenceLabelEntry[] }
  );
};

export type ViewportLabelVisibility = {
  node: boolean;
  member: boolean;
  support: boolean;
  load: boolean;
};

export type ViewportPointerInfo = {
  target: AgentTarget | null;
  snapTarget: { kind: 'node' | 'memberMidpoint' | 'member'; id: string } | null;
  point: { x: number; y: number; z: number } | null;
  hoverPoint?: { x: number; y: number; z: number } | null;
  targetSource?: 'label' | 'geometry';
  ray: { origin: { x: number; y: number; z: number }; direction: { x: number; y: number; z: number } } | null;
  screen: { x: number; y: number };
  shiftKey: boolean;
};

export type ViewportSelectionGesture = {
  operation: 'toggle';
  selectionKind: 'window' | 'crossing';
  shape: 'box' | 'lasso';
  targets: AgentTarget[];
  start: { x: number; y: number };
  end: { x: number; y: number };
  points: Array<{ x: number; y: number }>;
};

const AUTO_LABEL_MEMBER_LIMIT = 10000;

function isDarkMode() {
  return document.documentElement.classList.contains('dark');
}

function viewportBackgroundColor() {
  return isDarkMode() ? '#141517' : '#f1f3f5';
}

function viewportMemberColor() {
  return isDarkMode() ? '#f8fafc' : '#111827';
}

function viewportMemberPreviewColor() {
  return isDarkMode() ? '#cbd5e1' : '#5f666f';
}

function viewportNodeColor() {
  return viewportMemberColor();
}

function viewportInteractionPalette() {
  const dark = isDarkMode();
  return {
    hoverAccent: dark ? '#fbbf24' : '#b45309',
    selectedAccent: dark ? '#fbbf24' : '#b45309',
    haloUnderlay: viewportBackgroundColor(),
    hoverOpacity: dark ? 0.48 : 0.42,
    selectedOpacity: dark ? 0.96 : 0.92,
    selectedHoverOpacity: 1,
  };
}

function sceneSupportColor() {
  return isDarkMode() ? '#69db7c' : '#40c057';
}

function sceneLoadColor() {
  return isDarkMode() ? '#ff8787' : '#fa5252';
}

function sceneProposedMemberColor() {
  return isDarkMode() ? '#ffd43b' : '#fab005';
}

function nodeMap(scene: RenderScene) {
  return new Map(scene.nodes.map((n) => [n.id, n]));
}

function isSchemePreviewNode(node: RenderScene['nodes'][number]) {
  return node.source === 'scheme';
}

function supportNodeId(support: RenderSupport) {
  return support.targetNode ?? support.target_node;
}
function isBriefVisualSupport(support: RenderSupport) {
  return support.id.startsWith('brief-visual-support-');
}
function supportType(support: RenderSupport) {
  if (isBriefVisualSupport(support)) return 'Indicative';
  const translations = [support.ux, support.uy, support.uz].filter(Boolean).length;
  const rotations = [support.rx, support.ry, support.rz].filter(Boolean).length;
  if ([support.ux, support.uy, support.uz, support.rx, support.ry, support.rz].every(Boolean)) return 'Fixed';
  if (translations >= 2 && rotations === 0) return 'Pinned';
  if (translations === 1 && rotations === 0) return 'Roller';
  return 'Support';
}
function supportGroupLabel(support: RenderSupport) {
  return support.supportGroupLabel ?? support.support_group_label;
}
function loadDirection(load: RenderLoad) {
  return new THREE.Vector3(load.directionX ?? load.direction_x ?? 0, load.directionY ?? load.direction_y ?? -1, load.directionZ ?? load.direction_z ?? 0).normalize();
}

function memberStartId(member: { start?: string; startNode?: string; start_node?: string; i?: string; node_i?: string }) {
  return member.start ?? member.startNode ?? member.start_node ?? member.i ?? member.node_i;
}
function memberEndId(member: { end?: string; endNode?: string; end_node?: string; j?: string; node_j?: string }) {
  return member.end ?? member.endNode ?? member.end_node ?? member.j ?? member.node_j;
}
function loadMemberId(load: RenderLoad) {
  const label = load.targetLabel ?? load.target_label ?? '';
  if (label.startsWith('member ')) return label.slice(7);
  return load.targetMember ?? load.target_member ?? load.memberId ?? load.member_id;
}
function loadNodeId(load: RenderLoad) {
  const label = load.targetLabel ?? load.target_label ?? '';
  if (label.startsWith('node ')) return label.slice(5);
  return load.targetNode ?? load.target_node;
}
function isSelfWeightLoad(load: RenderLoad) {
  return (load.semanticLabel ?? load.semantic_label) === 'self_weight';
}
function releaseMemberId(release: RenderRelease) {
  return release.memberId ?? release.member_id;
}
function releaseEnd(release: RenderRelease) {
  return String(release.end ?? 'end').toLowerCase() === 'start' ? 'start' : 'end';
}
function supportDisplayId(support: RenderSupport, index: number) {
  return String(index + 1);
}
function memberLabelPoint(points: THREE.Vector3[]) {
  if (!points.length) return new THREE.Vector3();
  if (points.length === 1) return points[0].clone();
  const total = points.slice(1).reduce((sum, point, index) => sum + points[index].distanceTo(point), 0);
  if (total <= 1e-9) return points[0].clone();
  let travelled = 0;
  const halfway = total / 2;
  for (let index = 1; index < points.length; index += 1) {
    const a = points[index - 1];
    const b = points[index];
    const length = a.distanceTo(b);
    if (travelled + length >= halfway) {
      return a.clone().lerp(b, (halfway - travelled) / length);
    }
    travelled += length;
  }
  return points[points.length - 1].clone();
}

const MEMBER_END_DISPLAY_INSET_RATIO = 0.035;
const MEMBER_END_DISPLAY_MAX_TRIM_RATIO = 0.45;
const RELEASE_TICK_COLORS = {
  x: '#ef4444',
  y: '#22c55e',
  z: '#3b82f6',
};
const NODE_POINT_SIZE_PX = 12;
const NODE_POINT_RADIUS_RATIO = 28 / 64;
const INFERENCE_AXIS_COLOR_NUMBERS: Record<InferenceAxis, number> = {
  x: 0xef4444,
  y: 0x22c55e,
  z: 0x3b82f6,
};
const PROJECTION_PLANE_AXES: Record<ProjectionPlane, [InferenceAxis, InferenceAxis]> = {
  xy: ['x', 'y'],
  xz: ['x', 'z'],
  yz: ['y', 'z'],
};
const PROJECTION_PLANE_NORMALS: Record<ProjectionPlane, InferenceAxis> = {
  xy: 'z',
  xz: 'y',
  yz: 'x',
};
const PROJECTION_ARROW_HEAD_LENGTH_PX = 12;
const PROJECTION_ARROW_HEAD_RADIUS_PX = 4.5;
const PROJECTION_ARROW_SHAFT_WIDTH_PX = 2.4;
const PROJECTION_FINAL_ENDPOINT_GAP_PX = 12;
const PROJECTION_MIN_COMPONENT_SCREEN_LENGTH_PX = 20;
const PROJECTION_DEPTH_TIE_EPSILON = 0.002;
const LAYER_HALO_EXTRA_PX = 7;
const EDIT_RENDER_ORDER = {
  preview: 12,
  activePreviewHalo: 20.8,
  activePreview: 21,
  activePreviewNode: 23,
  projectionAxisHalo: 20.55,
  projectionArrowHalo: 20.55,
  projectionAxis: 20.65,
  projectionArrow: 22,
  node: 21.1,
  schemePreviewNode: 21.2,
  focusedNode: 21.3,
  hoverNode: 21.04,
  selectedNode: 21.05,
};

function memberEndDisplayInset(length: number) {
  return Math.max(0, length * MEMBER_END_DISPLAY_INSET_RATIO);
}

export function Viewport3D({
  scene,
  focusedTargets = [],
  fitInsets,
  labelVisibility = { node: true, member: true, support: true, load: true },
  selectionEnabled = true,
  cameraScopeKey = 'default',
  navigationProfileId = DEFAULT_VIEWPORT_NAVIGATION_PROFILE_ID,
  customNavigationSettings = DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS,
  onSelectTarget,
  onSelectionGesture,
  onViewportClick,
  onViewportPointerMove,
  editOverlay,
}: {
  scene: RenderScene;
  focusedTargets?: AgentTarget[];
  fitInsets?: ViewportFitInsets;
  labelVisibility?: ViewportLabelVisibility;
  selectionEnabled?: boolean;
  cameraScopeKey?: string;
  navigationProfileId?: ViewportNavigationProfileId;
  customNavigationSettings?: ViewportCustomNavigationSettings;
  onSelectTarget?: (target: AgentTarget | null) => void;
  onSelectionGesture?: (gesture: ViewportSelectionGesture) => void;
  onViewportClick?: (event: ViewportPointerInfo) => void;
  onViewportPointerMove?: (event: ViewportPointerInfo) => void;
  editOverlay?: ViewportEditOverlay | null;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const onSelectTargetRef = useRef(onSelectTarget);
  const onViewportClickRef = useRef(onViewportClick);
  const onViewportPointerMoveRef = useRef(onViewportPointerMove);
  const selectionEnabledRef = useRef(selectionEnabled);
  const sceneApiRef = useRef<{
    updateFocusedTargets: (targets: AgentTarget[]) => void;
    updateFitInsets: (insets: Required<ViewportFitInsets>) => void;
    updateEditOverlay: (overlay: ViewportEditOverlay | null | undefined) => void;
    updateLabelVisibility: (visibility: ViewportLabelVisibility) => void;
    updateNavigationProfile: (profileId: ViewportNavigationProfileId, customSettings: ViewportCustomNavigationSettings) => void;
  } | null>(null);
  const onSelectionGestureRef = useRef(onSelectionGesture);
  const fitInsetLeft = Math.max(0, fitInsets?.left ?? 0);
  const fitInsetRight = Math.max(0, fitInsets?.right ?? 0);
  const fitInsetTop = Math.max(0, fitInsets?.top ?? 0);
  const fitInsetBottom = Math.max(0, fitInsets?.bottom ?? 0);
  useEffect(() => {
    sceneApiRef.current?.updateFocusedTargets(focusedTargets);
  }, [focusedTargets]);

  useEffect(() => {
    onSelectTargetRef.current = onSelectTarget;
  }, [onSelectTarget]);

  useEffect(() => {
    onViewportClickRef.current = onViewportClick;
  }, [onViewportClick]);

  useEffect(() => {
    onSelectionGestureRef.current = onSelectionGesture;
  }, [onSelectionGesture]);

  useEffect(() => {
    selectionEnabledRef.current = selectionEnabled;
  }, [selectionEnabled]);

  useEffect(() => {
    onViewportPointerMoveRef.current = onViewportPointerMove;
  }, [onViewportPointerMove]);

  useEffect(() => {
    sceneApiRef.current?.updateFitInsets({
      left: fitInsetLeft,
      right: fitInsetRight,
      top: fitInsetTop,
      bottom: fitInsetBottom,
    });
  }, [fitInsetLeft, fitInsetRight, fitInsetTop, fitInsetBottom]);

  useEffect(() => {
    sceneApiRef.current?.updateEditOverlay(editOverlay);
  }, [editOverlay]);

  useEffect(() => {
    sceneApiRef.current?.updateLabelVisibility(labelVisibility);
  }, [labelVisibility]);

  useEffect(() => {
    sceneApiRef.current?.updateNavigationProfile(navigationProfileId, customNavigationSettings);
  }, [customNavigationSettings, navigationProfileId]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const viewportElement = el;

    el.style.position = 'relative';
    el.style.overflow = 'hidden';
    el.style.backgroundColor = viewportBackgroundColor();
    const displayMembers = displayMembersFor(scene);
    const unitProfile = {
      ...unitProfileFrom(scene.unitProfile ?? metricStructuralUnitProfile),
      length: { symbol: 'm', canonicalToDisplay: 1, precision: 2 },
    };
    const focusedMembers = new Set(focusedTargets.filter((target) => target.kind === 'member').map((target) => target.id));
    const focusedNodes = new Set(focusedTargets.filter((target) => target.kind === 'node').map((target) => target.id));
    const focusedSupports = new Set(focusedTargets.filter((target) => target.kind === 'support').map((target) => target.id));
    let currentFitInsetLeft = fitInsetLeft;
    let currentFitInsetRight = fitInsetRight;
    let currentFitInsetTop = fitInsetTop;
    let currentFitInsetBottom = fitInsetBottom;
    let currentLabelVisibility = { ...labelVisibility };
    let currentEditOverlay = editOverlay;
    let currentFocusedTargets = focusedTargets;
    let currentNavigationProfileId = navigationProfileId;
    let currentCustomNavigationSettings = customNavigationSettings;
    let hoveredTarget: AgentTarget | null = null;
    let hoveredMemberAnchor: THREE.Vector3 | null = null;
    let hoveredTargetSource: 'label' | 'geometry' | null = null;
    const nodesById = nodeMap(scene);
    const proposedSupportNodeIds = new Set(
      (scene.supports ?? [])
        .filter(isBriefVisualSupport)
        .map(supportNodeId)
        .filter((nodeId): nodeId is string => Boolean(nodeId)),
    );
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false, powerPreference: 'high-performance' });
    renderer.autoClear = false;
    renderer.domElement.dataset.fraiaCanvasRole = 'viewport-webgl';
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.domElement.style.position = 'absolute';
    renderer.domElement.style.inset = '0';
    renderer.domElement.style.display = 'block';
    renderer.domElement.style.width = '100%';
    renderer.domElement.style.height = '100%';
    renderer.domElement.style.touchAction = 'none';
    renderer.domElement.style.backgroundColor = viewportBackgroundColor();
    el.appendChild(renderer.domElement);
    const selectionCanvas = document.createElement('canvas');
    selectionCanvas.dataset.fraiaCanvasRole = 'selection-overlay';
    selectionCanvas.style.position = 'absolute';
    selectionCanvas.style.inset = '0';
    selectionCanvas.style.pointerEvents = 'none';
    selectionCanvas.style.display = 'block';
    selectionCanvas.style.width = '100%';
    selectionCanvas.style.height = '100%';
    el.appendChild(selectionCanvas);
    const selectionCtx = selectionCanvas.getContext('2d')!;
    const s = new THREE.Scene();
    let viewSize = 20;
    const camera = new THREE.OrthographicCamera(-10, 10, 10, -10, -1000, 1000);
    let currentVisualProfile = viewportVisualProfile(camera.zoom);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = false;
    controls.zoomToCursor = true;

    const light = new THREE.DirectionalLight(0xffffff, 2);
    light.position.set(8, 12, 10);
    s.add(light, new THREE.AmbientLight(0x9ccaff, 1.3));

    const viewGizmoOptions = (): GizmoOptions => {
      const dark = isDarkMode();
      const darkFaceColor = 0x111827;
      const darkHoverColor = 0x334155;
      const darkBackgroundColor = 0x1f2937;
      const xLabelColor = 0xef4444;
      const yLabelColor = 0x22c55e;
      const zLabelColor = 0x3b82f6;
      const faceLabel = (label: string, labelColor: number) => ({
        label: `  ${label}  `,
        labelColor,
        ...(dark ? { color: darkFaceColor } : {}),
        hover: {
          labelColor,
          ...(dark ? { color: darkHoverColor } : {}),
        },
      });
      return {
        container: el,
        type: 'cube',
        resolution: 256,
        placement: 'top-right',
        offset: {
          top: Math.max(12, currentFitInsetTop + 12),
          right: Math.max(12, currentFitInsetRight + 12),
          bottom: Math.max(12, currentFitInsetBottom + 12),
          left: Math.max(12, currentFitInsetLeft + 12),
        },
        ...(dark
          ? {
            background: {
              color: darkBackgroundColor,
              hover: { color: darkBackgroundColor },
            },
            edges: {
              color: 0x475569,
              hover: { color: 0x64748b },
            },
          }
          : {}),
        corners: {
          opacity: 0,
          scale: 0.2,
          ...(dark ? { color: 0x475569 } : {}),
          hover: {
            opacity: 1,
            scale: 0.2,
            ...(dark ? { color: 0x64748b } : {}),
          },
        },
        x: faceLabel('X', xLabelColor),
        y: faceLabel('Y', yLabelColor),
        z: faceLabel('Z', zLabelColor),
        nx: faceLabel('-X', xLabelColor),
        ny: faceLabel('-Y', yLabelColor),
        nz: faceLabel('-Z', zLabelColor),
      };
    };
    const viewGizmo = new ViewportGizmo(camera, renderer, viewGizmoOptions());
    installOrthographicViewCubeCamera(viewGizmo);
    viewGizmo.attachControls(controls);
    let viewGizmoTheme = isDarkMode() ? 'dark' : 'light';
    let viewGizmoPlacement = [
      currentFitInsetTop,
      currentFitInsetRight,
      currentFitInsetBottom,
      currentFitInsetLeft,
    ].join(':');
    let viewGizmoHoverElement: HTMLElement | null = null;
    type ViewGizmoInternals = {
      _intersections?: THREE.Object3D[];
      _background?: THREE.Object3D | null;
      _domElement?: HTMLElement;
      _focus?: THREE.Object3D | null;
    };
    type ViewGizmoMaterial = THREE.Material & {
      color?: THREE.Color;
      map?: THREE.Texture | null;
      opacity: number;
    };
    type ViewGizmoObjectState = {
      color?: THREE.ColorRepresentation;
      opacity?: number;
      scale?: number;
      hover?: {
        color?: THREE.ColorRepresentation;
        opacity?: number;
        scale?: number;
      };
    };
    function viewGizmoInternals() {
      return viewGizmo as unknown as ViewGizmoInternals;
    }

    function firstViewGizmoMaterial(object: THREE.Object3D) {
      const material = (object as unknown as { material?: THREE.Material | THREE.Material[] }).material;
      return (Array.isArray(material) ? material[0] : material) as ViewGizmoMaterial | undefined;
    }

    function sharpenViewGizmoLabelTextures() {
      const anisotropy = Math.max(1, Math.min(16, renderer.capabilities.getMaxAnisotropy()));
      const seenTextures = new Set<THREE.Texture>();
      viewGizmoInternals()._intersections?.forEach((object) => {
        const texture = firstViewGizmoMaterial(object)?.map;
        if (!texture || seenTextures.has(texture)) return;
        seenTextures.add(texture);
        texture.anisotropy = anisotropy;
        texture.generateMipmaps = true;
        texture.minFilter = THREE.LinearMipmapLinearFilter;
        texture.magFilter = THREE.LinearFilter;
        texture.needsUpdate = true;
      });
    }

    function applyViewGizmoHoverState(object: THREE.Object3D | null | undefined, hovered: boolean, focused: boolean) {
      if (!object) return;
      const state = object.userData as ViewGizmoObjectState;
      const material = firstViewGizmoMaterial(object);
      const isFaceButton = Boolean(material?.map);
      const next = focused ? state.hover : state;
      const nextOpacity = hovered && !isFaceButton && !focused ? 1 : next?.opacity;
      if (typeof next?.scale === 'number') object.scale.setScalar(next.scale);
      if (!material || !next) return;
      if (typeof nextOpacity === 'number') material.opacity = nextOpacity;
      if (material.map) {
        const offsetX = (material.map.userData as { offsetX?: number }).offsetX ?? 0;
        material.map.offset.x = (focused ? 0.5 : 0) + offsetX;
      } else if (material.color && next.color !== undefined) {
        material.color.set(next.color);
      }
    }

    function setViewGizmoGlobalHover(hovered: boolean) {
      const internals = viewGizmoInternals();
      applyViewGizmoHoverState(internals._background, hovered, hovered);
      const focus = hovered ? internals._focus : null;
      internals._intersections?.forEach((object) => applyViewGizmoHoverState(object, hovered, object === focus));
      if (!hovered) internals._focus = null;
      scheduleRender();
    }

    const handleViewGizmoGlobalHover = () => setViewGizmoGlobalHover(true);
    const handleViewGizmoGlobalLeave = () => setViewGizmoGlobalHover(false);

    function unbindViewGizmoGlobalHover() {
      if (!viewGizmoHoverElement) return;
      viewGizmoHoverElement.removeEventListener('pointerenter', handleViewGizmoGlobalHover);
      viewGizmoHoverElement.removeEventListener('pointermove', handleViewGizmoGlobalHover);
      viewGizmoHoverElement.removeEventListener('pointerleave', handleViewGizmoGlobalLeave);
      viewGizmoHoverElement = null;
    }

    function bindViewGizmoGlobalHover() {
      unbindViewGizmoGlobalHover();
      const element = viewGizmoInternals()._domElement;
      if (!element) return;
      viewGizmoHoverElement = element;
      element.addEventListener('pointerenter', handleViewGizmoGlobalHover);
      element.addEventListener('pointermove', handleViewGizmoGlobalHover);
      element.addEventListener('pointerleave', handleViewGizmoGlobalLeave);
    }
    sharpenViewGizmoLabelTextures();
    bindViewGizmoGlobalHover();

    function refreshViewGizmoIfNeeded({ force = false } = {}) {
      const nextTheme = isDarkMode() ? 'dark' : 'light';
      const nextPlacement = [
        currentFitInsetTop,
        currentFitInsetRight,
        currentFitInsetBottom,
        currentFitInsetLeft,
      ].join(':');
      if (!force && nextTheme === viewGizmoTheme && nextPlacement === viewGizmoPlacement) return;
      viewGizmoTheme = nextTheme;
      viewGizmoPlacement = nextPlacement;
      unbindViewGizmoGlobalHover();
      viewGizmo.set(viewGizmoOptions());
      installOrthographicViewCubeCamera(viewGizmo);
      viewGizmo.attachControls(controls);
      sharpenViewGizmoLabelTextures();
      bindViewGizmoGlobalHover();
      scheduleRender();
    }

    const memberObjects: Line2[] = [];
    const memberBatchObjects: LineSegments2[] = [];
    const memberHitSegments: Array<{ memberId: string; start: THREE.Vector3; end: THREE.Vector3; preview: boolean }> = [];
    const memberVisualSegments: Array<{ memberId: string; rawStart: THREE.Vector3; rawEnd: THREE.Vector3; start: THREE.Vector3; end: THREE.Vector3; preview: boolean }> = [];
    const nodeHitPoints: Array<{ nodeId: string; point: THREE.Vector3 }> = [];
    const supportNodeById = new Map<string, string>();
    let baseMemberBatch: LineSegments2 | null = null;
    let baseMemberHaloBatch: LineSegments2 | null = null;
    let previewMemberHaloBatch: LineSegments2 | null = null;
    let previewMemberBatch: LineSegments2 | null = null;
    let selectedMemberBatch: LineSegments2 | null = null;
    let hoverMemberBatch: LineSegments2 | null = null;
    const labelsEnabled = scene.members.length < AUTO_LABEL_MEMBER_LIMIT;
    type LabelPlacementDirection = 'below' | 'above' | 'right' | 'left' | 'below-left' | 'below-right' | 'above-left' | 'above-right';
    type LabelVisualState = 'base' | 'hover' | 'selected';
    type LabelTextureSpec = {
      texture: THREE.CanvasTexture;
      widthPx: number;
      heightPx: number;
      stateTextures?: {
        base: THREE.CanvasTexture;
        hover: THREE.CanvasTexture;
        selected: THREE.CanvasTexture;
      };
    };
    type LoadLabelLeaderVisual = {
      line: Line2;
      halo: Line2 | null;
    };
    type LoadLabelTrack = {
      start: THREE.Vector3;
      end: THREE.Vector3;
      direction: THREE.Vector3;
      leader?: LoadLabelLeaderVisual;
      labelFraction?: number;
    };
    type LabelSprite = {
      sprite: THREE.Sprite;
      material: THREE.SpriteMaterial;
      texture: THREE.CanvasTexture;
      stateTextures?: LabelTextureSpec['stateTextures'];
      widthPx: number;
      heightPx: number;
      anchor: THREE.Vector3;
      anchorClearancePx: number;
      offset: { x: number; y: number };
      priority: number;
      kind: 'node' | 'member' | 'support' | 'load';
      ownerTargets: AgentTarget[];
      hoverTargets?: AgentTarget[];
      placement?: 'pinned' | 'anchored' | 'load-line' | 'load-point';
      loadTrack?: LoadLabelTrack;
      themeTextureFactory?: () => LabelTextureSpec;
      themeOffsetFactory?: (textureSpec: LabelTextureSpec) => { x: number; y: number };
      hoverOnly?: boolean;
      placementDirection?: LabelPlacementDirection;
      wasVisible?: boolean;
      transition?: {
        startedAt: number;
        fromPosition: THREE.Vector3;
        fromScale: THREE.Vector3;
      };
    };
    const memberLabelSprites: Array<LabelSprite & { member: DisplayMember }> = [];
    const nodeLabelSprites: LabelSprite[] = [];
    const loadLabelSprites: LabelSprite[] = [];
    const supportLabelSprites: LabelSprite[] = [];
    const symbolSprites: Array<{ sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; widthPx: number; heightPx: number; anchor: THREE.Vector3; direction?: THREE.Vector3; offset?: { x: number; y: number }; pin?: { x: number; y: number }; tone: 'support' | 'load' | 'release'; focused: boolean; proposed?: boolean; target?: AgentTarget; halo?: { sprite: THREE.Sprite; material: THREE.SpriteMaterial } }> = [];
    const supportHitPoints: Array<{ supportId: string; nodeId: string; point: THREE.Vector3; kind: string }> = [];
    const loadHitAnchors: Array<{ loadId: string; point: THREE.Vector3 }> = [];
    const loadHitSegments: Array<{ loadId: string; start: THREE.Vector3; end: THREE.Vector3 }> = [];
    type LoadArrowVisual = {
      shaft: Line2;
      halo: Line2 | null;
      head: Line2;
      headHalo: Line2 | null;
      tail: THREE.Vector3;
      tip: THREE.Vector3;
    };
    type UniformLineLoadLayout = {
      start: THREE.Vector3;
      end: THREE.Vector3;
      direction: THREE.Vector3;
      tangent: THREE.Vector3;
      tailOffset: number;
    };
    type ParallelLineLoadLayout = {
      start: THREE.Vector3;
      tangent: THREE.Vector3;
      arrowTangent: THREE.Vector3;
      sign: number;
      arrowLength: number;
      firstCenter: number;
      lastCenter: number;
    };
    type LoadLineVisualGroup = {
      start: THREE.Vector3;
      end: THREE.Vector3;
      arrows: LoadArrowVisual[];
      focused: boolean;
      layout?: UniformLineLoadLayout | ParallelLineLoadLayout;
    };
    const loadArrowSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3; visual: LoadArrowVisual }> = [];
    const loadLineVisualGroups: LoadLineVisualGroup[] = [];
    const loadLabelLeaders: LoadLabelLeaderVisual[] = [];
    const loadInteractionStrokes: Array<{ loadId: string; line: Line2; halo: Line2 | null; baseRenderOrder: number }> = [];
    const nodeObjects: THREE.Points[] = [];
    const labelAnchorGapPx = 6;
    // Hover member labels are re-anchored on every pointermove; keep their side stable across near-tied placements.
    const hoverMemberPlacementHysteresis = 2000000;
    const editInferenceLabels: Array<{ sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number; offset: { x: number; y: number }; axis?: InferenceAxis; sign?: number }> = [];
    const editInferenceAxisCues: Array<{ anchor: THREE.Vector3; axis: InferenceAxis; sign: number }> = [];
    const editProjectionGuides: Array<{ plane: ProjectionPlane; start: THREE.Vector3; projectedEnd: THREE.Vector3; realEnd: THREE.Vector3; angles: ProjectionGuideAngle[]; outOfPlaneAxis?: InferenceAxis; angle?: { axis: InferenceAxis; angleDeg: number } }> = [];
    const editProjectionDimensionLabels: Record<InferenceAxis, { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number; text: string; axis: InferenceAxis; sign: number } | null> = { x: null, y: null, z: null };
    let activeProjectionVectorSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3; axis: InferenceAxis }> = [];
    let activePreviewVisualSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }> = [];
    let activePreviewSplitMaskSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }> = [];
    type EditPrimitiveLabel = { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number; offset: { x: number; y: number }; anchorClearancePx: number };
    const editPrimitiveLabels: EditPrimitiveLabel[] = [];
    let editCoordinateLabel: { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number } | null = null;
    let editProjectionAngleLabel: { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number; text: string } | null = null;

    const baseMemberHaloMat = new LineMaterial({ linewidth: viewportStroke.member + LAYER_HALO_EXTRA_PX, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false });
    const memberBatchMat = new LineMaterial({ linewidth: viewportStroke.member, worldUnits: false, transparent: true, opacity: 1, depthTest: true, depthWrite: false });
    const previewMemberHaloMat = new LineMaterial({ linewidth: viewportStroke.member + 7, worldUnits: false, transparent: true, depthTest: false, depthWrite: false });
    const previewMemberBatchMat = new LineMaterial({ linewidth: viewportStroke.member + 0.8, worldUnits: false, transparent: true, depthTest: false, depthWrite: false });
    const selectedMemberBatchMat = new LineMaterial({ linewidth: viewportStroke.member, worldUnits: false, dashed: true, dashSize: 0.45, gapSize: 0.3, transparent: true, depthTest: false, depthWrite: false });
    const hoverMemberMat = new LineMaterial({ linewidth: viewportStroke.member, worldUnits: false, transparent: true, depthTest: false, depthWrite: false });
    const loadMat = new LineMaterial({ linewidth: loadArrowSymbol.strokeWidth, worldUnits: false, transparent: true, depthTest: false, depthWrite: false });
    const focusedLoadMat = new LineMaterial({ linewidth: loadArrowSymbol.strokeWidth, worldUnits: false, transparent: true, depthTest: false, depthWrite: false });
    const loadHaloMat = new LineMaterial({ linewidth: loadArrowSymbol.strokeWidth + LAYER_HALO_EXTRA_PX, worldUnits: false, transparent: true, opacity: 0.94, depthTest: false, depthWrite: false });
    const focusedLoadHaloMat = new LineMaterial({ linewidth: loadArrowSymbol.strokeWidth + 4, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false });
    const hoverLoadHaloMat = new LineMaterial({ linewidth: loadArrowSymbol.strokeWidth + 3, worldUnits: false, transparent: true, opacity: 0.48, depthTest: false, depthWrite: false });
    const releaseLineMaterial = (axis: keyof typeof RELEASE_TICK_COLORS, focused = false) => new LineMaterial({
      color: RELEASE_TICK_COLORS[axis],
      linewidth: focused ? viewportStroke.symbol * 0.9 : viewportStroke.symbol * 0.7,
      worldUnits: false,
      transparent: true,
      depthTest: false,
      depthWrite: false,
    });
    const releaseHaloMat = new LineMaterial({
      linewidth: viewportStroke.symbol * 0.9 + LAYER_HALO_EXTRA_PX,
      worldUnits: false,
      transparent: true,
      opacity: 0.94,
      depthTest: false,
      depthWrite: false,
    });
    const releaseMats = {
      x: releaseLineMaterial('x'),
      y: releaseLineMaterial('y'),
      z: releaseLineMaterial('z'),
    };
    const focusedReleaseMats = {
      x: releaseLineMaterial('x', true),
      y: releaseLineMaterial('y', true),
      z: releaseLineMaterial('z', true),
    };
    const allReleaseMats = [...Object.values(releaseMats), ...Object.values(focusedReleaseMats)];
    const nodeTexture = makeNodeTexture();
    const nodeMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const proposedSupportNodeMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const focusedNodeMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const selectedNodeFillMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX + 4, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const hoverNodeFillMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX + 2, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const previewNodeFillMat = new THREE.PointsMaterial({ size: NODE_POINT_SIZE_PX + 2, sizeAttenuation: false, map: nodeTexture, alphaTest: 0.2, transparent: true, depthTest: false, depthWrite: false });
    const editGridMat = new LineMaterial({ color: 0x64748b, linewidth: 1, worldUnits: false, transparent: true, opacity: 0.3, depthTest: false, depthWrite: false });
    const editPreviewHaloMat = new LineMaterial({ color: viewportBackgroundColor(), linewidth: viewportStroke.member + 7, worldUnits: false, transparent: true, opacity: 0.92, depthTest: false, depthWrite: false });
    const editPreviewMat = new LineMaterial({ color: viewportMemberColor(), linewidth: viewportStroke.member + 0.8, worldUnits: false, transparent: true, opacity: 1, depthTest: false, depthWrite: false });
    const editPreviewSplitMat = new LineMaterial({ color: viewportMemberPreviewColor(), linewidth: viewportStroke.member, worldUnits: false, transparent: true, opacity: 0.94, depthTest: false, depthWrite: false });
    const editGuideMats: Record<SnapGuideAxis, LineMaterial> = {
      x: new LineMaterial({ color: 0xef4444, linewidth: 2, worldUnits: false, transparent: true, opacity: 0.78, depthTest: false, depthWrite: false }),
      y: new LineMaterial({ color: 0x22c55e, linewidth: 2, worldUnits: false, transparent: true, opacity: 0.78, depthTest: false, depthWrite: false }),
      z: new LineMaterial({ color: 0x3b82f6, linewidth: 2, worldUnits: false, transparent: true, opacity: 0.78, depthTest: false, depthWrite: false }),
      angle: new LineMaterial({ color: 0xf59e0b, linewidth: 1.8, worldUnits: false, transparent: true, opacity: 0.78, depthTest: false, depthWrite: false }),
    };
    const editGuideHaloMat = new LineMaterial({ linewidth: 2 + LAYER_HALO_EXTRA_PX, worldUnits: false, transparent: true, opacity: 0.88, depthTest: false, depthWrite: false });
    const editGridGeometry = new LineSegmentsGeometry();
    const editPreviewHaloGeometry = new LineSegmentsGeometry();
    const editPreviewGeometry = new LineSegmentsGeometry();
    const editPreviewSplitGeometry = new LineSegmentsGeometry();
    const editPreviewNodeGeometry = new THREE.BufferGeometry();
    const editGridLine = new LineSegments2(editGridGeometry, editGridMat);
    const editPreviewHaloLine = new LineSegments2(editPreviewHaloGeometry, editPreviewHaloMat);
    const editPreviewLine = new LineSegments2(editPreviewGeometry, editPreviewMat);
    const editPreviewSplitLine = new LineSegments2(editPreviewSplitGeometry, editPreviewSplitMat);
    const editPreviewForegroundHaloLine = new LineSegments2(new LineSegmentsGeometry(), editPreviewHaloMat);
    const editPreviewForegroundLine = new LineSegments2(new LineSegmentsGeometry(), editPreviewMat);
    const editPreviewNodeFill = new THREE.Points(editPreviewNodeGeometry, hoverNodeFillMat);
    const editGuideLines: Record<SnapGuideAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editGuideMats.x),
      y: new LineSegments2(new LineSegmentsGeometry(), editGuideMats.y),
      z: new LineSegments2(new LineSegmentsGeometry(), editGuideMats.z),
      angle: new LineSegments2(new LineSegmentsGeometry(), editGuideMats.angle),
    };
    const editGuideHaloLines: Record<SnapGuideAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editGuideHaloMat),
      y: new LineSegments2(new LineSegmentsGeometry(), editGuideHaloMat),
      z: new LineSegments2(new LineSegmentsGeometry(), editGuideHaloMat),
      angle: new LineSegments2(new LineSegmentsGeometry(), editGuideHaloMat),
    };
    const editInferenceAxisMats: Record<InferenceAxis, LineMaterial> = {
      x: new LineMaterial({ color: 0xef4444, linewidth: 2.4, worldUnits: false, transparent: true, opacity: 0.9, depthTest: false, depthWrite: false }),
      y: new LineMaterial({ color: 0x22c55e, linewidth: 2.4, worldUnits: false, transparent: true, opacity: 0.9, depthTest: false, depthWrite: false }),
      z: new LineMaterial({ color: 0x3b82f6, linewidth: 2.4, worldUnits: false, transparent: true, opacity: 0.9, depthTest: false, depthWrite: false }),
    };
    const editInferenceAxisHaloMat = new LineMaterial({ linewidth: 2.4 + LAYER_HALO_EXTRA_PX, worldUnits: false, transparent: true, opacity: 0.9, depthTest: false, depthWrite: false });
    const editInferenceAxisLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisMats.x),
      y: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisMats.y),
      z: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisMats.z),
    };
    const editInferenceAxisHaloLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisHaloMat),
      y: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisHaloMat),
      z: new LineSegments2(new LineSegmentsGeometry(), editInferenceAxisHaloMat),
    };
    const editProjectionAxisMats: Record<InferenceAxis, LineMaterial> = {
      x: new LineMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.x, linewidth: PROJECTION_ARROW_SHAFT_WIDTH_PX, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false }),
      y: new LineMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.y, linewidth: PROJECTION_ARROW_SHAFT_WIDTH_PX, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false }),
      z: new LineMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.z, linewidth: PROJECTION_ARROW_SHAFT_WIDTH_PX, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false }),
    };
    const editProjectionAxisHaloMat = new LineMaterial({ linewidth: PROJECTION_ARROW_SHAFT_WIDTH_PX + LAYER_HALO_EXTRA_PX, worldUnits: false, transparent: true, opacity: 0.9, depthTest: false, depthWrite: false });
    const editProjectionAngleMat = new LineMaterial({ color: 0xf59e0b, linewidth: 2.2, worldUnits: false, transparent: true, opacity: 0.96, depthTest: false, depthWrite: false });
    const editProjectionAngleHaloMat = new LineMaterial({ linewidth: 0.001, worldUnits: false, transparent: true, opacity: 0, depthTest: false, depthWrite: false });
    const editProjectionAxisLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.x),
      y: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.y),
      z: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.z),
    };
    const editProjectionAxisHaloLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
      y: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
      z: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
    };
    const editProjectionForegroundAxisLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.x),
      y: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.y),
      z: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisMats.z),
    };
    const editProjectionForegroundAxisHaloLines: Record<InferenceAxis, LineSegments2> = {
      x: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
      y: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
      z: new LineSegments2(new LineSegmentsGeometry(), editProjectionAxisHaloMat),
    };
    const editProjectionAngleLine = new LineSegments2(new LineSegmentsGeometry(), editProjectionAngleMat);
    const editProjectionAngleHaloLine = new LineSegments2(new LineSegmentsGeometry(), editProjectionAngleHaloMat);
    const editProjectionArrowGeometry = new THREE.ConeGeometry(0.32, 1, 24);
    const editProjectionArrowHaloMat = new THREE.MeshBasicMaterial({ transparent: true, opacity: 0.9, depthTest: false, depthWrite: false });
    const editProjectionArrowMats: Record<InferenceAxis, THREE.MeshBasicMaterial> = {
      x: new THREE.MeshBasicMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.x, transparent: true, opacity: 1, depthTest: false, depthWrite: false }),
      y: new THREE.MeshBasicMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.y, transparent: true, opacity: 1, depthTest: false, depthWrite: false }),
      z: new THREE.MeshBasicMaterial({ color: INFERENCE_AXIS_COLOR_NUMBERS.z, transparent: true, opacity: 1, depthTest: false, depthWrite: false }),
    };
    const editProjectionArrowHeads: Record<InferenceAxis, THREE.Mesh> = {
      x: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowMats.x),
      y: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowMats.y),
      z: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowMats.z),
    };
    const editProjectionArrowHeadHalos: Record<InferenceAxis, THREE.Mesh> = {
      x: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowHaloMat),
      y: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowHaloMat),
      z: new THREE.Mesh(editProjectionArrowGeometry, editProjectionArrowHaloMat),
    };
    let editSnapGlyph: { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number } | null = null;
    let editSnapLabel: { sprite: THREE.Sprite; material: THREE.SpriteMaterial; texture: THREE.CanvasTexture; anchor: THREE.Vector3; widthPx: number; heightPx: number } | null = null;
    editGridLine.frustumCulled = false;
    editPreviewHaloLine.frustumCulled = false;
    editPreviewLine.frustumCulled = false;
    editPreviewSplitLine.frustumCulled = false;
    editPreviewForegroundHaloLine.frustumCulled = false;
    editPreviewForegroundLine.frustumCulled = false;
    Object.values(editGuideLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 64;
      line.visible = false;
    });
    Object.values(editGuideHaloLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 63;
      line.visible = false;
    });
    Object.values(editInferenceAxisLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 66;
      line.visible = false;
    });
    Object.values(editInferenceAxisHaloLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 65;
      line.visible = false;
    });
    Object.values(editProjectionAxisLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = EDIT_RENDER_ORDER.projectionAxis;
      line.visible = false;
    });
    Object.values(editProjectionAxisHaloLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = EDIT_RENDER_ORDER.projectionAxisHalo;
      line.visible = false;
    });
    Object.values(editProjectionForegroundAxisHaloLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 21;
      line.visible = false;
    });
    Object.values(editProjectionForegroundAxisLines).forEach((line) => {
      line.frustumCulled = false;
      line.renderOrder = 22;
      line.visible = false;
    });
    editProjectionAngleHaloLine.frustumCulled = false;
    editProjectionAngleHaloLine.renderOrder = EDIT_RENDER_ORDER.projectionArrow + 1;
    editProjectionAngleHaloLine.visible = false;
    editProjectionAngleLine.frustumCulled = false;
    editProjectionAngleLine.renderOrder = EDIT_RENDER_ORDER.projectionArrow + 1;
    editProjectionAngleLine.visible = false;
    editGridLine.renderOrder = 2;
    editPreviewHaloLine.renderOrder = EDIT_RENDER_ORDER.activePreviewHalo;
    editPreviewLine.renderOrder = EDIT_RENDER_ORDER.activePreview;
    editPreviewSplitLine.renderOrder = EDIT_RENDER_ORDER.preview;
    editPreviewForegroundHaloLine.renderOrder = EDIT_RENDER_ORDER.activePreviewHalo + 0.2;
    editPreviewForegroundLine.renderOrder = EDIT_RENDER_ORDER.activePreview + 0.2;
    editPreviewNodeFill.renderOrder = EDIT_RENDER_ORDER.activePreviewNode;
    editGridLine.visible = false;
    editPreviewHaloLine.visible = false;
    editPreviewLine.visible = false;
    editPreviewSplitLine.visible = false;
    editPreviewForegroundHaloLine.visible = false;
    editPreviewForegroundLine.visible = false;
    editPreviewNodeFill.visible = false;
    Object.values(editProjectionArrowHeads).forEach((mesh) => {
      mesh.frustumCulled = false;
      mesh.renderOrder = EDIT_RENDER_ORDER.projectionArrow;
      mesh.visible = false;
    });
    Object.values(editProjectionArrowHeadHalos).forEach((mesh) => {
      mesh.frustumCulled = false;
      mesh.renderOrder = EDIT_RENDER_ORDER.projectionArrowHalo;
      mesh.visible = false;
    });
    s.add(
      editGridLine,
      editPreviewHaloLine,
      editPreviewLine,
      editPreviewSplitLine,
      editPreviewForegroundHaloLine,
      editPreviewForegroundLine,
      editPreviewNodeFill,
      ...Object.values(editGuideHaloLines),
      ...Object.values(editGuideLines),
      ...Object.values(editProjectionAxisHaloLines),
      ...Object.values(editProjectionAxisLines),
      ...Object.values(editProjectionForegroundAxisHaloLines),
      ...Object.values(editProjectionForegroundAxisLines),
      editProjectionAngleHaloLine,
      editProjectionAngleLine,
      ...Object.values(editProjectionArrowHeadHalos),
      ...Object.values(editProjectionArrowHeads),
      ...Object.values(editInferenceAxisHaloLines),
      ...Object.values(editInferenceAxisLines),
    );

    function isSchemePreviewMember(member: DisplayMember) {
      return member.sources.includes('scheme') || member.schemeNotes.includes('approval required');
    }

    function isSchemePreviewRenderMember(member: RenderScene['members'][number]) {
      return member.source === 'scheme' || member.schemeNote === 'approval required';
    }

    function createMemberSegmentBatch(positions: number[], material: LineMaterial, renderOrder: number) {
      const geo = new LineSegmentsGeometry();
      geo.setPositions(positions);
      const line = new LineSegments2(geo, material);
      line.computeLineDistances();
      line.frustumCulled = false;
      line.renderOrder = renderOrder;
      memberBatchObjects.push(line);
      s.add(line);
      return line;
    }

    function createGlyphLine(points: THREE.Vector3[], material: LineMaterial, renderOrder = 42, haloMaterial?: LineMaterial) {
      let haloLine: Line2 | null = null;
      if (haloMaterial) {
        const haloGeo = new LineGeometry();
        haloGeo.setPositions(points.flatMap((point) => [point.x, point.y, point.z]));
        haloLine = new Line2(haloGeo, haloMaterial);
        haloLine.computeLineDistances();
        haloLine.renderOrder = Math.max(0, renderOrder - 1);
        memberObjects.push(haloLine);
        s.add(haloLine);
      }
      const geo = new LineGeometry();
      geo.setPositions(points.flatMap((point) => [point.x, point.y, point.z]));
      const line = new Line2(geo, material);
      line.computeLineDistances();
      line.renderOrder = renderOrder;
      memberObjects.push(line);
      s.add(line);
      return { line, haloLine };
    }

    function makeNodeTexture() {
      const canvas = document.createElement('canvas');
      canvas.width = 64;
      canvas.height = 64;
      const ctx = canvas.getContext('2d')!;
      ctx.clearRect(0, 0, 64, 64);
      ctx.fillStyle = '#ffffff';
      ctx.beginPath();
      ctx.arc(32, 32, 28, 0, Math.PI * 2);
      ctx.fill();
      return new THREE.CanvasTexture(canvas);
    }

    function makeSnapGlyphTexture(kind: NonNullable<ViewportEditOverlay['snapPoint']>['kind']) {
      const canvas = document.createElement('canvas');
      canvas.width = 40;
      canvas.height = 40;
      const ctx = canvas.getContext('2d')!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const markerColor = '#22c55e';
      ctx.strokeStyle = markerColor;
      ctx.lineWidth = 2;
      ctx.lineJoin = 'round';
      ctx.lineCap = 'round';
      ctx.beginPath();
      if (kind === 'end') {
        ctx.rect(12, 12, 16, 16);
      } else if (kind === 'mid') {
        ctx.moveTo(20, 10);
        ctx.lineTo(30, 28);
        ctx.lineTo(10, 28);
        ctx.closePath();
      } else if (kind === 'near') {
        ctx.moveTo(20, 10);
        ctx.lineTo(30, 20);
        ctx.lineTo(20, 30);
        ctx.lineTo(10, 20);
        ctx.closePath();
      } else if (kind === 'axis') {
        ctx.moveTo(8, 20);
        ctx.lineTo(32, 20);
        ctx.moveTo(24, 12);
        ctx.lineTo(32, 20);
        ctx.lineTo(24, 28);
      } else {
        ctx.arc(20, 22, 12, -Math.PI * 0.2, Math.PI * 0.95);
        ctx.moveTo(28, 10);
        ctx.lineTo(32, 20);
        ctx.lineTo(22, 18);
      }
      ctx.stroke();
      return { texture: new THREE.CanvasTexture(canvas), widthPx: canvas.width, heightPx: canvas.height };
    }

    function makeSnapLabelTexture(text: string) {
      const dpr = 3;
      const measure = document.createElement('canvas').getContext('2d')!;
      measure.font = '800 11px Inter, ui-sans-serif, system-ui';
      const label = text.trim().slice(0, 18) || 'Snap';
      const widthPx = Math.ceil(Math.max(30, measure.measureText(label).width + 18));
      const heightPx = 24;
      const canvas = document.createElement('canvas');
      canvas.width = widthPx * dpr;
      canvas.height = heightPx * dpr;
      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, widthPx, heightPx);
      roundedRect(ctx, 0.5, 0.5, widthPx - 1, heightPx - 1, 7);
      ctx.fillStyle = isDarkMode() ? 'rgba(20,21,23,0.92)' : 'rgba(255,255,255,0.92)';
      ctx.fill();
      ctx.strokeStyle = isDarkMode() ? 'rgba(248,249,250,0.42)' : 'rgba(20,21,23,0.3)';
      ctx.lineWidth = 1;
      ctx.stroke();
      ctx.font = '800 11px Inter, ui-sans-serif, system-ui';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = isDarkMode() ? '#f8f9fa' : '#171b20';
      ctx.fillText(label, widthPx / 2, heightPx / 2 + 0.5);
      const texture = new THREE.CanvasTexture(canvas);
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.needsUpdate = true;
      return { texture, widthPx, heightPx };
    }

    function inferenceAxisColor(axis: InferenceAxis) {
      if (axis === 'x') return '#ef4444';
      if (axis === 'y') return '#22c55e';
      return '#3b82f6';
    }

    function inferenceAxisOffset(axis: InferenceAxis) {
      if (axis === 'x') return { x: -38, y: -4 };
      if (axis === 'y') return { x: 0, y: -36 };
      return { x: 38, y: -4 };
    }

    function inferenceAxisVector(axis: InferenceAxis) {
      if (axis === 'x') return new THREE.Vector3(1, 0, 0);
      if (axis === 'y') return new THREE.Vector3(0, 1, 0);
      return new THREE.Vector3(0, 0, 1);
    }

    function makeInferenceTextTexture(text: string, color: string, fontSize = 15) {
      const dpr = 3;
      const measure = document.createElement('canvas').getContext('2d')!;
      measure.font = `800 ${fontSize}px Inter, ui-sans-serif, system-ui`;
      const label = text.trim().slice(0, 18) || 'Snap';
      const widthPx = Math.ceil(Math.max(18, measure.measureText(label).width + 10));
      const heightPx = Math.ceil(fontSize + 10);
      const canvas = document.createElement('canvas');
      canvas.width = widthPx * dpr;
      canvas.height = heightPx * dpr;
      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, widthPx, heightPx);
      ctx.font = `800 ${fontSize}px Inter, ui-sans-serif, system-ui`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.lineJoin = 'round';
      ctx.strokeStyle = isDarkMode() ? 'rgba(20,21,23,0.9)' : 'rgba(255,255,255,0.92)';
      ctx.lineWidth = 4;
      ctx.strokeText(label, widthPx / 2, heightPx / 2 + 0.5);
      ctx.fillStyle = color;
      ctx.fillText(label, widthPx / 2, heightPx / 2 + 0.5);
      const texture = new THREE.CanvasTexture(canvas);
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.needsUpdate = true;
      return { texture, widthPx, heightPx };
    }

    function disposeProjectionDimensionLabel(axis: InferenceAxis) {
      const label = editProjectionDimensionLabels[axis];
      if (!label) return;
      s.remove(label.sprite);
      label.material.dispose();
      label.texture.dispose();
      editProjectionDimensionLabels[axis] = null;
    }

    function setProjectionDimensionLabel(axis: InferenceAxis, anchor: THREE.Vector3, sign: number, text: string) {
      const current = editProjectionDimensionLabels[axis];
      if (current?.text === text) {
        current.anchor.copy(anchor);
        current.sign = sign;
        current.sprite.visible = true;
        return;
      }
      disposeProjectionDimensionLabel(axis);
      const { texture, widthPx, heightPx } = makePreviewIndicatorTexture(text, inferenceAxisColor(axis));
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.renderOrder = 99;
      sprite.position.copy(anchor);
      editProjectionDimensionLabels[axis] = { sprite, material, texture, anchor, widthPx, heightPx, text, axis, sign };
      s.add(sprite);
    }

    function disposeProjectionAngleLabel() {
      if (!editProjectionAngleLabel) return;
      s.remove(editProjectionAngleLabel.sprite);
      editProjectionAngleLabel.material.dispose();
      editProjectionAngleLabel.texture.dispose();
      editProjectionAngleLabel = null;
    }

    function setProjectionAngleLabel(anchor: THREE.Vector3, text: string) {
      if (editProjectionAngleLabel?.text === text) {
        editProjectionAngleLabel.anchor.copy(anchor);
        editProjectionAngleLabel.sprite.visible = true;
        return;
      }
      disposeProjectionAngleLabel();
      const { texture, widthPx, heightPx } = makePreviewIndicatorTexture(text, '#f59e0b');
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.renderOrder = 99;
      sprite.position.copy(anchor);
      editProjectionAngleLabel = { sprite, material, texture, anchor, widthPx, heightPx, text };
      s.add(sprite);
    }

    function clearEditPrimitiveLabels() {
      while (editPrimitiveLabels.length) {
        const label = editPrimitiveLabels.pop()!;
        s.remove(label.sprite);
        label.material.dispose();
        label.texture.dispose();
      }
    }

    function addEditPrimitiveLabel(anchor: THREE.Vector3, textureSpec: { texture: THREE.CanvasTexture; widthPx: number; heightPx: number }, offset = { x: 0, y: -34 }, anchorClearancePx = NODE_POINT_SIZE_PX * 0.5) {
      const material = new THREE.SpriteMaterial({ map: textureSpec.texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.renderOrder = 99;
      sprite.position.copy(anchor);
      const label = { sprite, material, texture: textureSpec.texture, anchor, widthPx: textureSpec.widthPx, heightPx: textureSpec.heightPx, offset, anchorClearancePx };
      editPrimitiveLabels.push(label);
      s.add(sprite);
      return label;
    }

    function clearEditInferenceLabels() {
      while (editInferenceLabels.length) {
        const label = editInferenceLabels.pop()!;
        s.remove(label.sprite);
        label.material.dispose();
        label.texture.dispose();
      }
      editInferenceAxisCues.splice(0, editInferenceAxisCues.length);
    }

    function clearEditProjectionGuides() {
      editProjectionGuides.splice(0, editProjectionGuides.length);
    }

    function addEditInferenceLabel(anchor: THREE.Vector3, text: string, color: string, offset: { x: number; y: number }, fontSize = 15, axis?: InferenceAxis, sign = 1) {
      const { texture, widthPx, heightPx } = makeInferenceTextTexture(text, color, fontSize);
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.renderOrder = 99;
      sprite.position.copy(anchor);
      const label = { sprite, material, texture, anchor, widthPx, heightPx, offset, axis, sign };
      editInferenceLabels.push(label);
      s.add(sprite);
    }

    function createNodePoints(points: THREE.Vector3[], focused = false) {
      if (!points.length) return null;
      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.Float32BufferAttribute(points.flatMap((point) => [point.x, point.y, point.z]), 3));
      const pointCloud = new THREE.Points(geo, focused ? focusedNodeMat : nodeMat);
      pointCloud.renderOrder = focused ? EDIT_RENDER_ORDER.focusedNode : EDIT_RENDER_ORDER.node;
      return pointCloud;
    }

    function createDynamicNodePoints(material: THREE.PointsMaterial, renderOrder: number) {
      const pointCloud = new THREE.Points(new THREE.BufferGeometry(), material);
      pointCloud.renderOrder = renderOrder;
      pointCloud.visible = false;
      nodeObjects.push(pointCloud);
      s.add(pointCloud);
      return pointCloud;
    }

    function updateNodePointGeometry(pointCloud: THREE.Points, points: THREE.Vector3[]) {
      pointCloud.geometry.dispose();
      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.Float32BufferAttribute(points.flatMap((point) => [point.x, point.y, point.z]), 3));
      pointCloud.geometry = geo;
      pointCloud.visible = points.length > 0;
    }

    function roundedRect(ctx: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, radius: number) {
      ctx.beginPath();
      ctx.moveTo(x + radius, y);
      ctx.lineTo(x + width - radius, y);
      ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
      ctx.lineTo(x + width, y + height - radius);
      ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
      ctx.lineTo(x + radius, y + height);
      ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
      ctx.lineTo(x, y + radius);
      ctx.quadraticCurveTo(x, y, x + radius, y);
      ctx.closePath();
    }

    type EntityLabelKind = 'node' | 'member' | 'support' | 'load';
    type EntityLabelTone = 'default' | 'proposedMember';
    type EntityLabelRow = { text: string; color?: string; variant?: 'group' | 'detail'; wrap?: boolean; inlineWithPrimary?: boolean };
    type ExpandedEntityLabelLine = string | { text: string; color?: string };

    function proposedMemberColor() {
      return sceneProposedMemberColor();
    }

    function entityLabelTexture(
      kind: EntityLabelKind,
      primaryText: string,
      rows: EntityLabelRow[] = [],
      tone: EntityLabelTone = 'default',
      inkOverride?: string,
    ) {
      const dark = isDarkMode();
      const dpr = 2;
      const measure = document.createElement('canvas').getContext('2d')!;
      const horizontalPadding = 8;
      const maxTextWidth = 320;
      const rowLineHeight = 13;
      const rowGap = 1;
      const groupRows = rows.filter((row) => row.variant === 'group' && !row.inlineWithPrimary);
      const inlineText = rows
        .filter((row) => row.inlineWithPrimary)
        .map((row) => row.text.trim())
        .filter(Boolean)
        .join(' ');
      const detailRows = rows.filter((row) => row.variant !== 'group' && !row.inlineWithPrimary);
      const primaryRowText = inlineText ? `${primaryText} ${inlineText}` : primaryText;
      const nonInlineRows = rows.filter((row) => !row.inlineWithPrimary);
      const orderedRows = kind === 'member'
        ? [
            ...groupRows.map((row) => ({ ...row, isPrimary: false })),
            { text: primaryRowText, color: undefined as string | undefined, variant: undefined as EntityLabelRow['variant'], wrap: false, isPrimary: true },
            ...detailRows.map((row) => ({ ...row, isPrimary: false })),
          ]
        : [
            { text: primaryRowText, color: undefined as string | undefined, variant: undefined as EntityLabelRow['variant'], wrap: false, isPrimary: true },
            ...nonInlineRows.map((row) => ({ ...row, isPrimary: false })),
          ];
      const rowSpecs = orderedRows.map((row) => {
        measure.font = '800 12px Inter, ui-sans-serif, system-ui';
        const text = fitText(measure, row.text, maxTextWidth);
        const lines = [text];
        const lineWidths = lines.map((line) => measure.measureText(line).width);
        return {
          ...row,
          lines,
          textWidth: lines.length ? Math.min(Math.max(...lineWidths), maxTextWidth) : 0,
          height: Math.max(lines.length * rowLineHeight, rowLineHeight),
        };
      });
      const widthPx = Math.ceil(horizontalPadding * 2 + Math.max(24, ...rowSpecs.map((row) => row.textWidth)));
      const contentHeight = rowSpecs.reduce((sum, row, index) => sum + row.height + (index ? rowGap : 0), 0);
      const heightPx = Math.ceil(contentHeight + 10);
      const canvas = document.createElement('canvas');
      canvas.width = widthPx * dpr;
      canvas.height = heightPx * dpr;
      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, widthPx, heightPx);
      const isProposedMemberLabel = tone === 'proposedMember';
      const proposedInk = proposedMemberColor();
      const ink = inkOverride
        ?? (isProposedMemberLabel
          ? proposedInk
          : kind === 'node'
            ? viewportNodeColor()
            : kind === 'support'
              ? sceneSupportColor()
              : kind === 'load'
                ? sceneLoadColor()
                : (dark ? '#eef2f4' : '#171b20'));
      const background = isProposedMemberLabel
        ? (dark ? '#1f2024' : '#fff9db')
        : (dark ? '#141517' : '#ffffff');
      roundedRect(ctx, 0.5, 0.5, widthPx - 1, heightPx - 1, 11);
      ctx.fillStyle = background;
      ctx.fill();
      ctx.lineWidth = 1;
      ctx.save();
      ctx.globalAlpha = isProposedMemberLabel ? (dark ? 0.9 : 0.74) : (dark ? 0.48 : 0.36);
      ctx.strokeStyle = ink;
      ctx.stroke();
      ctx.restore();
      const textX = widthPx / 2;
      let rowTop = (heightPx - contentHeight) / 2;
      rowSpecs.forEach((row, rowIndex) => {
        ctx.save();
        const color = row.color ?? ink;
        ctx.fillStyle = color;
        const rowCenterY = rowTop + row.height / 2;
        ctx.font = '800 12px Inter, ui-sans-serif, system-ui';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        const firstLineY = rowCenterY - (row.lines.length - 1) * rowLineHeight / 2;
        row.lines.forEach((line, lineIndex) => {
          ctx.fillText(line, textX, firstLineY + lineIndex * rowLineHeight);
        });
        ctx.restore();
        rowTop += row.height + (rowIndex < rowSpecs.length - 1 ? rowGap : 0);
      });
      const texture = new THREE.CanvasTexture(canvas);
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.needsUpdate = true;
      return { texture, widthPx, heightPx };
    }

    function entityLabelTextureStates(kind: EntityLabelKind, primaryText: string, rows: EntityLabelRow[] = [], tone: EntityLabelTone = 'default') {
      const base = entityLabelTexture(kind, primaryText, rows, tone);
      return {
        texture: base.texture,
        widthPx: base.widthPx,
        heightPx: base.heightPx,
        stateTextures: {
          base: base.texture,
          hover: base.texture,
          selected: base.texture,
        },
      };
    }

    function expandedEntityLabelTexture(
      kind: EntityLabelKind,
      lines: ExpandedEntityLabelLine[],
      tone: EntityLabelTone = 'default',
      inkOverride?: string,
    ) {
      const dark = isDarkMode();
      const dpr = 2;
      const measure = document.createElement('canvas').getContext('2d')!;
      const font = '800 12px Inter, ui-sans-serif, system-ui';
      const horizontalPadding = 12;
      const verticalPadding = 9;
      const lineHeight = 15;
      const labelLines = lines
        .map((line) => typeof line === 'string' ? { text: line.trim() } : { ...line, text: line.text.trim() })
        .filter((line) => Boolean(line.text));
      measure.font = font;
      const widthPx = Math.ceil(horizontalPadding * 2 + Math.max(56, ...labelLines.map((line) => measure.measureText(line.text).width)));
      const heightPx = Math.ceil(verticalPadding * 2 + Math.max(1, labelLines.length) * lineHeight);
      const canvas = document.createElement('canvas');
      canvas.width = widthPx * dpr;
      canvas.height = heightPx * dpr;
      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, widthPx, heightPx);
      const isProposedMemberLabel = tone === 'proposedMember';
      const proposedInk = proposedMemberColor();
      const ink = inkOverride
        ?? (isProposedMemberLabel
          ? proposedInk
          : kind === 'node'
            ? viewportNodeColor()
            : kind === 'support'
              ? sceneSupportColor()
              : kind === 'load'
                ? sceneLoadColor()
                : (dark ? '#eef2f4' : '#171b20'));
      const background = isProposedMemberLabel
        ? (dark ? '#1f2024' : '#fff9db')
        : (dark ? '#141517' : '#ffffff');
      roundedRect(ctx, 0.5, 0.5, widthPx - 1, heightPx - 1, 11);
      ctx.fillStyle = background;
      ctx.fill();
      ctx.lineWidth = 1;
      ctx.save();
      ctx.globalAlpha = isProposedMemberLabel ? (dark ? 0.9 : 0.74) : (dark ? 0.55 : 0.4);
      ctx.strokeStyle = ink;
      ctx.stroke();
      ctx.restore();
      ctx.font = font;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = ink;
      labelLines.forEach((line, index) => {
        ctx.fillStyle = line.color ?? ink;
        ctx.fillText(line.text, widthPx / 2, verticalPadding + lineHeight / 2 + index * lineHeight);
      });
      const texture = new THREE.CanvasTexture(canvas);
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.needsUpdate = true;
      return { texture, widthPx, heightPx };
    }

    function expandedEntityLabelTextureStates(kind: EntityLabelKind, lines: ExpandedEntityLabelLine[], tone: EntityLabelTone = 'default') {
      const base = expandedEntityLabelTexture(kind, lines, tone);
      return {
        texture: base.texture,
        widthPx: base.widthPx,
        heightPx: base.heightPx,
        stateTextures: {
          base: base.texture,
          hover: base.texture,
          selected: base.texture,
        },
      };
    }

    function makePreviewIndicatorTexture(text: string, inkOverride?: string) {
      return entityLabelTexture('member', text, [], 'default', inkOverride);
    }

    function makeCoordinateLabelTexture(point: THREE.Vector3) {
      const text = [point.x, point.y, point.z].map(formatLengthScalar).join(', ');
      return makePreviewIndicatorTexture(text);
    }

    function coordinateText(point: THREE.Vector3) {
      return [point.x, point.y, point.z].map(formatLengthScalar).join(', ');
    }

    function compactMemberLabel(member: DisplayMember) {
      return `M${member.id}`;
    }

    function compactMemberDetail(member: DisplayMember) {
      const length = Number.isFinite(member.length) && member.length > 0
        ? formatLengthScalar(member.length)
        : 'n/a';
      return `L${length}`;
    }

    function compactNodeLabel(index: number) {
      return `N${index + 1}`;
    }

    function compactSupportLabel(support: RenderSupport, index: number) {
      return `S${supportDisplayId(support, index)}`;
    }

    function loadAbbreviation(load: RenderLoad) {
      return load.kind === 'uniform_line' ? 'UDL' : 'PL';
    }

    function compactLoadLabel(load: RenderLoad, index: number) {
      return `${loadAbbreviation(load)}${index + 1}`;
    }

    function makeMemberLabelTexture(member: DisplayMember, rows: EntityLabelRow[] = []) {
      const lengthRows = [{ text: compactMemberDetail(member), variant: 'detail' as const }];
      const selfWeightRows = memberHasSelfWeight(member)
        ? [{ text: 'SW', color: sceneLoadColor(), variant: 'detail' as const }]
        : [];
      return entityLabelTextureStates('member', compactMemberLabel(member), [...lengthRows, ...selfWeightRows, ...rows], isSchemePreviewMember(member) ? 'proposedMember' : 'default');
    }

    function makeMemberDimensionLabelTexture(member: DisplayMember) {
      return makeMemberLabelTexture(member);
    }

    function makeExpandedMemberLabelTexture(member: DisplayMember) {
      const length = Number.isFinite(member.length) && member.length > 0
        ? formatLengthScalar(member.length)
        : 'n/a';
      const tone = isSchemePreviewMember(member) ? 'proposedMember' : 'default';
      const lines: ExpandedEntityLabelLine[] = [
        `Member: ${member.id}`,
        `Length: ${length}`,
      ];
      if (memberHasSelfWeight(member)) lines.push({ text: 'Self weight', color: sceneLoadColor() });
      return expandedEntityLabelTextureStates('member', lines, tone);
    }

    function memberHasSelfWeight(member: DisplayMember) {
      const memberIds = new Set([member.id, ...member.segments.map((segment) => segment.memberId)]);
      return (scene.loads ?? []).some((load) => isSelfWeightLoad(load) && memberIds.has(loadMemberId(load) ?? ''));
    }

    function makeMemberIdLabelTexture(memberId: string, anchor: THREE.Vector3, includeCoordinates = false) {
      const member = displayMembers.find((item) => item.id === memberId || item.segments.some((segment) => segment.memberId === memberId));
      const rows = includeCoordinates ? [{ text: coordinateText(anchor), variant: 'detail' as const }] : [];
      if (member) return makeMemberLabelTexture(member, rows);
      return entityLabelTextureStates('member', memberId, [{ text: 'n/a', variant: 'detail' as const }, ...rows]);
    }

    function pendingNodeLabelIds() {
      const ids = scene.nodes.map((node) => node.id);
      const addPendingNode = (label: ViewportEditOverlay['memberStartLabel'] | ViewportEditOverlay['memberEndLabel'] | undefined) => {
        if (label?.kind !== 'node' || ids.includes(label.id)) return;
        ids.push(label.id);
      };
      addPendingNode(currentEditOverlay?.memberStartLabel);
      addPendingNode(currentEditOverlay?.memberEndLabel);
      return ids;
    }

    function previewNodeLabelIndex(nodeId: string) {
      const nodeIndex = scene.nodes.findIndex((item) => item.id === nodeId);
      if (nodeIndex >= 0) return nodeIndex;
      const pendingIndex = pendingNodeLabelIds().indexOf(nodeId);
      return pendingIndex >= 0 ? pendingIndex : scene.nodes.length;
    }

    function shouldHidePermanentNodeEditLabel(nodeId: string) {
      return Boolean(currentLabelVisibility.node && scene.nodes.some((node) => node.id === nodeId));
    }

    function makeNodeIdLabelTexture(nodeId: string, anchor: THREE.Vector3) {
      const nodeIndex = scene.nodes.findIndex((item) => item.id === nodeId);
      const node = nodeIndex >= 0 ? scene.nodes[nodeIndex] : { id: nodeId, x: anchor.x, y: anchor.y, z: anchor.z };
      return makeNodeLabelTexture(node, previewNodeLabelIndex(nodeId));
    }

    function nodeLabelPreviewOffset(textureSpec: { widthPx: number; heightPx: number }, anchorClearancePx: number) {
      return { x: 0, y: -(textureSpec.heightPx / 2 + anchorClearancePx + 5) };
    }

    function addEditNodeLabel(anchor: THREE.Vector3, nodeId: string) {
      const textureSpec = makeNodeIdLabelTexture(nodeId, anchor);
      const anchorClearancePx = NODE_POINT_SIZE_PX * 0.5;
      return addEditPrimitiveLabel(
        anchor,
        textureSpec,
        nodeLabelPreviewOffset(textureSpec, anchorClearancePx),
        anchorClearancePx,
      );
    }

    function formatLengthScalar(value: number) {
      const unit = unitProfile.length;
      const displayValue = value * unit.canonicalToDisplay;
      const precision = unit.precision;
      if (!Number.isFinite(displayValue)) return 'n/a';
      const text = displayValue.toFixed(precision);
      const trimmed = precision <= 0 ? text.replace(/^-0$/, '0') : text.replace(/\.?0+$/, '').replace(/^-0$/, '0');
      const [integer, decimal] = trimmed.split('.');
      const sign = integer.startsWith('-') ? '-' : '';
      const unsigned = sign ? integer.slice(1) : integer;
      const grouped = unsigned.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
      return `${sign}${grouped}${decimal ? `.${decimal}` : ''}`;
    }

    function nodeHasProposedSupport(nodeId: string | undefined) {
      return Boolean(nodeId && proposedSupportNodeIds.has(nodeId));
    }

    function makeNodeLabelTexture(node: { id?: string; x: number; y: number; z: number }, index: number, rows: EntityLabelRow[] = []) {
      const coordinateRows = [{
        text: `X${formatLengthScalar(node.x)}  Y${formatLengthScalar(node.y)}  Z${formatLengthScalar(node.z)}`,
        variant: 'detail' as const,
      }];
      const supportRows = nodeHasProposedSupport(node.id)
        ? [{ text: 'PS', color: sceneSupportColor(), variant: 'detail' as const }]
        : [];
      return entityLabelTextureStates('node', compactNodeLabel(index), [...coordinateRows, ...supportRows, ...rows]);
    }

    function makeExpandedNodeLabelTexture(node: { id?: string; x: number; y: number; z: number }, index: number) {
      const nodeId = String(index + 1);
      const xValue = formatLengthScalar(node.x);
      const yValue = formatLengthScalar(node.y);
      const zValue = formatLengthScalar(node.z);
      const lines: ExpandedEntityLabelLine[] = [
        `Node: ${nodeId}`,
        `X: ${xValue}  Y: ${yValue}  Z: ${zValue}`,
      ];
      if (nodeHasProposedSupport(node.id)) lines.push({ text: 'Proposed support', color: sceneSupportColor() });
      return expandedEntityLabelTextureStates('node', lines);
    }

    function nodeDisplayNumber(nodeId: string | undefined) {
      if (!nodeId) return 'n/a';
      const index = scene.nodes.findIndex((node) => node.id === nodeId);
      return index >= 0 ? String(index + 1) : nodeId;
    }

    function memberDisplayNumber(memberId: string | undefined) {
      if (!memberId) return 'n/a';
      const member = displayMembers.find((item) => (
        item.id === memberId || item.segments.some((segment) => segment.memberId === memberId)
      ));
      return member?.id ?? memberId;
    }

    function makeSupportLabelTexture(support: RenderSupport, index: number) {
      const detail = isBriefVisualSupport(support) ? 'Proposed' : supportType(support);
      return entityLabelTextureStates('support', compactSupportLabel(support, index), [{ text: detail, variant: 'detail' }]);
    }

    function makeExpandedSupportLabelTexture(support: RenderSupport, index: number) {
      const nodeId = supportNodeId(support);
      const lines = [
        `Support: ${supportDisplayId(support, index)}`,
        `Type: ${supportType(support)}`,
        `Node: ${nodeDisplayNumber(nodeId)}`,
      ];
      const group = supportGroupLabel(support);
      if (group && !isBriefVisualSupport(support)) lines.push(`Group: ${group}`);
      return expandedEntityLabelTextureStates('support', lines);
    }

    function loadValueText(load: RenderLoad) {
      const magnitude = typeof load.magnitude === 'number' ? load.magnitude : load.magnitude?.value;
      if (!Number.isFinite(magnitude)) {
        return load.kind === 'uniform_line' ? 'Line load' : 'Point load';
      }
      const kind = load.kind === 'uniform_line' ? 'line' : 'point';
      const quantityKind = kind === 'line' ? 'line_load' : 'force';
      return formatQuantity(magnitude ?? 0, quantityKind, unitProfile);
    }

    function loadDisplayText(load: RenderLoad) {
      const value = loadValueText(load);
      return isSelfWeightLoad(load) ? 'Self weight' : value;
    }

    function makeLoadLabelTexture(load: RenderLoad, index: number) {
      return entityLabelTextureStates('load', compactLoadLabel(load, index), [{ text: loadDisplayText(load), variant: 'detail' }]);
    }

    function loadTargetText(load: RenderLoad) {
      const memberId = loadMemberId(load);
      if (memberId) return `Member ${memberDisplayNumber(memberId)}`;
      const nodeId = loadNodeId(load);
      if (nodeId) return `Node ${nodeDisplayNumber(nodeId)}`;
      if (Number.isFinite(load.x) && Number.isFinite(load.y)) {
        return [load.x ?? 0, load.y ?? 0, load.z ?? 0].map(formatLengthScalar).join(',');
      }
      return 'n/a';
    }

    function makeExpandedLoadLabelTexture(load: RenderLoad, index: number) {
      return expandedEntityLabelTextureStates('load', [
        `${loadAbbreviation(load)} ${index + 1}`,
        `Value: ${loadDisplayText(load)}`,
        `Target: ${loadTargetText(load)}`,
      ]);
    }

    function fitText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number) {
      if (ctx.measureText(text).width <= maxWidth) return text;
      let next = text;
      while (next.length > 4 && ctx.measureText(`${next}...`).width > maxWidth) {
        next = next.slice(0, -1);
      }
      return `${next}...`;
    }

    function createMemberLabel(member: DisplayMember, points: THREE.Vector3[], ownerTargets: AgentTarget[] = []) {
      const themeTextureFactory = () => makeMemberDimensionLabelTexture(member);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      const anchor = memberLabelPoint(points);
      sprite.position.copy(anchor);
      sprite.renderOrder = 90;
      const anchorClearancePx = 0;
      return { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: { x: 0, y: 0 }, priority: 3, kind: 'member' as const, ownerTargets, member, placement: 'pinned' as const, themeTextureFactory };
    }

    function createMemberDetailLabel(member: DisplayMember, points: THREE.Vector3[], ownerTargets: AgentTarget[] = []) {
      const themeTextureFactory = () => makeExpandedMemberLabelTexture(member);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      const anchor = memberLabelPoint(points);
      sprite.position.copy(anchor);
      sprite.renderOrder = 94;
      const anchorClearancePx = 0;
      return { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: { x: 0, y: 0 }, priority: 0, kind: 'member' as const, ownerTargets, member, hoverOnly: true, themeTextureFactory };
    }

    function labelPrimitiveRadius(widthPx: number, heightPx: number, anchorClearancePx = 0) {
      return Math.max(widthPx, heightPx) * 0.5 + anchorClearancePx + labelAnchorGapPx;
    }

    function labelPrimitiveOffset(widthPx: number, heightPx: number, angleRadians: number, anchorClearancePx = 0) {
      const radius = labelPrimitiveRadius(widthPx, heightPx, anchorClearancePx);
      return {
        x: Math.cos(angleRadians) * radius,
        y: Math.sin(angleRadians) * radius,
      };
    }

    function createNodeLabel(node: { id: string; x: number; y: number; z: number }, index: number, focused: boolean, rows: EntityLabelRow[] = [], ownerTargets: AgentTarget[] = []) {
      const themeTextureFactory = () => makeNodeLabelTexture(node, index, rows);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      const anchor = new THREE.Vector3(node.x, node.y, node.z);
      sprite.position.copy(anchor);
      sprite.renderOrder = focused ? 93 : 88;
      const anchorClearancePx = NODE_POINT_SIZE_PX * 0.5;
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: labelPrimitiveOffset(widthPx, heightPx, -Math.PI / 2, anchorClearancePx), priority: focused ? 0 : 1, kind: 'node' as const, ownerTargets, themeTextureFactory };
      nodeLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function createNodeDetailLabel(node: { id: string; x: number; y: number; z: number }, index: number, ownerTargets: AgentTarget[] = []) {
      const themeTextureFactory = () => makeExpandedNodeLabelTexture(node, index);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      const anchor = new THREE.Vector3(node.x, node.y, node.z);
      sprite.position.copy(anchor);
      sprite.renderOrder = 94;
      const anchorClearancePx = NODE_POINT_SIZE_PX * 0.5;
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: labelPrimitiveOffset(widthPx, heightPx, -Math.PI / 2, anchorClearancePx), priority: 0, kind: 'node' as const, ownerTargets, hoverOnly: true, themeTextureFactory };
      nodeLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function createSupportLabel(support: RenderSupport, index: number, anchor: THREE.Vector3, focused: boolean) {
      const themeTextureFactory = () => makeSupportLabelTexture(support, index);
      const themeOffsetFactory = (textureSpec: LabelTextureSpec) => supportLabelOffset(supportType(support), textureSpec.heightPx);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.position.copy(anchor);
      sprite.renderOrder = focused ? 94 : 91;
      const anchorClearancePx = 12;
      const supportTarget = { kind: 'support' as const, id: support.id };
      const ownerTargets = isBriefVisualSupport(support) ? [] : [supportTarget];
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: themeOffsetFactory({ texture, widthPx, heightPx, stateTextures }), priority: focused ? 0 : 2, kind: 'support' as const, ownerTargets, hoverTargets: ownerTargets, placement: 'anchored' as const, themeTextureFactory, themeOffsetFactory };
      supportLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function createSupportDetailLabel(support: RenderSupport, index: number, anchor: THREE.Vector3) {
      const themeTextureFactory = () => makeExpandedSupportLabelTexture(support, index);
      const themeOffsetFactory = (textureSpec: LabelTextureSpec) => supportLabelOffset(supportType(support), textureSpec.heightPx);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.position.copy(anchor);
      sprite.renderOrder = 95;
      const anchorClearancePx = 12;
      const supportTarget = { kind: 'support' as const, id: support.id };
      const ownerTargets = isBriefVisualSupport(support) ? [] : [supportTarget];
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: themeOffsetFactory({ texture, widthPx, heightPx, stateTextures }), priority: 0, kind: 'support' as const, ownerTargets, hoverTargets: ownerTargets, placement: 'anchored' as const, hoverOnly: true, themeTextureFactory, themeOffsetFactory };
      supportLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function createLoadLabel(
      load: RenderLoad,
      index: number,
      anchor: THREE.Vector3,
      loadTrack?: LoadLabelTrack,
    ) {
      const themeTextureFactory = () => makeLoadLabelTexture(load, index);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.position.copy(anchor);
      sprite.renderOrder = 92;
      const anchorClearancePx = 0;
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: labelPrimitiveOffset(widthPx, heightPx, -Math.PI / 4, anchorClearancePx), priority: 4, kind: 'load' as const, ownerTargets: [
        { kind: 'load', id: load.id },
      ], placement: loadTrack ? 'load-line' as const : 'load-point' as const, loadTrack, themeTextureFactory };
      loadLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function createLoadDetailLabel(
      load: RenderLoad,
      index: number,
      anchor: THREE.Vector3,
      loadTrack?: LoadLabelTrack,
    ) {
      const themeTextureFactory = () => makeExpandedLoadLabelTexture(load, index);
      const { texture, widthPx, heightPx, stateTextures } = themeTextureFactory();
      const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      sprite.position.copy(anchor);
      sprite.renderOrder = 95;
      const anchorClearancePx = 0;
      const label = { sprite, material, texture, stateTextures, widthPx, heightPx, anchor, anchorClearancePx, offset: labelPrimitiveOffset(widthPx, heightPx, -Math.PI / 4, anchorClearancePx), priority: 4, kind: 'load' as const, ownerTargets: [
        { kind: 'load', id: load.id },
      ], placement: loadTrack ? 'load-line' as const : 'load-point' as const, loadTrack, hoverOnly: true, themeTextureFactory };
      loadLabelSprites.push(label);
      s.add(sprite);
      return label;
    }

    function pointFromOverlay(point: { x: number; y: number; z: number }) {
      return new THREE.Vector3(point.x, point.y, point.z);
    }

    function updateLineSegments(line: LineSegments2, positions: number[]) {
      line.geometry.dispose();
      const geo = new LineSegmentsGeometry();
      geo.setPositions(positions);
      line.geometry = geo;
      line.computeLineDistances();
      line.visible = positions.length > 0;
    }

    function previewVisualSegments(segments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }>) {
      return segments.map((segment) => {
        return { start: segment.start.clone(), end: segment.end.clone() };
      });
    }

    function editPreviewLineColor() {
      const line = currentEditOverlay?.previewLine;
      if (!line) return viewportInteractionPalette().hoverAccent;
      if (line.tone === 'member') {
        return line.axis === 'x' ? 0xef4444
          : line.axis === 'y' ? 0x22c55e
            : line.axis === 'z' ? 0x3b82f6
              : viewportInteractionPalette().hoverAccent;
      }
      return line.tone === 'move' ? 0x22c55e
        : line.tone === 'split' ? 0xf59e0b
          : viewportMemberColor();
    }

    function previewSegmentPositions(segments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }>) {
      const positions: number[] = [];
      segments.forEach((segment) => {
        positions.push(segment.start.x, segment.start.y, segment.start.z, segment.end.x, segment.end.y, segment.end.z);
      });
      return positions;
    }

    function splitMaskSegments(segments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }>) {
      const masks: Array<{ start: THREE.Vector3; end: THREE.Vector3 }> = [];
      const endpointCounts = new Map<string, number>();
      const keyFor = (point: THREE.Vector3) => `${point.x.toFixed(5)}:${point.y.toFixed(5)}:${point.z.toFixed(5)}`;
      segments.forEach((segment) => {
        [segment.start, segment.end].forEach((point) => {
          const key = keyFor(point);
          endpointCounts.set(key, (endpointCounts.get(key) ?? 0) + 1);
        });
      });
      const seen = new Set<string>();
      segments.forEach((segment) => {
        const length = segment.start.distanceTo(segment.end);
        if (length <= 1e-6) return;
        const direction = segment.end.clone().sub(segment.start).normalize();
        const inset = memberEndDisplayInset(length) * 0.9;
        const atStart = keyFor(segment.start);
        const atEnd = keyFor(segment.end);
        if ((endpointCounts.get(atStart) ?? 0) > 1 && !seen.has(atStart)) {
          seen.add(atStart);
          masks.push({
            start: segment.start.clone().addScaledVector(direction, -inset),
            end: segment.start.clone().addScaledVector(direction, inset),
          });
        }
        if ((endpointCounts.get(atEnd) ?? 0) > 1 && !seen.has(atEnd)) {
          seen.add(atEnd);
          masks.push({
            start: segment.end.clone().addScaledVector(direction, -inset),
            end: segment.end.clone().addScaledVector(direction, inset),
          });
        }
      });
      return masks;
    }

    function updateEditOverlay(overlay: ViewportEditOverlay | null | undefined) {
      currentEditOverlay = overlay;
      clearEditPrimitiveLabels();
      const gridPositions: number[] = [];
      if (overlay?.grid?.visible && overlay.grid.size > 0) {
        const extent = overlay.grid.extent ?? Math.max(20, Math.ceil(viewSize / overlay.grid.size) * overlay.grid.size * 2);
        const step = overlay.grid.size;
        for (let value = -extent; value <= extent + 1e-9; value += step) {
          gridPositions.push(-extent, value, 0, extent, value, 0);
          gridPositions.push(value, -extent, 0, value, extent, 0);
        }
      }
      updateLineSegments(editGridLine, gridPositions);

      if (overlay?.previewLine) {
        const start = pointFromOverlay(overlay.previewLine.start);
        const end = pointFromOverlay(overlay.previewLine.end);
        const segments = overlay.previewMemberSegments?.length
          ? overlay.previewMemberSegments.map((segment) => ({ start: pointFromOverlay(segment.start), end: pointFromOverlay(segment.end) }))
          : [{ start, end }];
        activePreviewVisualSegments = previewVisualSegments(segments);
        const positions = previewSegmentPositions(activePreviewVisualSegments);
        updateLineSegments(editPreviewHaloLine, overlay.previewLine.tone === 'member' ? positions : []);
        updateLineSegments(editPreviewLine, positions);
        if (overlay.previewLine.tone === 'member') {
          const previewNodes = overlay.previewNodes?.length
            ? overlay.previewNodes.map(pointFromOverlay)
            : [end];
          editPreviewNodeGeometry.setAttribute('position', new THREE.Float32BufferAttribute(previewNodes.flatMap((point) => [point.x, point.y, point.z]), 3));
          editPreviewNodeGeometry.attributes.position.needsUpdate = true;
          editPreviewNodeFill.visible = true;
        } else {
          editPreviewNodeFill.visible = false;
        }
        editPreviewMat.color.set(editPreviewLineColor());
      } else {
        updateLineSegments(editPreviewHaloLine, []);
        updateLineSegments(editPreviewLine, []);
        activePreviewVisualSegments = [];
        editPreviewNodeFill.visible = false;
      }

      const splitSegments = overlay?.previewSplitMemberSegments?.map((segment) => ({
        start: pointFromOverlay(segment.start),
        end: pointFromOverlay(segment.end),
      })) ?? [];
      const visualSplitSegments = previewVisualSegments(splitSegments);
      updateLineSegments(editPreviewSplitLine, previewSegmentPositions(visualSplitSegments));
      activePreviewSplitMaskSegments = splitMaskSegments(splitSegments);

      const guidePositions: Record<SnapGuideAxis, number[]> = { x: [], y: [], z: [], angle: [] };
      (overlay?.guideLines ?? []).forEach((guide) => {
        const start = pointFromOverlay(guide.start);
        const end = pointFromOverlay(guide.end);
        guidePositions[guide.axis].push(start.x, start.y, start.z, end.x, end.y, end.z);
      });
      (Object.keys(editGuideLines) as SnapGuideAxis[]).forEach((axis) => {
        updateLineSegments(editGuideHaloLines[axis], axis === 'angle' ? [] : guidePositions[axis]);
        updateLineSegments(editGuideLines[axis], guidePositions[axis]);
      });

      clearEditProjectionGuides();
      if (overlay?.projectionGuide?.guide) {
        const guide = overlay.projectionGuide.guide;
        const start = pointFromOverlay(guide.start);
        const projectedEnd = pointFromOverlay(guide.projectedEnd);
        const realEnd = pointFromOverlay(guide.realEnd);
        editProjectionGuides.push({
          plane: guide.plane,
          start,
          projectedEnd,
          realEnd,
          angles: guide.angles,
          outOfPlaneAxis: guide.outOfPlaneAxis,
          angle: guide.angle,
        });
      }

      if (overlay?.coordinateLabel) {
        const anchor = pointFromOverlay(overlay.coordinateLabel.point);
        if (editCoordinateLabel) {
          s.remove(editCoordinateLabel.sprite);
          editCoordinateLabel.material.dispose();
          editCoordinateLabel.texture.dispose();
        }
        const { texture, widthPx, heightPx } = makeCoordinateLabelTexture(anchor);
        const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
        const sprite = new THREE.Sprite(material);
        sprite.renderOrder = 99;
        sprite.position.copy(anchor);
        editCoordinateLabel = { sprite, material, texture, anchor, widthPx, heightPx };
        s.add(sprite);
      } else if (editCoordinateLabel) {
        s.remove(editCoordinateLabel.sprite);
        editCoordinateLabel.material.dispose();
        editCoordinateLabel.texture.dispose();
        editCoordinateLabel = null;
      }

      if (overlay?.memberStartLabel) {
        const anchor = pointFromOverlay(overlay.memberStartLabel.point);
        if (overlay.memberStartLabel.kind === 'node') {
          if (!shouldHidePermanentNodeEditLabel(overlay.memberStartLabel.id)) {
            addEditNodeLabel(anchor, overlay.memberStartLabel.id);
          }
        } else if (overlay.memberStartLabel.kind === 'member') {
          addEditPrimitiveLabel(anchor, makeMemberIdLabelTexture(overlay.memberStartLabel.id, anchor, false), { x: 0, y: -38 });
        } else {
          addEditPrimitiveLabel(anchor, makeCoordinateLabelTexture(anchor), { x: 0, y: -38 });
        }
      }
      if (overlay?.memberEndLabel) {
        const anchor = pointFromOverlay(overlay.memberEndLabel.point);
        addEditNodeLabel(anchor, overlay.memberEndLabel.id);
      }
      if (overlay?.memberSnapLabel) {
        const anchor = pointFromOverlay(overlay.memberSnapLabel.point);
        addEditPrimitiveLabel(anchor, makeMemberIdLabelTexture(overlay.memberSnapLabel.memberId, anchor, Boolean(overlay.memberSnapLabel.showCoordinates)), { x: 0, y: -40 });
      }
      (overlay?.memberSplitDimensions ?? []).forEach((dimension, index) => {
        const start = pointFromOverlay(dimension.start);
        const end = pointFromOverlay(dimension.end);
        const anchor = start.clone().lerp(end, 0.5);
        const offset = index % 2 === 0 ? { x: 0, y: -30 } : { x: 0, y: 30 };
        addEditPrimitiveLabel(anchor, makePreviewIndicatorTexture(formatLengthScalar(dimension.distance)), offset, 0);
      });

      if (overlay?.snapPoint) {
        const snap = pointFromOverlay(overlay.snapPoint);
        if (editSnapGlyph) {
          s.remove(editSnapGlyph.sprite);
          editSnapGlyph.material.dispose();
          editSnapGlyph.texture.dispose();
        }
        const { texture, widthPx, heightPx } = makeSnapGlyphTexture(overlay.snapPoint.kind);
        const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
        const sprite = new THREE.Sprite(material);
        sprite.renderOrder = 97;
        sprite.position.copy(snap);
        editSnapGlyph = { sprite, material, texture, anchor: snap, widthPx, heightPx };
        s.add(sprite);
      } else {
        if (editSnapGlyph) {
          s.remove(editSnapGlyph.sprite);
          editSnapGlyph.material.dispose();
          editSnapGlyph.texture.dispose();
          editSnapGlyph = null;
        }
      }

      if (overlay?.snapLabel) {
        const anchor = pointFromOverlay(overlay.snapLabel.point);
        if (editSnapLabel) {
          s.remove(editSnapLabel.sprite);
          editSnapLabel.material.dispose();
          editSnapLabel.texture.dispose();
        }
        const { texture, widthPx, heightPx } = makeSnapLabelTexture(overlay.snapLabel.text);
        const material = new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true });
        const sprite = new THREE.Sprite(material);
        sprite.renderOrder = 98;
        sprite.position.copy(anchor);
        editSnapLabel = { sprite, material, texture, anchor, widthPx, heightPx };
        s.add(sprite);
      } else if (editSnapLabel) {
        s.remove(editSnapLabel.sprite);
        editSnapLabel.material.dispose();
        editSnapLabel.texture.dispose();
        editSnapLabel = null;
      }

      clearEditInferenceLabels();
      if (overlay?.inferenceLabel) {
        const anchor = pointFromOverlay(overlay.inferenceLabel.anchor);
        const previewStart = overlay.previewLine ? pointFromOverlay(overlay.previewLine.start) : anchor;
        const previewEnd = overlay.previewLine ? pointFromOverlay(overlay.previewLine.end) : anchor;
        const previewDirection = previewEnd.clone().sub(previewStart);
        const hasPreviewDirection = previewDirection.lengthSq() > 1e-9;
        if (hasPreviewDirection) previewDirection.normalize();
        const inferenceSign = (axis: InferenceAxis) => {
          if (!hasPreviewDirection) return 1;
          return previewDirection.dot(inferenceAxisVector(axis)) >= 0 ? 1 : -1;
        };
        const addAxisCue = (axis: InferenceAxis) => {
          const sign = inferenceSign(axis);
          editInferenceAxisCues.push({ anchor, axis, sign });
          return sign;
        };
        if (overlay.inferenceLabel.kind === 'axis') {
          const sign = addAxisCue(overlay.inferenceLabel.axis);
          addEditInferenceLabel(anchor, overlay.inferenceLabel.label, inferenceAxisColor(overlay.inferenceLabel.axis), inferenceAxisOffset(overlay.inferenceLabel.axis), 15, overlay.inferenceLabel.axis, sign);
        } else {
          overlay.inferenceLabel.entries.forEach((entry) => {
            const sign = addAxisCue(entry.axis);
            addEditInferenceLabel(anchor, `${entry.angleDeg}°`, inferenceAxisColor(entry.axis), inferenceAxisOffset(entry.axis), 15, entry.axis, sign);
          });
        }
      }

      scheduleRender();
    }

    function makeSymbolTexture(spec: ViewportSymbolSpec) {
      const dpr = 3;
      const canvas = document.createElement('canvas');
      canvas.width = spec.widthPx * dpr;
      canvas.height = spec.heightPx * dpr;
      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      ctx.fillStyle = '#000000';
      ctx.fillRect(0, 0, spec.widthPx, spec.heightPx);
      ctx.strokeStyle = '#ffffff';
      ctx.fillStyle = '#ffffff';
      ctx.lineWidth = spec.strokeWidth;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      spec.draw(ctx);
      const texture = new THREE.CanvasTexture(canvas);
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.needsUpdate = true;
      return { texture, widthPx: spec.widthPx, heightPx: spec.heightPx };
    }

    function createSymbolSprite(
      textureSpec: { texture: THREE.CanvasTexture; widthPx: number; heightPx: number },
      anchor: THREE.Vector3,
      tone: 'support' | 'load' | 'release',
      focused: boolean,
      options: { direction?: THREE.Vector3; offset?: { x: number; y: number }; pin?: { x: number; y: number }; proposed?: boolean; target?: AgentTarget } = {},
    ) {
      const material = new THREE.SpriteMaterial({ alphaMap: textureSpec.texture, depthTest: false, depthWrite: false, transparent: true });
      const sprite = new THREE.Sprite(material);
      if (options.pin) {
        sprite.center.set(options.pin.x / textureSpec.widthPx, 1 - options.pin.y / textureSpec.heightPx);
      }
      sprite.position.copy(anchor);
      sprite.renderOrder = tone === 'load' ? (focused ? 49 : 48) : (focused ? 45 : 40);
      const halo = tone === 'support'
        ? (() => {
            const haloMaterial = new THREE.SpriteMaterial({ alphaMap: textureSpec.texture, depthTest: false, depthWrite: false, transparent: true });
            const haloSprite = new THREE.Sprite(haloMaterial);
            haloSprite.position.copy(anchor);
            haloSprite.renderOrder = sprite.renderOrder - 1;
            haloSprite.visible = focused;
            s.add(haloSprite);
            return { sprite: haloSprite, material: haloMaterial };
          })()
        : undefined;
      const item = { sprite, material, texture: textureSpec.texture, widthPx: textureSpec.widthPx, heightPx: textureSpec.heightPx, anchor, tone, focused, halo, ...options };
      symbolSprites.push(item);
      s.add(sprite);
      return item;
    }

    function createSupportSymbol(support: RenderSupport, anchor: THREE.Vector3, focused: boolean) {
      const kind = supportType(support);
      const proposed = isBriefVisualSupport(support);
      return createSymbolSprite(makeSymbolTexture(supportSymbolSpec(kind, supportGroupLabel(support))), anchor, 'support', focused, {
        offset: supportSymbolOffset(kind),
        proposed,
        target: proposed ? undefined : { kind: 'support', id: support.id },
      });
    }

    function arrowHeadWidthAxis(direction: THREE.Vector3, preferredAxis: THREE.Vector3) {
      const dir = direction.clone().normalize();
      const axis = preferredAxis.clone().sub(dir.clone().multiplyScalar(preferredAxis.dot(dir)));
      if (axis.lengthSq() > 1e-8) return axis.normalize();
      const fallbackAxes = [
        new THREE.Vector3(1, 0, 0),
        new THREE.Vector3(0, 1, 0),
        new THREE.Vector3(0, 0, 1),
      ];
      for (const fallback of fallbackAxes) {
        const candidate = fallback.clone().sub(dir.clone().multiplyScalar(fallback.dot(dir)));
        if (candidate.lengthSq() > 1e-8) return candidate.normalize();
      }
      return new THREE.Vector3(1, 0, 0);
    }

    function registerLoadInteractionStroke(loadId: string, line: Line2, halo: Line2 | null, baseRenderOrder: number) {
      loadInteractionStrokes.push({ loadId, line, halo, baseRenderOrder });
    }

    function createLoadLabelLeader(loadId: string) {
      const origin = new THREE.Vector3();
      const railRenderOrder = 16.5;
      // Layer the leader halo below the rail foreground and the leader stroke
      // above it, preserving both red strokes through their shared junction.
      const glyph = createGlyphLine([origin, origin], loadMat, railRenderOrder + 0.5, loadHaloMat);
      glyph.line.visible = false;
      glyph.line.frustumCulled = false;
      if (glyph.haloLine) {
        glyph.haloLine.visible = false;
        glyph.haloLine.frustumCulled = false;
      }
      const leader = { line: glyph.line, halo: glyph.haloLine };
      loadLabelLeaders.push(leader);
      registerLoadInteractionStroke(loadId, glyph.line, glyph.haloLine, railRenderOrder + 0.5);
      return leader;
    }

    function updateLoadLabelLeaderGeometry(leader: LoadLabelLeaderVisual, start: THREE.Vector3, end: THREE.Vector3) {
      const positions = [start, end].flatMap((point) => [point.x, point.y, point.z]);
      (leader.line.geometry as LineGeometry).setPositions(positions);
      leader.line.computeLineDistances();
      leader.line.visible = true;
      if (leader.halo) {
        (leader.halo.geometry as LineGeometry).setPositions(positions);
        leader.halo.computeLineDistances();
        leader.halo.visible = true;
      }
    }

    function createLoadArrow(loadId: string, points: { tail: THREE.Vector3; head: THREE.Vector3; tangent: THREE.Vector3 }, renderOrder: number) {
      const dir = points.head.clone().sub(points.tail).normalize();
      const tangent = arrowHeadWidthAxis(dir, points.tangent);
      const neck = points.head.clone().addScaledVector(dir, -loadArrowSymbol.headBack);
      // Keep the shaft foreground above the open head's halo so it remains visible
      // all the way to the exact tip shared by the two head strokes.
      const shaft = createGlyphLine([points.tail, points.head], loadMat, renderOrder + 0.5, loadHaloMat);
      const left = neck.clone().addScaledVector(tangent, -loadArrowSymbol.headHalfWidth);
      const right = neck.clone().addScaledVector(tangent, loadArrowSymbol.headHalfWidth);
      const head = createGlyphLine(
        [left, points.head, right],
        loadMat,
        renderOrder + 1,
        loadHaloMat,
      );
      const visual: LoadArrowVisual = {
        shaft: shaft.line,
        halo: shaft.haloLine,
        head: head.line,
        headHalo: head.haloLine,
        tail: points.tail.clone(),
        tip: points.head.clone(),
      };
      registerLoadInteractionStroke(loadId, shaft.line, shaft.haloLine, renderOrder + 0.5);
      registerLoadInteractionStroke(loadId, head.line, head.haloLine, renderOrder + 1);
      loadArrowSegments.push({ start: points.tail.clone(), end: points.head.clone(), visual });
      return visual;
    }

    function updateLoadArrowGeometry(
      visual: LoadArrowVisual,
      points: { tail: THREE.Vector3; head: THREE.Vector3; tangent: THREE.Vector3 },
    ) {
      visual.tail.copy(points.tail);
      visual.tip.copy(points.head);
      const shaftPositions = [points.tail, points.head].flatMap((point) => [point.x, point.y, point.z]);
      (visual.shaft.geometry as LineGeometry).setPositions(shaftPositions);
      visual.shaft.computeLineDistances();
      if (visual.halo) {
        (visual.halo.geometry as LineGeometry).setPositions(shaftPositions);
        visual.halo.computeLineDistances();
      }
      updateCameraFacingLoadArrowHead(visual);
    }

    function updateCameraFacingLoadArrowHead(visual: LoadArrowVisual) {
      const tail = projectToViewport(visual.tail);
      const tip = projectToViewport(visual.tip);
      const dx = tip.x - tail.x;
      const dy = tip.y - tail.y;
      const length = Math.hypot(dx, dy);
      if (length <= 1e-6) return;
      const along = { x: dx / length, y: dy / length };
      const across = { x: -along.y, y: along.x };
      const headBackPx = 9 * currentVisualProfile.symbolScale;
      const headHalfWidthPx = 6 * currentVisualProfile.symbolScale;
      const neck = {
        x: tip.x - along.x * headBackPx,
        y: tip.y - along.y * headBackPx,
      };
      const left = screenPointToWorld({
        x: neck.x - across.x * headHalfWidthPx,
        y: neck.y - across.y * headHalfWidthPx,
      }, tip.z);
      const right = screenPointToWorld({
        x: neck.x + across.x * headHalfWidthPx,
        y: neck.y + across.y * headHalfWidthPx,
      }, tip.z);
      const positions = [left, visual.tip, right].flatMap((point) => [point.x, point.y, point.z]);
      (visual.head.geometry as LineGeometry).setPositions(positions);
      visual.head.computeLineDistances();
      if (visual.headHalo) {
        (visual.headHalo.geometry as LineGeometry).setPositions(positions);
        visual.headHalo.computeLineDistances();
      }
    }

    function createPointLoadSymbol(loadId: string, anchor: THREE.Vector3, direction: THREE.Vector3, memberTangent?: THREE.Vector3) {
      const dir = direction.clone().normalize();
      const head = anchor.clone().addScaledVector(dir, -0.16);
      const tail = anchor.clone().addScaledVector(dir, -(loadArrowSymbol.shaftLength + 0.16));
      createLoadArrow(loadId, { tail, head, tangent: memberTangent ?? new THREE.Vector3(0, 1, 0) }, 16.5);
      return tail;
    }

    function createLineLoadSymbol(loadId: string, start: THREE.Vector3, end: THREE.Vector3, direction: THREE.Vector3) {
      const dir = direction.clone().normalize();
      const memberVector = end.clone().sub(start);
      const memberLength = memberVector.length();
      if (memberLength <= 1e-8 || dir.lengthSq() <= 1e-8) return start.clone();
      const tangent = memberVector.clone().normalize();
      const isParallelToMember = Math.abs(tangent.dot(dir)) > 0.985;
      const arrowTangent = arrowHeadWidthAxis(dir, tangent);
      const topOffset = loadArrowSymbol.shaftLength;
      const headOffset = 0.16;
      if (isParallelToMember) {
        const renderOrder = 16.5;
        const arrowLength = Math.min(loadArrowSymbol.shaftLength, Math.max(0.5, memberLength * 0.22), Math.max(0.05, memberLength * 0.75));
        const endMargin = Math.min(0.35, memberLength * 0.08);
        const firstCenter = endMargin + arrowLength / 2;
        const lastCenter = memberLength - endMargin - arrowLength / 2;
        const available = Math.max(0, lastCenter - firstCenter);
        const sign = tangent.dot(dir) >= 0 ? 1 : -1;
        const arrows: LoadArrowVisual[] = [];
        const maximumArrowCount = 8;
        for (let index = 0; index < maximumArrowCount; index += 1) {
          const centerDistance = firstCenter + (available * index) / (maximumArrowCount - 1);
          const tailDistance = centerDistance - sign * arrowLength / 2;
          const headDistance = centerDistance + sign * arrowLength / 2;
          const tail = start.clone().addScaledVector(tangent, tailDistance);
          const head = start.clone().addScaledVector(tangent, headDistance);
          arrows.push(createLoadArrow(loadId, { tail, head, tangent: arrowTangent }, renderOrder));
        }
        loadLineVisualGroups.push({
          start: start.clone(),
          end: end.clone(),
          arrows,
          focused: false,
          layout: {
            start: start.clone(),
            tangent: tangent.clone(),
            arrowTangent: arrowTangent.clone(),
            sign,
            arrowLength,
            firstCenter,
            lastCenter,
          },
        });
        return start.clone().lerp(end, 0.5);
      }
      const rail = createGlyphLine([
        start.clone().addScaledVector(dir, -topOffset),
        end.clone().addScaledVector(dir, -topOffset),
      ], loadMat, 16.5, loadHaloMat);
      registerLoadInteractionStroke(loadId, rail.line, rail.haloLine, 16.5);
      const arrows: LoadArrowVisual[] = [];
      Array.from({ length: 8 }, (_, index) => index / 7).forEach((t) => {
        const base = start.clone().lerp(end, t);
        const top = base.clone().addScaledVector(dir, -topOffset);
        const head = base.clone().addScaledVector(dir, -headOffset);
        const arrow = createLoadArrow(loadId, { tail: top, head, tangent: arrowTangent }, 16.5);
        arrows.push(arrow);
      });
      loadLineVisualGroups.push({
        start: start.clone(),
        end: end.clone(),
        arrows,
        focused: false,
        layout: {
          start: start.clone(),
          end: end.clone(),
          direction: dir.clone(),
          tangent: arrowTangent.clone(),
          tailOffset: topOffset,
        },
      });
      return start.clone().lerp(end, 0.5).addScaledVector(dir, -topOffset);
    }

    function releaseLocalAxes(start: THREE.Vector3, end: THREE.Vector3, atStart: boolean) {
      const memberAxis = end.clone().sub(start);
      if (memberAxis.lengthSq() <= 1e-10) return null;
      const x = memberAxis.normalize().multiplyScalar(atStart ? -1 : 1);
      const preferredNormals = [
        new THREE.Vector3(0, 0, 1),
        new THREE.Vector3(0, 1, 0),
        new THREE.Vector3(1, 0, 0),
      ];
      let z: THREE.Vector3 | null = null;
      for (const normal of preferredNormals) {
        const projected = normal.clone().sub(x.clone().multiplyScalar(normal.dot(x)));
        if (projected.lengthSq() > 1e-8) {
          z = projected.normalize();
          break;
        }
      }
      if (!z) z = new THREE.Vector3(0, 0, 1);
      const y = new THREE.Vector3().crossVectors(z, x).normalize();
      return { x, y, z };
    }

    type ReleaseTick = { axis: keyof typeof RELEASE_TICK_COLORS; direction: THREE.Vector3 };

    function releaseTickDirections(release: RenderRelease, axes: { x: THREE.Vector3; y: THREE.Vector3; z: THREE.Vector3 }): ReleaseTick[] {
      return [
        release.ux ? { axis: 'x' as const, direction: axes.x } : null,
        release.uy ? { axis: 'y' as const, direction: axes.y } : null,
        release.uz ? { axis: 'z' as const, direction: axes.z } : null,
        release.rx ? { axis: 'x' as const, direction: axes.x.clone().negate() } : null,
        release.ry ? { axis: 'y' as const, direction: axes.y.clone().negate() } : null,
        release.rz ? { axis: 'z' as const, direction: axes.z.clone().negate() } : null,
      ].filter((tick): tick is ReleaseTick => Boolean(tick));
    }

    function createReleaseSymbol(release: RenderRelease, start: THREE.Vector3, end: THREE.Vector3, focused: boolean) {
      const atStart = releaseEnd(release) === 'start';
      const memberVector = end.clone().sub(start);
      const memberLength = memberVector.length();
      if (memberLength <= 1e-8) return;
      const axes = releaseLocalAxes(start, end, atStart);
      if (!axes) return;
      const memberDirection = memberVector.normalize();
      const trimInset = memberEndDisplayInset(memberLength);
      const anchor = atStart
        ? start.clone().addScaledVector(memberDirection, trimInset)
        : end.clone().addScaledVector(memberDirection, -trimInset);
      const tickLength = Math.min(0.18, Math.max(0.08, memberLength * 0.025));
      const stemOffset = Math.min(0.04, Math.max(0.015, memberLength * 0.006));
      const ticks = releaseTickDirections(release, axes);
      ticks.forEach(({ axis, direction }) => {
        const material = focused ? focusedReleaseMats[axis] : releaseMats[axis];
        const tickStart = anchor.clone().addScaledVector(direction, stemOffset);
        const tickEnd = anchor.clone().addScaledVector(direction, stemOffset + tickLength);
        createGlyphLine([tickStart, tickEnd], material, focused ? 46 : 41, releaseHaloMat);
      });
    }

    const memberEndpoints = new Map(scene.members.map((member) => [member.id, member]));

    function memberPointsForLoad(load: RenderLoad) {
      const memberId = loadMemberId(load);
      const member = memberId ? memberEndpoints.get(memberId) : undefined;
      const startNode = member ? nodesById.get(memberStartId(member) ?? '') : undefined;
      const endNode = member ? nodesById.get(memberEndId(member) ?? '') : undefined;
      return { memberId, startNode, endNode };
    }

    function loadBasePoints(load: RenderLoad) {
      const targetNodeId = loadNodeId(load);
      const targetNode = targetNodeId ? nodesById.get(targetNodeId) : undefined;
      if (targetNode && load.kind === 'point') {
        return [new THREE.Vector3(targetNode.x, targetNode.y, targetNode.z)];
      }
      const { startNode, endNode } = memberPointsForLoad(load);
      if (startNode && endNode && load.kind === 'uniform_line') {
        const a = new THREE.Vector3(startNode.x, startNode.y, startNode.z);
        const b = new THREE.Vector3(endNode.x, endNode.y, endNode.z);
        return [a.clone().lerp(b, 0.5)];
      }
      if (startNode && endNode) {
        return [new THREE.Vector3((startNode.x + endNode.x) / 2, (startNode.y + endNode.y) / 2, (startNode.z + endNode.z) / 2)];
      }
      return [new THREE.Vector3(load.x ?? 0, load.y ?? 0, load.z ?? 0)];
    }

    function loadMemberPoints(load: RenderLoad) {
      const { startNode, endNode } = memberPointsForLoad(load);
      if (!startNode || !endNode) return undefined;
      return {
        start: new THREE.Vector3(startNode.x, startNode.y, startNode.z),
        end: new THREE.Vector3(endNode.x, endNode.y, endNode.z),
      };
    }

    function hasExplicitLoadPoint(load: RenderLoad) {
      return Number.isFinite(load.x) && Number.isFinite(load.y);
    }

    function hasValidLoadLabelAnchor(load: RenderLoad, memberPoints: ReturnType<typeof loadMemberPoints>) {
      const nodeId = loadNodeId(load);
      return Boolean(memberPoints || (load.kind === 'point' && nodeId && nodesById.has(nodeId)) || hasExplicitLoadPoint(load));
    }

    function nearbyMemberTangentForLoad(point: THREE.Vector3, direction: THREE.Vector3) {
      const dir = direction.clone().normalize();
      let bestScore = Number.POSITIVE_INFINITY;
      let bestTangent: THREE.Vector3 | undefined;
      scene.members.forEach((member) => {
        const start = nodesById.get(memberStartId(member) ?? '');
        const end = nodesById.get(memberEndId(member) ?? '');
        if (!start || !end) return;
        const a = new THREE.Vector3(start.x, start.y, start.z);
        const b = new THREE.Vector3(end.x, end.y, end.z);
        const tangent = b.clone().sub(a);
        if (tangent.lengthSq() <= 1e-9) return;
        tangent.normalize();
        const distance = Math.min(point.distanceTo(a), point.distanceTo(b));
        const alignmentPenalty = Math.abs(tangent.dot(dir));
        const score = distance * 4 + alignmentPenalty;
        if (score < bestScore) {
          bestScore = score;
          bestTangent = tangent;
        }
      });
      return bestTangent;
    }

    const baseMemberPositions: number[] = [];
    const previewMemberPositions: number[] = [];
    const selectedMemberPositions: number[] = [];
    for (const member of scene.members) {
      const startNode = nodesById.get(memberStartId(member) ?? '');
      const endNode = nodesById.get(memberEndId(member) ?? '');
      if (!startNode || !endNode) continue;
      const start = new THREE.Vector3(startNode.x, startNode.y, startNode.z);
      const end = new THREE.Vector3(endNode.x, endNode.y, endNode.z);
      const isPreviewMember = isSchemePreviewRenderMember(member);
      const targetPositions = isPreviewMember ? previewMemberPositions : baseMemberPositions;
      const batchStart = start.clone();
      const batchEnd = end.clone();
      targetPositions.push(batchStart.x, batchStart.y, batchStart.z, batchEnd.x, batchEnd.y, batchEnd.z);
      if (focusedMembers.has(member.id)) selectedMemberPositions.push(batchStart.x, batchStart.y, batchStart.z, batchEnd.x, batchEnd.y, batchEnd.z);
      memberVisualSegments.push({
        memberId: member.id,
        rawStart: start,
        rawEnd: end,
        start: batchStart,
        end: batchEnd,
        preview: isPreviewMember,
      });
      memberHitSegments.push({
        memberId: member.id,
        start,
        end,
        preview: isPreviewMember,
      });
    }
    for (const member of displayMembers) {
      const rawPoints = member.nodeIds.map((id) => nodesById.get(id)).filter(Boolean).map((n) => new THREE.Vector3(n!.x, n!.y, n!.z));
      if (rawPoints.length && labelsEnabled) {
        const ownerTargets = [
          { kind: 'member', id: member.id },
          ...member.segments.map((segment) => ({ kind: 'member', id: segment.memberId })),
        ];
        const dimensionLabel = createMemberLabel(member, rawPoints, ownerTargets);
        const detailLabel = createMemberDetailLabel(member, rawPoints, ownerTargets);
        memberLabelSprites.push(dimensionLabel, detailLabel);
        s.add(dimensionLabel.sprite, detailLabel.sprite);
      }
    }
    previewMemberHaloBatch = createMemberSegmentBatch(previewMemberPositions, previewMemberHaloMat, EDIT_RENDER_ORDER.preview - 1);
    previewMemberBatch = createMemberSegmentBatch(previewMemberPositions, previewMemberBatchMat, EDIT_RENDER_ORDER.preview);
    baseMemberHaloBatch = createMemberSegmentBatch(baseMemberPositions, baseMemberHaloMat, 15);
    baseMemberBatch = createMemberSegmentBatch(baseMemberPositions, memberBatchMat, 16);
    hoverMemberBatch = createMemberSegmentBatch([], hoverMemberMat, 18);
    selectedMemberBatch = createMemberSegmentBatch(selectedMemberPositions, selectedMemberBatchMat, 20);
    selectedMemberBatch.visible = selectedMemberPositions.length > 0;
    if (hoverMemberBatch) hoverMemberBatch.visible = false;

    const baseNodePoints = scene.nodes
      .filter((node) => !isSchemePreviewNode(node) && isViewportNodeSelectable(node.id, proposedSupportNodeIds))
      .map((node) => new THREE.Vector3(node.x, node.y, node.z));
    const previewNodePoints = scene.nodes
      .filter((node) => isSchemePreviewNode(node) && isViewportNodeSelectable(node.id, proposedSupportNodeIds))
      .map((node) => new THREE.Vector3(node.x, node.y, node.z));
    const proposedSupportNodePoints = scene.nodes
      .filter((node) => proposedSupportNodeIds.has(node.id))
      .map((node) => new THREE.Vector3(node.x, node.y, node.z));
    scene.nodes
      .filter((node) => isViewportNodeSelectable(node.id, proposedSupportNodeIds))
      .forEach((node) => nodeHitPoints.push({ nodeId: node.id, point: new THREE.Vector3(node.x, node.y, node.z) }));
    [createNodePoints(baseNodePoints)].forEach((points) => {
      if (!points) return;
      nodeObjects.push(points);
      s.add(points);
    });
    updateNodePointGeometry(createDynamicNodePoints(previewNodeFillMat, EDIT_RENDER_ORDER.schemePreviewNode), previewNodePoints);
    updateNodePointGeometry(createDynamicNodePoints(proposedSupportNodeMat, EDIT_RENDER_ORDER.node), proposedSupportNodePoints);
    const hoverNodeFillPoints = createDynamicNodePoints(hoverNodeFillMat, EDIT_RENDER_ORDER.hoverNode);
    const selectedNodeFillPoints = createDynamicNodePoints(selectedNodeFillMat, EDIT_RENDER_ORDER.selectedNode);

    if (labelsEnabled) {
      scene.nodes.forEach((node, index) => {
        if (isSchemePreviewNode(node) || !isViewportNodeSelectable(node.id, proposedSupportNodeIds)) return;
        const ownerTargets = [
          { kind: 'node', id: node.id },
        ];
        createNodeLabel(node, index, focusedNodes.has(node.id), [], ownerTargets);
        createNodeDetailLabel(node, index, ownerTargets);
      });
    }

    for (const [index, support] of (scene.supports ?? []).entries()) {
      const n = nodesById.get(supportNodeId(support) ?? '');
      if (!n) continue;
      const p = new THREE.Vector3(n.x, n.y, n.z);
      const supportFocused = focusedSupports.has(support.id);
      supportNodeById.set(support.id, supportNodeId(support) ?? '');
      if (!isBriefVisualSupport(support)) {
        supportHitPoints.push({ supportId: support.id, nodeId: supportNodeId(support) ?? '', point: p, kind: supportType(support) });
      }
      createSupportSymbol(support, p, supportFocused);
      if (labelsEnabled && !isBriefVisualSupport(support)) {
        createSupportLabel(support, index, p, supportFocused);
        createSupportDetailLabel(support, index, p);
      }
    }

    for (const [loadIndex, load] of (scene.loads ?? []).entries()) {
      if (isSelfWeightLoad(load)) continue;
      const { memberId } = memberPointsForLoad(load);
      const direction = loadDirection(load);
      const bases = loadBasePoints(load);
      const memberPoints = loadMemberPoints(load);
      const validLabelAnchor = hasValidLoadLabelAnchor(load, memberPoints);
      let labelAnchor: THREE.Vector3 | undefined;
      let loadTrack: LoadLabelTrack | undefined;
      if (load.kind === 'uniform_line' && memberPoints) {
        labelAnchor = createLineLoadSymbol(load.id, memberPoints.start, memberPoints.end, direction);
        const memberTangent = memberPoints.end.clone().sub(memberPoints.start).normalize();
        const visualOffset = Math.abs(memberTangent.dot(direction)) > 0.985
          ? new THREE.Vector3()
          : direction.clone().multiplyScalar(-loadArrowSymbol.shaftLength);
        loadTrack = {
          start: memberPoints.start.clone().add(visualOffset),
          end: memberPoints.end.clone().add(visualOffset),
          direction: direction.clone(),
        };
        loadHitSegments.push({
          loadId: load.id,
          start: memberPoints.start.clone().add(visualOffset),
          end: memberPoints.end.clone().add(visualOffset),
        });
      } else {
        const memberTangent = memberPoints ? memberPoints.end.clone().sub(memberPoints.start) : nearbyMemberTangentForLoad(bases[0], direction);
        bases.forEach((base, baseIndex) => {
          const anchor = createPointLoadSymbol(load.id, base, direction, memberTangent);
          if (baseIndex === 0) labelAnchor = anchor;
          loadHitAnchors.push({ loadId: load.id, point: anchor });
        });
      }
      bases.forEach((base) => loadHitAnchors.push({ loadId: load.id, point: base }));
      if (labelsEnabled && validLabelAnchor) {
        const anchor = labelAnchor ?? bases[0];
        if (loadTrack) loadTrack.leader = createLoadLabelLeader(load.id);
        createLoadLabel(load, loadIndex, anchor, loadTrack);
        createLoadDetailLabel(load, loadIndex, anchor, loadTrack);
      }
    }

    for (const release of scene.releases ?? []) {
      const member = memberEndpoints.get(releaseMemberId(release) ?? '');
      if (!member) continue;
      const startNode = nodesById.get(memberStartId(member) ?? '');
      const endNode = nodesById.get(memberEndId(member) ?? '');
      if (!startNode || !endNode) continue;
      const releaseFocused = focusedMembers.has(member.id);
      createReleaseSymbol(
        release,
        new THREE.Vector3(startNode.x, startNode.y, startNode.z),
        new THREE.Vector3(endNode.x, endNode.y, endNode.z),
        releaseFocused,
      );
    }

    function refreshLabelTexturesForTheme() {
      const labels = [
        ...memberLabelSprites,
        ...nodeLabelSprites,
        ...supportLabelSprites,
        ...loadLabelSprites,
      ];
      labels.forEach((label) => {
        if (!label.themeTextureFactory) return;
        const previousTextures = new Set<THREE.CanvasTexture>([
          label.texture,
          ...Object.values(label.stateTextures ?? {}),
        ]);
        const previousState: LabelVisualState = label.material.map === label.stateTextures?.selected
          ? 'selected'
          : label.material.map === label.stateTextures?.hover
            ? 'hover'
            : 'base';
        const next = label.themeTextureFactory();
        label.texture = next.texture;
        label.stateTextures = next.stateTextures;
        label.widthPx = next.widthPx;
        label.heightPx = next.heightPx;
        if (label.themeOffsetFactory) label.offset = label.themeOffsetFactory(next);
        label.material.map = next.stateTextures?.[previousState] ?? next.texture;
        label.material.needsUpdate = true;
        previousTextures.forEach((texture) => texture.dispose());
      });
    }

    function applyTheme() {
      const bg = viewportBackgroundColor();
      const isDark = isDarkMode();
      const member = viewportMemberColor();
      const interaction = viewportInteractionPalette();
      viewportElement.style.backgroundColor = bg;
      renderer.domElement.style.backgroundColor = bg;
      s.background = new THREE.Color(bg);
      renderer.setClearColor(bg, 1);
      baseMemberHaloMat.color.set(interaction.haloUnderlay);
      memberBatchMat.color.set(member);
      previewMemberHaloMat.color.set(interaction.haloUnderlay);
      previewMemberHaloMat.opacity = 0.92;
      previewMemberBatchMat.color.set(interaction.hoverAccent);
      previewMemberBatchMat.opacity = interaction.hoverOpacity;
      selectedMemberBatchMat.color.set(interaction.selectedAccent);
      selectedMemberBatchMat.opacity = interaction.selectedOpacity;
      hoverMemberMat.color.set(interaction.hoverAccent);
      hoverMemberMat.opacity = interaction.hoverOpacity;
      loadMat.color.set(sceneLoadColor());
      focusedLoadMat.color.set(sceneLoadColor());
      loadHaloMat.color.set(interaction.haloUnderlay);
      focusedLoadHaloMat.color.set(interaction.selectedAccent);
      focusedLoadHaloMat.opacity = interaction.selectedOpacity;
      hoverLoadHaloMat.color.set(interaction.hoverAccent);
      hoverLoadHaloMat.opacity = interaction.hoverOpacity;
      releaseHaloMat.color.set(interaction.haloUnderlay);
      nodeMat.color.set(viewportNodeColor());
      proposedSupportNodeMat.color.set(viewportNodeColor());
      focusedNodeMat.color.set(viewportNodeColor());
      selectedNodeFillMat.color.set(interaction.selectedAccent);
      selectedNodeFillMat.opacity = interaction.selectedOpacity;
      hoverNodeFillMat.color.set(interaction.hoverAccent);
      hoverNodeFillMat.opacity = Math.max(interaction.hoverOpacity, 0.88);
      previewNodeFillMat.color.set(interaction.hoverAccent);
      previewNodeFillMat.opacity = Math.max(interaction.hoverOpacity, 0.88);
      editPreviewHaloMat.color.set(interaction.haloUnderlay);
      editPreviewHaloMat.opacity = 0.92;
      editPreviewMat.color.set(editPreviewLineColor());
      editPreviewMat.opacity = currentEditOverlay?.previewLine?.tone === 'member' ? interaction.hoverOpacity : 1;
      editPreviewSplitMat.color.set(viewportMemberPreviewColor());
      editGuideHaloMat.color.set(interaction.haloUnderlay);
      editInferenceAxisHaloMat.color.set(interaction.haloUnderlay);
      editProjectionAxisHaloMat.color.set(interaction.haloUnderlay);
      editProjectionArrowHaloMat.color.set(interaction.haloUnderlay);
      const supportColor = sceneSupportColor();
      const loadColor = sceneLoadColor();
      symbolSprites.forEach((symbol) => {
        if (symbol.tone === 'support') symbol.material.color.set(supportColor);
        if (symbol.tone === 'load') symbol.material.color.set(loadColor);
        if (symbol.tone === 'release') symbol.material.color.set(isDark ? '#cbd5e1' : '#475569');
        if (symbol.halo) {
          symbol.halo.material.color.set(interaction.selectedAccent);
          symbol.halo.material.opacity = interaction.selectedOpacity;
        }
      });
      updateSymbolInteractionStates();
      refreshViewGizmoIfNeeded();
    }
    applyTheme();
    const themeQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleThemeChange = () => {
      refreshLabelTexturesForTheme();
      updateEditOverlay(currentEditOverlay);
      applyTheme();
      scheduleRender();
    };
    themeQuery.addEventListener('change', handleThemeChange);
    window.addEventListener('fraia:themechange', handleThemeChange);

    const scenePoints = [
      ...scene.nodes.map((n) => new THREE.Vector3(n.x, n.y, n.z)),
      ...(scene.loads ?? []).flatMap((load) => {
        const bases = loadBasePoints(load);
        if (isSelfWeightLoad(load)) return bases;
        return bases.flatMap((base) => [base, base.clone().addScaledVector(loadDirection(load), -1.2)]);
      }),
    ];
    const box = new THREE.Box3().setFromPoints(scenePoints);
    const hasSceneGeometry = !box.isEmpty();
    let sceneCenter = new THREE.Vector3(0, 0, 0);
    let sceneSize = 16;
    let shouldFitScene = true;
    if (hasSceneGeometry) {
      sceneCenter = box.getCenter(new THREE.Vector3());
      const extents = box.getSize(new THREE.Vector3());
      sceneSize = Math.max(extents.x, extents.y, extents.z, 1);
      controls.minZoom = 0.15;
      controls.maxZoom = 12;
    }
    const savedCamera = loadStoredViewportCamera(cameraScopeKey);
    function cameraPreset(view: ViewportCameraView) {
      return view === 'front'
        ? { direction: new THREE.Vector3(0, 0, 1), up: new THREE.Vector3(0, 1, 0) }
        : view === 'top'
          ? { direction: new THREE.Vector3(0, 1, 0), up: new THREE.Vector3(0, 0, 1) }
          : view === 'right'
            ? { direction: new THREE.Vector3(1, 0, 0), up: new THREE.Vector3(0, 1, 0) }
            : { direction: new THREE.Vector3(1, 0.75, 1), up: new THREE.Vector3(0, 1, 0) };
    }
    function applyCameraView(view: ViewportCameraView) {
      const target = box.isEmpty() ? new THREE.Vector3(0, 0, 0) : sceneCenter.clone();
      const distance = Math.max(sceneSize * 2.8, 1);
      const preset = cameraPreset(view);
      camera.up.copy(preset.up);
      camera.position.copy(target.clone().addScaledVector(preset.direction.normalize(), distance));
      camera.zoom = 1;
      if (box.isEmpty()) viewSize = 24;
      camera.lookAt(target);
      controls.target.copy(target);
      controls.cursor.copy(target);
      shouldFitScene = true;
    }
    function saveCameraState() {
      saveStoredViewportCamera(cameraScopeKey, {
        hasSceneGeometry,
        position: [camera.position.x, camera.position.y, camera.position.z],
        target: [controls.target.x, controls.target.y, controls.target.z],
        up: [camera.up.x, camera.up.y, camera.up.z],
        zoom: camera.zoom,
        viewSize,
      });
    }
    applyCameraView('iso');
    const canRestoreSavedCamera = savedCamera && (savedCamera.hasSceneGeometry || !hasSceneGeometry);
    if (canRestoreSavedCamera) {
      camera.position.fromArray(savedCamera.position);
      controls.target.fromArray(savedCamera.target);
      controls.cursor.fromArray(savedCamera.target);
      camera.up.fromArray(savedCamera.up);
      camera.zoom = savedCamera.zoom;
      viewSize = savedCamera.viewSize;
      shouldFitScene = false;
    }
    controls.update();
    let userInteractingWithCamera = false;
    let fitAnimationFrameId = 0;
    function cancelSmoothFitAnimation() {
      if (!fitAnimationFrameId) return;
      cancelAnimationFrame(fitAnimationFrameId);
      fitAnimationFrameId = 0;
    }
    const markCameraInteractionStart = () => {
      cancelSmoothFitAnimation();
      userInteractingWithCamera = true;
      shouldFitScene = false;
    };
    const rememberCamera = () => {
      if (!userInteractingWithCamera) return;
      saveCameraState();
      scheduleRender();
    };
    let activeCameraPointerId: number | null = null;
    let strandChordPan: { pointerId: number; x: number; y: number } | null = null;

    function finishStrandChordPan() {
      if (!strandChordPan) return;
      strandChordPan = null;
      controls.enabled = true;
      activeCameraPointerId = null;
      saveCameraState();
      scheduleRender();
    }

    function handleStrandChordPointerMove(event: PointerEvent) {
      if (!strandChordPan) return;
      const dx = event.clientX - strandChordPan.x;
      const dy = event.clientY - strandChordPan.y;
      strandChordPan.x = event.clientX;
      strandChordPan.y = event.clientY;
      camera.updateMatrixWorld(true);
      const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
      const up = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
      const worldX = (camera.right - camera.left) / Math.max(camera.zoom, 0.001) / Math.max(renderer.domElement.clientWidth, 1);
      const worldY = (camera.top - camera.bottom) / Math.max(camera.zoom, 0.001) / Math.max(renderer.domElement.clientHeight, 1);
      const offset = right.multiplyScalar(-dx * worldX).add(up.multiplyScalar(dy * worldY));
      camera.position.add(offset);
      controls.target.add(offset);
      controls.cursor.add(offset);
      saveCameraState();
      scheduleRender();
    }

    function handleStrandChordPointerUp() {
      finishStrandChordPan();
      document.removeEventListener('pointermove', handleStrandChordPointerMove, true);
      document.removeEventListener('pointerup', handleStrandChordPointerUp, true);
      document.removeEventListener('pointercancel', handleStrandChordPointerUp, true);
    }

    function cancelActiveCameraGesture() {
      if (strandChordPan) {
        handleStrandChordPointerUp();
        return;
      }
      if (activeCameraPointerId === null) return;
      renderer.domElement.dispatchEvent(new PointerEvent('pointercancel', { pointerId: activeCameraPointerId }));
      activeCameraPointerId = null;
    }

    function orbitMouseAction(action: ViewportNavigationAction, event: PointerEvent) {
      if (action === 'zoom') return THREE.MOUSE.DOLLY;
      const modified = event.shiftKey || event.ctrlKey || event.metaKey;
      if (action === 'rotate') return modified ? THREE.MOUSE.PAN : THREE.MOUSE.ROTATE;
      if (action === 'pan') return modified ? THREE.MOUSE.ROTATE : THREE.MOUSE.PAN;
      return -1;
    }

    function configureNavigationGesture(event: PointerEvent) {
      const action = resolveViewportNavigationGesture(currentNavigationProfileId, event, currentCustomNavigationSettings);
      controls.zoomSpeed = viewportZoomSpeedForGesture(action);
      if (action === 'pan' && currentNavigationProfileId === 'strand7' && event.buttons === 3) {
        cancelActiveCameraGesture();
        controls.enabled = false;
        markCameraInteractionStart();
        activeCameraPointerId = event.pointerId;
        strandChordPan = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
        document.addEventListener('pointermove', handleStrandChordPointerMove, true);
        document.addEventListener('pointerup', handleStrandChordPointerUp, true);
        document.addEventListener('pointercancel', handleStrandChordPointerUp, true);
        event.preventDefault();
        event.stopPropagation();
        return;
      }
      const buttonKey = event.button === 0 ? 'LEFT' : event.button === 1 ? 'MIDDLE' : event.button === 2 ? 'RIGHT' : null;
      if (!buttonKey) return;
      controls.mouseButtons[buttonKey] = orbitMouseAction(action, event) as THREE.MOUSE;
      activeCameraPointerId = action === 'none' ? null : event.pointerId;
    }
    const handleViewGizmoStart = () => {
      cancelSmoothFitAnimation();
      userInteractingWithCamera = true;
      shouldFitScene = false;
      if (viewGizmo.animating) startSmoothFitToContent({ followGizmoRotation: true });
    };
    const handleViewGizmoChange = () => {
      controls.update();
      saveCameraState();
      scheduleRender();
    };
    const handleViewGizmoEnd = () => {
      controls.update();
      saveCameraState();
      scheduleRender();
    };
    controls.addEventListener('start', markCameraInteractionStart);
    controls.addEventListener('change', rememberCamera);
    viewGizmo.addEventListener('start', handleViewGizmoStart);
    viewGizmo.addEventListener('change', handleViewGizmoChange);
    viewGizmo.addEventListener('end', handleViewGizmoEnd);

    function labelFitPaddingPx() {
      const labels = [
        ...nodeLabelSprites,
        ...supportLabelSprites,
        ...memberLabelSprites,
        ...loadLabelSprites,
      ];
      const maxHalfWidth = labels.reduce((max, label) => Math.max(max, label.widthPx / 2 + label.anchorClearancePx), 0);
      const maxHalfHeight = labels.reduce((max, label) => Math.max(max, label.heightPx / 2 + label.anchorClearancePx), 0);
      return {
        x: Math.min(260, Math.max(56, maxHalfWidth + labelAnchorGapPx + 18)),
        y: Math.min(180, Math.max(44, maxHalfHeight + labelAnchorGapPx + 18)),
      };
    }

    function fittedViewSize(aspect: number, usableWidthFraction: number, usableHeightFraction: number) {
      if (box.isEmpty()) return viewSize;
      camera.updateMatrixWorld(true);
      const corners = [
        new THREE.Vector3(box.min.x, box.min.y, box.min.z),
        new THREE.Vector3(box.min.x, box.min.y, box.max.z),
        new THREE.Vector3(box.min.x, box.max.y, box.min.z),
        new THREE.Vector3(box.min.x, box.max.y, box.max.z),
        new THREE.Vector3(box.max.x, box.min.y, box.min.z),
        new THREE.Vector3(box.max.x, box.min.y, box.max.z),
        new THREE.Vector3(box.max.x, box.max.y, box.min.z),
        new THREE.Vector3(box.max.x, box.max.y, box.max.z),
      ].map((point) => point.applyMatrix4(camera.matrixWorldInverse));
      const minX = Math.min(...corners.map((point) => point.x));
      const maxX = Math.max(...corners.map((point) => point.x));
      const minY = Math.min(...corners.map((point) => point.y));
      const maxY = Math.max(...corners.map((point) => point.y));
      const projectedWidth = Math.max(maxX - minX, 1);
      const projectedHeight = Math.max(maxY - minY, 1);
      return Math.max(
        projectedHeight / Math.max(usableHeightFraction, 0.1),
        projectedWidth / Math.max(aspect * usableWidthFraction, 0.1)
      ) * 1.16;
    }

    function viewportFitLayout() {
      const r = viewportElement.getBoundingClientRect();
      const width = Math.max(r.width, 1);
      const height = Math.max(r.height, 1);
      const aspect = width / height;
      const labelPadding = labelFitPaddingPx();
      const usableWidth = Math.max(width - currentFitInsetLeft - currentFitInsetRight - labelPadding.x * 2, width * 0.2);
      const usableHeight = Math.max(height - currentFitInsetTop - currentFitInsetBottom - labelPadding.y * 2, height * 0.2);
      return {
        bounds: r,
        width,
        height,
        aspect,
        usableWidthFraction: usableWidth / width,
        usableHeightFraction: usableHeight / height,
      };
    }

    function applyViewportProjection(layout = viewportFitLayout()) {
      const worldWidth = viewSize * layout.aspect;
      const worldHeight = viewSize;
      const frustumXOffset = -((currentFitInsetLeft - currentFitInsetRight) / (2 * layout.width)) * worldWidth;
      const frustumYOffset = ((currentFitInsetTop - currentFitInsetBottom) / (2 * layout.height)) * worldHeight;
      camera.left = -worldWidth / 2 + frustumXOffset;
      camera.right = worldWidth / 2 + frustumXOffset;
      camera.top = worldHeight / 2 + frustumYOffset;
      camera.bottom = -worldHeight / 2 + frustumYOffset;
      camera.updateProjectionMatrix();
    }

    function startSmoothFitToContent({ followGizmoRotation = false } = {}) {
      cancelSmoothFitAnimation();
      const layout = viewportFitLayout();
      const startTarget = controls.target.clone();
      const endTarget = box.isEmpty() ? new THREE.Vector3(0, 0, 0) : sceneCenter.clone();
      const direction = camera.position.clone().sub(controls.target);
      const distance = Math.max(direction.length(), sceneSize * 2.8, 1);
      if (direction.lengthSq() <= 1e-8) direction.set(1, 1, 1);
      direction.normalize();
      const startPosition = camera.position.clone();
      const endPosition = endTarget.clone().addScaledVector(direction, distance);
      const startViewSize = viewSize;
      const startZoom = camera.zoom;
      const initialEndViewSize = box.isEmpty() ? 24 : fittedViewSize(layout.aspect, layout.usableWidthFraction, layout.usableHeightFraction);
      const durationMs = followGizmoRotation ? 220 : 320;
      const startTime = performance.now();
      shouldFitScene = false;

      const animateFit = (now: number) => {
        const rawT = Math.min(1, (now - startTime) / durationMs);
        const easedT = 1 - Math.pow(1 - rawT, 3);
        const previousTarget = controls.target.clone();
        const nextTarget = startTarget.clone().lerp(endTarget, easedT);
        controls.target.copy(nextTarget);
        controls.cursor.copy(controls.target);
        if (followGizmoRotation) {
          camera.position.add(nextTarget.sub(previousTarget));
        } else {
          camera.position.copy(startPosition).lerp(endPosition, easedT);
        }
        camera.zoom = THREE.MathUtils.lerp(startZoom, 1, easedT);
        const desiredViewSize = box.isEmpty() ? 24 : fittedViewSize(layout.aspect, layout.usableWidthFraction, layout.usableHeightFraction);
        viewSize = THREE.MathUtils.lerp(startViewSize, desiredViewSize, easedT);
        if (!followGizmoRotation) camera.lookAt(controls.target);
        applyViewportProjection(layout);
        if (!followGizmoRotation) controls.update();
        viewGizmo.update(false);
        scheduleRender();
        saveCameraState();
        if (rawT < 1) {
          fitAnimationFrameId = requestAnimationFrame(animateFit);
        } else {
          fitAnimationFrameId = 0;
          const endViewSize = box.isEmpty() ? 24 : fittedViewSize(layout.aspect, layout.usableWidthFraction, layout.usableHeightFraction);
          camera.zoom = 1;
          viewSize = Number.isFinite(endViewSize) ? endViewSize : initialEndViewSize;
          const finalDelta = endTarget.clone().sub(controls.target);
          controls.target.copy(endTarget);
          controls.cursor.copy(endTarget);
          if (followGizmoRotation) {
            camera.position.add(finalDelta);
          } else {
            camera.position.copy(endPosition);
            camera.lookAt(endTarget);
          }
          applyViewportProjection(layout);
          if (!followGizmoRotation) controls.update();
          viewGizmo.update(false);
          saveCameraState();
          scheduleRender();
        }
      };
      fitAnimationFrameId = requestAnimationFrame(animateFit);
    }

    let renderFrameId = 0;
    let frameCount = 0;
    let lastMemberViewportSignature = '';
    let rendererWidth = 0;
    let rendererHeight = 0;
    let selectionCanvasWidth = 0;
    let selectionCanvasHeight = 0;
    let resizeRenderLoopId = 0;
    let resizeRenderLoopUntil = 0;

    function uniformLineArrowPoints(layout: UniformLineLoadLayout, t: number, isEndpoint: boolean) {
      const profile = currentVisualProfile;
      const base = layout.start.clone().lerp(layout.end, t);
      const pxPerWorldUnit = projectedDirectionScale(base, layout.direction);
      const loadVisualRadiusPx = (profile.loadStrokePx + profile.haloExtraPx) / 2;
      const obstacleRadiusPx = isEndpoint
        ? NODE_POINT_SIZE_PX * NODE_POINT_RADIUS_RATIO * profile.nodeScale
        : profile.memberStrokePx / 2;
      const headClearanceWorld = (
        obstacleRadiusPx + loadVisualRadiusPx + 2.5
      ) / pxPerWorldUnit * profile.detail;
      return {
        tail: base.clone().addScaledVector(layout.direction, -layout.tailOffset),
        head: base.clone().addScaledVector(layout.direction, -headClearanceWorld),
        tangent: layout.tangent,
      };
    }

    function parallelLineArrowPoints(layout: ParallelLineLoadLayout, t: number) {
      const centerDistance = THREE.MathUtils.lerp(layout.firstCenter, layout.lastCenter, t);
      const tailDistance = centerDistance - layout.sign * layout.arrowLength / 2;
      const headDistance = centerDistance + layout.sign * layout.arrowLength / 2;
      return {
        tail: layout.start.clone().addScaledVector(layout.tangent, tailDistance),
        head: layout.start.clone().addScaledVector(layout.tangent, headDistance),
        tangent: layout.arrowTangent,
      };
    }

    function applyViewportVisualProfile() {
      currentVisualProfile = viewportVisualProfile(camera.zoom);
      const profile = currentVisualProfile;

      memberBatchMat.linewidth = profile.memberStrokePx;
      selectedMemberBatchMat.linewidth = profile.memberStrokePx;
      hoverMemberMat.linewidth = profile.memberStrokePx;
      baseMemberHaloMat.linewidth = profile.memberStrokePx + profile.haloExtraPx;
      baseMemberHaloMat.opacity = 0.96 * profile.detail;

      loadMat.linewidth = profile.loadStrokePx;
      loadMat.opacity = 0.78 + profile.detail * 0.22;
      loadHaloMat.linewidth = profile.loadStrokePx + profile.haloExtraPx;
      loadHaloMat.opacity = 0.94 * profile.detail;
      focusedLoadMat.linewidth = profile.loadStrokePx;
      focusedLoadHaloMat.linewidth = profile.loadStrokePx + Math.min(4, profile.haloExtraPx);
      hoverLoadHaloMat.linewidth = profile.loadStrokePx + Math.min(3, profile.haloExtraPx);

      Object.values(releaseMats).forEach((material) => {
        material.linewidth = viewportStroke.symbol * 0.7 * profile.symbolScale;
      });
      releaseHaloMat.linewidth = viewportStroke.symbol * 0.9 * profile.symbolScale + profile.haloExtraPx;
      releaseHaloMat.opacity = 0.94 * profile.detail;

      nodeMat.size = NODE_POINT_SIZE_PX * profile.nodeScale;
      selectedNodeFillMat.size = (NODE_POINT_SIZE_PX + 4) * profile.nodeScale;
      hoverNodeFillMat.size = (NODE_POINT_SIZE_PX + 2) * profile.nodeScale;
      proposedSupportNodeMat.size = NODE_POINT_SIZE_PX * profile.nodeScale;
      proposedSupportNodeMat.color.set(sceneSupportColor()).lerp(nodeMat.color, profile.detail);
      previewNodeFillMat.size = (NODE_POINT_SIZE_PX + 2) * profile.nodeScale;

      loadArrowSegments.forEach(({ visual }) => updateCameraFacingLoadArrowHead(visual));
      const showLoadArrowHeads = profile.loadArrowDetail === 1;
      loadLineVisualGroups.forEach((group) => {
        const count = Math.min(group.arrows.length, profile.loadArrowCount);
        group.arrows.forEach((arrow, index) => {
          const visible = index < count;
          const layout = group.layout;
          if (visible && layout && 'direction' in layout) {
            const t = count === 1 ? 0.5 : index / (count - 1);
            const isEndpoint = index === 0 || index === count - 1;
            updateLoadArrowGeometry(arrow, uniformLineArrowPoints(layout, t, isEndpoint));
          } else if (visible && layout && 'arrowLength' in layout) {
            const t = count === 1 ? 0.5 : index / (count - 1);
            updateLoadArrowGeometry(arrow, parallelLineArrowPoints(layout, t));
          }
          arrow.shaft.visible = visible;
          if (arrow.halo) arrow.halo.visible = visible;
          arrow.head.visible = visible && showLoadArrowHeads;
          if (arrow.headHalo) arrow.headHalo.visible = visible && showLoadArrowHeads;
        });
      });
    }

    function scheduleRender() {
      if (renderFrameId) return;
      renderFrameId = requestAnimationFrame(renderFrame);
    }

    function renderFrame() {
      renderFrameId = 0;
      frameCount += 1;
      applyViewportVisualProfile();
      updateMemberVisualGeometry();
      updateSymbolSprites();
      updateLabelSprites();
      const r = renderer.domElement.getBoundingClientRect();
      renderer.setViewport(0, 0, rendererWidth || r.width, rendererHeight || r.height);
      renderer.setScissorTest(false);
      renderer.clear(true, true, true);
      const sceneBackground = s.background;
      s.background = null;
      renderer.render(s, camera);
      s.background = sceneBackground;
      viewGizmo.render();
      if (frameCount === 1 || frameCount % 30 === 0) {
        viewportStats.rendererInfo = {
          geometries: renderer.info.memory.geometries,
          textures: renderer.info.memory.textures,
          calls: renderer.info.render.calls,
          triangles: renderer.info.render.triangles,
          points: renderer.info.render.points,
          lines: renderer.info.render.lines,
        };
      }
    }

    function renderImmediately() {
      if (renderFrameId) {
        cancelAnimationFrame(renderFrameId);
        renderFrameId = 0;
      }
      renderFrame();
    }

    function applyBackingStoreSize(width: number, height: number) {
      if (width === rendererWidth && height === rendererHeight) return;
      rendererWidth = width;
      rendererHeight = height;
      renderer.setSize(width, height);
      renderer.domElement.style.width = '100%';
      renderer.domElement.style.height = '100%';
      const pixelRatio = renderer.getPixelRatio();
      const nextSelectionCanvasWidth = Math.max(1, Math.round(width * pixelRatio));
      const nextSelectionCanvasHeight = Math.max(1, Math.round(height * pixelRatio));
      if (nextSelectionCanvasWidth !== selectionCanvasWidth || nextSelectionCanvasHeight !== selectionCanvasHeight) {
        selectionCanvasWidth = nextSelectionCanvasWidth;
        selectionCanvasHeight = nextSelectionCanvasHeight;
        selectionCanvas.width = nextSelectionCanvasWidth;
        selectionCanvas.height = nextSelectionCanvasHeight;
      }
      selectionCtx.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
      selectionCtx.clearRect(0, 0, width, height);
    }

    function startResizeRenderLoop() {
      resizeRenderLoopUntil = performance.now() + 180;
      if (resizeRenderLoopId) return;
      const tick = () => {
        resize();
        if (performance.now() < resizeRenderLoopUntil) {
          resizeRenderLoopId = requestAnimationFrame(tick);
        } else {
          resizeRenderLoopId = 0;
        }
      };
      resizeRenderLoopId = requestAnimationFrame(tick);
    }

    const resize = () => {
      const r = el.getBoundingClientRect();
      const width = Math.max(1, Math.round(r.width));
      const height = Math.max(1, Math.round(r.height));
      if (r.width < 1 || r.height < 1) return;
      if (rendererWidth && rendererHeight && (width !== rendererWidth || height !== rendererHeight)) {
        startResizeRenderLoop();
      }
      applyBackingStoreSize(width, height);
      renderer.setViewport(0, 0, width, height);
      const pixelRatio = renderer.getPixelRatio();
      selectionCtx.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
      selectionCtx.clearRect(0, 0, width, height);
      memberBatchMat.resolution.set(width, height);
      baseMemberHaloMat.resolution.set(width, height);
      previewMemberHaloMat.resolution.set(width, height);
      previewMemberBatchMat.resolution.set(width, height);
      selectedMemberBatchMat.resolution.set(width, height);
      hoverMemberMat.resolution.set(width, height);
      loadMat.resolution.set(width, height);
      focusedLoadMat.resolution.set(width, height);
      loadHaloMat.resolution.set(width, height);
      focusedLoadHaloMat.resolution.set(width, height);
      hoverLoadHaloMat.resolution.set(width, height);
      releaseHaloMat.resolution.set(width, height);
      allReleaseMats.forEach((material) => material.resolution.set(width, height));
      Object.values(editGuideMats).forEach((material) => material.resolution.set(width, height));
      editGuideHaloMat.resolution.set(width, height);
      editPreviewHaloMat.resolution.set(width, height);
      editPreviewMat.resolution.set(width, height);
      editPreviewSplitMat.resolution.set(width, height);
      Object.values(editInferenceAxisMats).forEach((material) => material.resolution.set(width, height));
      editInferenceAxisHaloMat.resolution.set(width, height);
      Object.values(editProjectionAxisMats).forEach((material) => material.resolution.set(width, height));
      editProjectionAxisHaloMat.resolution.set(width, height);
      const layout = viewportFitLayout();
      if (shouldFitScene) viewSize = fittedViewSize(layout.aspect, layout.usableWidthFraction, layout.usableHeightFraction);
      applyViewportProjection(layout);
      applyViewportVisualProfile();
      saveCameraState();
      updateMemberVisualGeometry();
      updateSymbolSprites();
      updateLabelSprites();
      viewGizmo.update(false);
      renderImmediately();
    };
    const ro = new ResizeObserver(resize);
    ro.observe(el);

    const pointerDown = { x: 0, y: 0, button: -1 };

    function projectToViewport(point: THREE.Vector3) {
      const bounds = renderer.domElement.getBoundingClientRect();
      const projected = point.clone().project(camera);
      return {
        x: (projected.x * 0.5 + 0.5) * bounds.width,
        y: (-projected.y * 0.5 + 0.5) * bounds.height,
        z: projected.z,
      };
    }

    function memberViewportSignature() {
      const bounds = renderer.domElement.getBoundingClientRect();
      camera.updateMatrixWorld(true);
      return [
        Math.round(bounds.width),
        Math.round(bounds.height),
        currentVisualProfile.memberEndInsetPx.toFixed(4),
        ...camera.matrixWorld.elements.map((value) => value.toFixed(6)),
        ...camera.projectionMatrix.elements.map((value) => value.toFixed(6)),
      ].join(':');
    }

    function visibleMemberSegment(start: THREE.Vector3, end: THREE.Vector3, preview: boolean) {
      if (preview) return { start: start.clone(), end: end.clone() };
      const worldLength = start.distanceTo(end);
      if (worldLength <= 1e-8) return { start: start.clone(), end: end.clone() };
      const screenStart = projectToViewport(start);
      const screenEnd = projectToViewport(end);
      const screenLength = Math.hypot(screenEnd.x - screenStart.x, screenEnd.y - screenStart.y);
      if (!Number.isFinite(screenLength) || screenLength <= 1e-6) return { start: start.clone(), end: end.clone() };
      const trimRatio = Math.min(MEMBER_END_DISPLAY_MAX_TRIM_RATIO, currentVisualProfile.memberEndInsetPx / screenLength);
      return {
        start: start.clone().lerp(end, trimRatio),
        end: start.clone().lerp(end, 1 - trimRatio),
      };
    }

    function positionsForMemberSegments(segments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }>) {
      return segments.flatMap((segment) => [segment.start.x, segment.start.y, segment.start.z, segment.end.x, segment.end.y, segment.end.z]);
    }

    function updateMemberVisualGeometry() {
      const signature = memberViewportSignature();
      if (signature === lastMemberViewportSignature) return;
      lastMemberViewportSignature = signature;

      const baseSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }> = [];
      const previewSegments: Array<{ start: THREE.Vector3; end: THREE.Vector3 }> = [];
      for (const segment of memberVisualSegments) {
        const visible = visibleMemberSegment(segment.rawStart, segment.rawEnd, segment.preview);
        segment.start.copy(visible.start);
        segment.end.copy(visible.end);
        (segment.preview ? previewSegments : baseSegments).push(segment);
      }

      if (baseMemberHaloBatch) updateLineSegments(baseMemberHaloBatch, positionsForMemberSegments(baseSegments));
      if (baseMemberBatch) updateLineSegments(baseMemberBatch, positionsForMemberSegments(baseSegments));
      if (previewMemberHaloBatch) updateLineSegments(previewMemberHaloBatch, positionsForMemberSegments(previewSegments));
      if (previewMemberBatch) updateLineSegments(previewMemberBatch, positionsForMemberSegments(previewSegments));

      const nextFocusedMembers = new Set(currentFocusedTargets.filter((target) => target.kind === 'member').map((target) => target.id));
      updateMemberOverlay(null, selectedMemberBatch, memberPositions(nextFocusedMembers));
      updateHoverTargets(hoveredTarget ? [hoveredTarget] : []);
    }

    function distanceToScreenSegment(point: { x: number; y: number }, start: { x: number; y: number }, end: { x: number; y: number }) {
      const dx = end.x - start.x;
      const dy = end.y - start.y;
      const lengthSquared = dx * dx + dy * dy;
      if (lengthSquared <= 1e-8) return Math.hypot(point.x - start.x, point.y - start.y);
      const t = Math.max(0, Math.min(1, ((point.x - start.x) * dx + (point.y - start.y) * dy) / lengthSquared));
      return Math.hypot(point.x - (start.x + dx * t), point.y - (start.y + dy * t));
    }

    function pointerRay(event: PointerEvent) {
      const bounds = renderer.domElement.getBoundingClientRect();
      const pointer = new THREE.Vector2(
        ((event.clientX - bounds.left) / bounds.width) * 2 - 1,
        -(((event.clientY - bounds.top) / bounds.height) * 2 - 1)
      );
      camera.updateMatrixWorld(true);
      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(pointer, camera);
      return raycaster.ray.clone();
    }

    function pointOnGroundPlane(event: PointerEvent): THREE.Vector3 | null {
      const ray = pointerRay(event);
      const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
      const point = new THREE.Vector3();
      return ray.intersectPlane(plane, point);
    }

    type ScreenPoint = { x: number; y: number };
    type SelectionDragState = {
      pointerId: number | null;
      mode: ViewportSelectionGesture['shape'];
      start: ScreenPoint;
      points: ScreenPoint[];
      active: boolean;
    };
    let selectionDrag: SelectionDragState | null = null;
    let suppressBoxCompletionPointerUpId: number | null = null;
    const selectionDragThresholdPx = 6;

    function screenDistance(a: ScreenPoint, b: ScreenPoint) {
      return Math.hypot(b.x - a.x, b.y - a.y);
    }

    function selectionKind(start: ScreenPoint, end: ScreenPoint): ViewportSelectionGesture['selectionKind'] {
      return viewportSelectionKind(start.x, end.x);
    }

    function selectionBounds(points: ScreenPoint[]) {
      const xs = points.map((point) => point.x);
      const ys = points.map((point) => point.y);
      return {
        left: Math.min(...xs),
        right: Math.max(...xs),
        top: Math.min(...ys),
        bottom: Math.max(...ys),
      };
    }

    function pointInBounds(point: ScreenPoint, bounds: ReturnType<typeof selectionBounds>) {
      return point.x >= bounds.left && point.x <= bounds.right && point.y >= bounds.top && point.y <= bounds.bottom;
    }

    function pointInPolygon(point: ScreenPoint, polygon: ScreenPoint[]) {
      if (polygon.length < 3) return false;
      let inside = false;
      for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
        const a = polygon[i];
        const b = polygon[j];
        const crosses = (a.y > point.y) !== (b.y > point.y)
          && point.x < ((b.x - a.x) * (point.y - a.y)) / ((b.y - a.y) || 1e-9) + a.x;
        if (crosses) inside = !inside;
      }
      return inside;
    }

    function ccw(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint) {
      return (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x);
    }

    function segmentsIntersect(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint, d: ScreenPoint) {
      return ccw(a, c, d) !== ccw(b, c, d) && ccw(a, b, c) !== ccw(a, b, d);
    }

    function segmentIntersectsBounds(start: ScreenPoint, end: ScreenPoint, bounds: ReturnType<typeof selectionBounds>) {
      if (pointInBounds(start, bounds) || pointInBounds(end, bounds)) return true;
      const corners = [
        { x: bounds.left, y: bounds.top },
        { x: bounds.right, y: bounds.top },
        { x: bounds.right, y: bounds.bottom },
        { x: bounds.left, y: bounds.bottom },
      ];
      return corners.some((corner, index) => segmentsIntersect(start, end, corner, corners[(index + 1) % corners.length]));
    }

    function segmentIntersectsPolygon(start: ScreenPoint, end: ScreenPoint, polygon: ScreenPoint[]) {
      if (pointInPolygon(start, polygon) || pointInPolygon(end, polygon)) return true;
      return polygon.some((point, index) => segmentsIntersect(start, end, point, polygon[(index + 1) % polygon.length]));
    }

    function targetKey(target: AgentTarget) {
      return `${target.kind}:${target.id}`;
    }

    function uniqueTargets(targets: AgentTarget[]) {
      return [...new Map(targets.map((target) => [targetKey(target), target])).values()];
    }

    function selectionPathPoints(drag: SelectionDragState) {
      if (drag.mode === 'lasso') return drag.points;
      const bounds = selectionBounds([drag.start, drag.points[drag.points.length - 1]]);
      return [
        { x: bounds.left, y: bounds.top },
        { x: bounds.right, y: bounds.top },
        { x: bounds.right, y: bounds.bottom },
        { x: bounds.left, y: bounds.bottom },
      ];
    }

    function pointSelected(point: ScreenPoint, path: ScreenPoint[], shape: ViewportSelectionGesture['shape']) {
      return shape === 'box' ? pointInBounds(point, selectionBounds(path)) : pointInPolygon(point, path);
    }

    function projectedSupportCenter(support: (typeof supportHitPoints)[number]) {
      const projected = projectToViewport(support.point);
      const active = currentFocusedTargets.some((target) => target.kind === 'support' && target.id === support.supportId)
        || (hoveredTarget?.kind === 'support' && hoveredTarget.id === support.supportId);
      const overviewScale = active ? 1 : currentVisualProfile.symbolScale;
      const hitRegion = supportSymbolHitRegion(support.kind, overviewScale);
      return { ...projected, x: projected.x + hitRegion.x, y: projected.y + hitRegion.y, hitRegion };
    }

    function distanceToSupportMarker(pointer: ScreenPoint, support: (typeof supportHitPoints)[number]) {
      const marker = projectedSupportCenter(support);
      const dx = Math.max(Math.abs(pointer.x - marker.x) - marker.hitRegion.width / 2, 0);
      const dy = Math.max(Math.abs(pointer.y - marker.y) - marker.hitRegion.height / 2, 0);
      return { distance: Math.hypot(dx, dy), z: marker.z };
    }

    function segmentSelected(
      start: ScreenPoint,
      end: ScreenPoint,
      path: ScreenPoint[],
      shape: ViewportSelectionGesture['shape'],
      kind: ViewportSelectionGesture['selectionKind'],
    ) {
      if (kind === 'window') return pointSelected(start, path, shape) && pointSelected(end, path, shape);
      return shape === 'box'
        ? segmentIntersectsBounds(start, end, selectionBounds(path))
        : segmentIntersectsPolygon(start, end, path);
    }

    function selectionTargets(drag: SelectionDragState): AgentTarget[] {
      const end = drag.points[drag.points.length - 1];
      const kind = selectionKind(drag.start, end);
      const shape = drag.mode;
      const path = selectionPathPoints(drag);
      const targets: AgentTarget[] = [];
      for (const node of nodeHitPoints) {
        const projected = projectToViewport(node.point);
        if (projected.z < -1 || projected.z > 1) continue;
        if (pointSelected(projected, path, shape)) targets.push({ kind: 'node', id: node.nodeId });
      }
      for (const support of supportHitPoints) {
        const projected = projectedSupportCenter(support);
        if (projected.z < -1 || projected.z > 1) continue;
        if (pointSelected(projected, path, shape)) targets.push({ kind: 'support', id: support.supportId });
      }
      for (const load of loadHitAnchors) {
        const projected = projectToViewport(load.point);
        if (projected.z < -1 || projected.z > 1) continue;
        if (pointSelected(projected, path, shape)) targets.push({ kind: 'load', id: load.loadId });
      }
      for (const segment of memberHitSegments) {
        const start = projectToViewport(segment.start);
        const endPoint = projectToViewport(segment.end);
        if ((start.z < -1 || start.z > 1) && (endPoint.z < -1 || endPoint.z > 1)) continue;
        if (segmentSelected(start, endPoint, path, shape, kind)) targets.push({ kind: 'member', id: segment.memberId });
      }
      for (const segment of loadHitSegments) {
        const start = projectToViewport(segment.start);
        const endPoint = projectToViewport(segment.end);
        if ((start.z < -1 || start.z > 1) && (endPoint.z < -1 || endPoint.z > 1)) continue;
        if (segmentSelected(start, endPoint, path, shape, kind)) targets.push({ kind: 'load', id: segment.loadId });
      }
      return uniqueTargets(targets);
    }

    function labelTarget(label: LabelSprite): AgentTarget | null {
      const matchingKindTargets = label.ownerTargets.filter((target) => target.kind === label.kind);
      if (label.kind === 'member') {
        return matchingKindTargets.find((target) => memberEndpoints.has(target.id)) ?? matchingKindTargets[0] ?? null;
      }
      return matchingKindTargets[0] ?? label.ownerTargets[0] ?? null;
    }

    function pointInRect(point: ScreenPoint, rect: ReturnType<typeof screenRect>) {
      return point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom;
    }

    function labelTargetAtPointer(pointer: ScreenPoint) {
      const allLabels = [
        ...nodeLabelSprites,
        ...supportLabelSprites,
        ...memberLabelSprites,
        ...loadLabelSprites,
      ];
      const labels = allLabels
        .filter((label) => label.sprite.visible)
        .sort((a, b) => b.sprite.renderOrder - a.sprite.renderOrder || a.priority - b.priority);
      const hitResult = (label: LabelSprite, target: AgentTarget) => ({
        target,
        hoverMemberAnchor: target.kind === 'member' ? label.anchor.clone() : null,
        source: 'label' as const,
      });
      const labelOwnsTarget = (label: LabelSprite, target: AgentTarget) => (
        (label.hoverTargets ?? label.ownerTargets).some((ownerTarget) => ownerTarget.kind === target.kind && ownerTarget.id === target.id)
        || Boolean(labelTarget(label)?.kind === target.kind && labelTarget(label)?.id === target.id)
      );
      const labelScreenCenter = (label: LabelSprite) => {
        const center = projectToViewport(label.sprite.position);
        if (center.z < -1 || center.z > 1) return null;
        return center;
      };
      const pointerHitsLabel = (label: LabelSprite, paddingPx: number) => {
        const center = labelScreenCenter(label);
        if (!center) return false;
        return pointInRect(pointer, screenRect(center, label.widthPx, label.heightPx, paddingPx));
      };
      const pointerHitsLabelBridge = (a: LabelSprite, b: LabelSprite, paddingPx: number) => {
        const aCenter = labelScreenCenter(a);
        const bCenter = labelScreenCenter(b);
        if (!aCenter || !bCenter) return false;
        const rect = {
          left: Math.min(aCenter.x - a.widthPx / 2, bCenter.x - b.widthPx / 2) - paddingPx,
          right: Math.max(aCenter.x + a.widthPx / 2, bCenter.x + b.widthPx / 2) + paddingPx,
          top: Math.min(aCenter.y - a.heightPx / 2, bCenter.y - b.heightPx / 2) - paddingPx,
          bottom: Math.max(aCenter.y + a.heightPx / 2, bCenter.y + b.heightPx / 2) + paddingPx,
        };
        return pointInRect(pointer, rect);
      };
      for (const label of labels) {
        if (!pointerHitsLabel(label, 3)) continue;
        const target = labelTarget(label);
        if (!target) continue;
        return hitResult(label, target);
      }
      if (hoveredTargetSource === 'label' && hoveredTarget) {
        const stickyLabels = allLabels
          .filter((label) => labelOwnsTarget(label, hoveredTarget!))
          .sort((a, b) => Number(b.hoverOnly) - Number(a.hoverOnly) || b.sprite.renderOrder - a.sprite.renderOrder);
        const stickyPadding = hoveredTarget.kind === 'member' ? 44 : 30;
        for (const label of stickyLabels) {
          if (!pointerHitsLabel(label, stickyPadding)) continue;
          return hitResult(label, hoveredTarget);
        }
        const expandedLabel = stickyLabels.find((label) => label.hoverOnly);
        const compactLabel = stickyLabels.find((label) => !label.hoverOnly);
        if (expandedLabel && compactLabel && pointerHitsLabelBridge(expandedLabel, compactLabel, 22)) {
          return hitResult(expandedLabel, hoveredTarget);
        }
      }
      return null;
    }

    function clearSelectionCanvas() {
      const width = selectionCanvas.width / Math.max(renderer.getPixelRatio(), 1);
      const height = selectionCanvas.height / Math.max(renderer.getPixelRatio(), 1);
      selectionCtx.clearRect(0, 0, width, height);
    }

    function drawSelectionDrag(drag: SelectionDragState) {
      const width = selectionCanvas.width / Math.max(renderer.getPixelRatio(), 1);
      const height = selectionCanvas.height / Math.max(renderer.getPixelRatio(), 1);
      selectionCtx.clearRect(0, 0, width, height);
      const end = drag.points[drag.points.length - 1];
      if (!drag.active) {
        return;
      }
      const kind = selectionKind(drag.start, end);
      const shape = drag.mode;
      const path = selectionPathPoints(drag);
      const crossing = kind === 'crossing';
      const stroke = crossing ? '#22c55e' : '#2563eb';
      const fill = crossing ? 'rgba(34, 197, 94, 0.14)' : 'rgba(37, 99, 235, 0.13)';
      selectionCtx.save();
      selectionCtx.lineWidth = 1.5;
      selectionCtx.strokeStyle = stroke;
      selectionCtx.fillStyle = fill;
      selectionCtx.setLineDash(crossing ? [7, 5] : []);
      selectionCtx.beginPath();
      path.forEach((point, index) => {
        if (index === 0) selectionCtx.moveTo(point.x, point.y);
        else selectionCtx.lineTo(point.x, point.y);
      });
      selectionCtx.closePath();
      selectionCtx.fill();
      selectionCtx.stroke();
      selectionCtx.restore();
    }

    function pointerTargets(event: PointerEvent): { target: AgentTarget | null; snapTarget: ViewportPointerInfo['snapTarget']; hoverMemberAnchor: THREE.Vector3 | null; source: 'label' | 'geometry' | null } {
      const bounds = renderer.domElement.getBoundingClientRect();
      const pointer = { x: event.clientX - bounds.left, y: event.clientY - bounds.top };
      camera.updateMatrixWorld(true);
      const labelHit = labelTargetAtPointer(pointer);
      const labelSnapTarget = labelHit?.target.kind === 'node'
        ? { kind: 'node' as const, id: labelHit.target.id }
        : labelHit?.target.kind === 'member'
          ? { kind: 'member' as const, id: labelHit.target.id }
          : null;
      let best: { id: string; distance: number; anchor: THREE.Vector3 } | null = null;
      let bestNode: { id: string; distance: number } | null = null;
      let bestMidpoint: { id: string; distance: number } | null = null;
      let bestSupport: { id: string; distance: number } | null = null;
      let bestLoad: { id: string; distance: number } | null = null;
      for (const node of nodeHitPoints) {
        const projected = projectToViewport(node.point);
        if (projected.z < -1 || projected.z > 1) continue;
        const distance = Math.hypot(pointer.x - projected.x, pointer.y - projected.y);
        if (!bestNode || distance < bestNode.distance) bestNode = { id: node.nodeId, distance };
      }
      for (const support of supportHitPoints) {
        const marker = distanceToSupportMarker(pointer, support);
        if (marker.z < -1 || marker.z > 1) continue;
        const distance = marker.distance;
        if (!bestSupport || distance < bestSupport.distance) bestSupport = { id: support.supportId, distance };
      }
      for (const load of loadHitAnchors) {
        const projected = projectToViewport(load.point);
        if (projected.z < -1 || projected.z > 1) continue;
        const distance = Math.hypot(pointer.x - projected.x, pointer.y - projected.y);
        if (!bestLoad || distance < bestLoad.distance) bestLoad = { id: load.loadId, distance };
      }
      for (const segment of loadHitSegments) {
        const start = projectToViewport(segment.start);
        const end = projectToViewport(segment.end);
        if ((start.z < -1 || start.z > 1) && (end.z < -1 || end.z > 1)) continue;
        const distance = distanceToScreenSegment(pointer, start, end);
        if (!bestLoad || distance < bestLoad.distance) bestLoad = { id: segment.loadId, distance };
      }
      for (const segment of memberHitSegments) {
        const start = projectToViewport(segment.start);
        const end = projectToViewport(segment.end);
        if ((start.z < -1 || start.z > 1) && (end.z < -1 || end.z > 1)) continue;
        const distance = distanceToScreenSegment(pointer, start, end);
        if (!best || distance < best.distance) {
          const dx = end.x - start.x;
          const dy = end.y - start.y;
          const lengthSquared = dx * dx + dy * dy;
          const t = lengthSquared <= 1e-8
            ? 0.5
            : Math.max(0, Math.min(1, ((pointer.x - start.x) * dx + (pointer.y - start.y) * dy) / lengthSquared));
          best = { id: segment.memberId, distance, anchor: segment.start.clone().lerp(segment.end, t) };
        }
        const midpoint = {
          x: (start.x + end.x) / 2,
          y: (start.y + end.y) / 2,
        };
        const midpointDistance = Math.hypot(pointer.x - midpoint.x, pointer.y - midpoint.y);
        if (!bestMidpoint || midpointDistance < bestMidpoint.distance) bestMidpoint = { id: segment.memberId, distance: midpointDistance };
      }
      const target = bestNode && bestNode.distance <= 14
          ? { kind: 'node' as const, id: bestNode.id }
        : best && best.distance <= 14
          ? { kind: 'member' as const, id: best.id }
          : bestSupport && bestSupport.distance <= 6
            ? { kind: 'support' as const, id: bestSupport.id }
            : bestLoad && bestLoad.distance <= 12
              ? { kind: 'load' as const, id: bestLoad.id }
              : null;
      const snapTarget = bestNode && bestNode.distance <= 14
        ? { kind: 'node' as const, id: bestNode.id }
        : bestMidpoint && bestMidpoint.distance <= 14
          ? { kind: 'memberMidpoint' as const, id: bestMidpoint.id }
          : best && best.distance <= 14
            ? { kind: 'member' as const, id: best.id }
            : null;
      const prioritizedTarget = prioritizeViewportPointerTarget(target, labelHit?.target ?? null);
      if (labelHit && prioritizedTarget === labelHit.target) {
        return {
          target: labelHit.target,
          snapTarget: snapTarget ?? labelSnapTarget,
          hoverMemberAnchor: labelHit.hoverMemberAnchor,
          source: labelHit.source,
        };
      }
      return {
        target: prioritizedTarget,
        snapTarget,
        hoverMemberAnchor: prioritizedTarget?.kind === 'member' ? best?.anchor ?? null : null,
        source: prioritizedTarget ? 'geometry' : null,
      };
    }

    function pointerInfo(event: PointerEvent): ViewportPointerInfo {
      const bounds = renderer.domElement.getBoundingClientRect();
      const ray = pointerRay(event);
      const point = pointOnGroundPlane(event);
      const targets = pointerTargets(event);
      return {
        target: targets.target,
        snapTarget: targets.snapTarget,
        point: point ? { x: point.x, y: point.y, z: point.z } : null,
        hoverPoint: targets.hoverMemberAnchor ? { x: targets.hoverMemberAnchor.x, y: targets.hoverMemberAnchor.y, z: targets.hoverMemberAnchor.z } : null,
        targetSource: targets.source ?? undefined,
        ray: {
          origin: { x: ray.origin.x, y: ray.origin.y, z: ray.origin.z },
          direction: { x: ray.direction.x, y: ray.direction.y, z: ray.direction.z },
        },
        screen: { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
        shiftKey: event.shiftKey,
      };
    }

    function pickMember(event: PointerEvent) {
      const info = pointerInfo(event);
      onSelectTargetRef.current?.(info.target);
      onViewportClickRef.current?.(info);
    }

    function emitSelectionGesture(drag: SelectionDragState) {
      clearSelectionCanvas();
      const end = drag.points[drag.points.length - 1];
      onSelectionGestureRef.current?.({
        operation: 'toggle',
        selectionKind: selectionKind(drag.start, end),
        shape: drag.mode,
        targets: selectionTargets(drag),
        start: drag.start,
        end,
        points: [...drag.points],
      });
    }

    function updateSelectionPreview(drag: SelectionDragState) {
      updateHoverTargets(drag.active ? selectionTargets(drag) : []);
    }

    function startArmedBoxSelection(start: ScreenPoint) {
      selectionDrag = {
        pointerId: null,
        mode: 'box',
        start,
        points: [start],
        active: false,
      };
      clearSelectionCanvas();
      drawSelectionDrag(selectionDrag);
      updateHoverTargets([]);
    }

    function handlePointerDown(event: PointerEvent) {
      pointerDown.x = event.clientX;
      pointerDown.y = event.clientY;
      pointerDown.button = event.button;
      const info = pointerInfo(event);

      if (selectionDrag?.mode === 'box' && selectionDrag.pointerId === null && event.button === 0) {
        event.preventDefault();
        event.stopPropagation();
        const drag = {
          ...selectionDrag,
          points: [selectionDrag.start, info.screen],
          active: screenDistance(selectionDrag.start, info.screen) > selectionDragThresholdPx,
        };
        selectionDrag = null;
        suppressBoxCompletionPointerUpId = event.pointerId;
        updateSelectionPreview(drag);
        if (drag.active) emitSelectionGesture(drag);
        else clearSelectionCanvas();
        return;
      }

      const startsLasso = selectionEnabledRef.current && event.button === 0 && event.altKey && !info.target;
      selectionDrag = startsLasso
        ? {
            pointerId: event.pointerId,
            mode: 'lasso',
            start: info.screen,
            points: [info.screen],
            active: false,
          }
        : null;
      if (selectionDrag) {
        event.preventDefault();
        event.stopPropagation();
        controls.enabled = false;
        renderer.domElement.setPointerCapture?.(event.pointerId);
        return;
      }
      configureNavigationGesture(event);
    }

    function handlePointerUp(event: PointerEvent) {
      if (activeCameraPointerId === event.pointerId && !strandChordPan) {
        activeCameraPointerId = null;
        controls.zoomSpeed = viewportZoomSpeedForGesture('none');
      }
      if (suppressBoxCompletionPointerUpId === event.pointerId) {
        suppressBoxCompletionPointerUpId = null;
        return;
      }
      if (pointerDown.button !== 0 || event.button !== 0) return;
      const moved = Math.hypot(event.clientX - pointerDown.x, event.clientY - pointerDown.y);
      const drag = selectionDrag;
      if (drag?.pointerId !== null && drag?.pointerId === event.pointerId) {
        selectionDrag = null;
        controls.enabled = true;
        try {
          renderer.domElement.releasePointerCapture?.(event.pointerId);
        } catch {
          // Pointer capture can already be released if the pointer leaves the canvas.
        }
        if (drag.active && moved > selectionDragThresholdPx) emitSelectionGesture(drag);
        else clearSelectionCanvas();
        return;
      }
      if (moved <= 4) {
        const info = pointerInfo(event);
        if (selectionEnabledRef.current && !event.altKey && !info.target) {
          const action = emptyCanvasSelectionAction(currentFocusedTargets.length > 0, event.shiftKey);
          if (action === 'clear') {
            onSelectTargetRef.current?.(null);
            onViewportClickRef.current?.(info);
            return;
          }
          startArmedBoxSelection(info.screen);
          onViewportClickRef.current?.(info);
          return;
        }
        pickMember(event);
      }
    }

    function cancelSelectionDrag(event?: PointerEvent) {
      if (!selectionDrag) return;
      try {
        if (selectionDrag.pointerId !== null) renderer.domElement.releasePointerCapture?.(event?.pointerId ?? selectionDrag.pointerId);
      } catch {
        // Pointer capture may already be gone after cancel/leave.
      }
      selectionDrag = null;
      controls.enabled = true;
      clearSelectionCanvas();
      updateHoverTarget(hoveredTarget, hoveredMemberAnchor, hoveredTargetSource);
    }

    function targetEquals(a: AgentTarget | null, b: AgentTarget | null) {
      return (a?.kind ?? '') === (b?.kind ?? '') && (a?.id ?? '') === (b?.id ?? '');
    }

    function targetIsFocused(target: AgentTarget | null) {
      return Boolean(target && currentFocusedTargets.some((item) => item.kind === target.kind && item.id === target.id));
    }

    function memberPositions(memberIds: Set<string>) {
      const positions: number[] = [];
      for (const segment of memberVisualSegments) {
        if (!memberIds.has(segment.memberId)) continue;
        positions.push(segment.start.x, segment.start.y, segment.start.z, segment.end.x, segment.end.y, segment.end.z);
      }
      return positions;
    }

    function nodeIdsForTargets(targets: AgentTarget[]) {
      return new Set(targets.filter((target) => target.kind === 'node').map((target) => target.id));
    }

    function nodePoints(nodeIds: Set<string>) {
      const points: THREE.Vector3[] = [];
      nodeIds.forEach((nodeId) => {
        const node = nodesById.get(nodeId);
        if (node) points.push(new THREE.Vector3(node.x, node.y, node.z));
      });
      return points;
    }

    function updateMemberOverlay(
      haloBatch: LineSegments2 | null,
      accentBatch: LineSegments2 | null,
      positions: number[],
    ) {
      if (haloBatch) updateLineSegments(haloBatch, positions);
      if (accentBatch) updateLineSegments(accentBatch, positions);
    }

    function updateHoverTargets(targets: AgentTarget[]) {
      const previewTargets = targets.filter((target) => !targetIsFocused(target));
      const hoverMemberIds = new Set(previewTargets.filter((target) => target.kind === 'member').map((target) => target.id));
      updateMemberOverlay(null, hoverMemberBatch, memberPositions(hoverMemberIds));
      const hoverPoints = nodePoints(nodeIdsForTargets(previewTargets));
      updateNodePointGeometry(hoverNodeFillPoints, hoverPoints);
      const interaction = viewportInteractionPalette();
      hoverMemberMat.opacity = interaction.hoverOpacity;
      hoverNodeFillMat.opacity = Math.max(interaction.hoverOpacity, 0.88);
      updateSymbolInteractionStates();
      scheduleRender();
    }

    function updateHoverTarget(next: AgentTarget | null, nextHoverAnchor: THREE.Vector3 | null = null, nextSource: 'label' | 'geometry' | null = null) {
      hoveredTarget = next;
      hoveredMemberAnchor = nextHoverAnchor;
      hoveredTargetSource = next ? nextSource : null;
      updateHoverTargets(next ? [next] : []);
    }

    function handlePointerMove(event: PointerEvent) {
      const info = pointerInfo(event);
      if (selectionDrag) {
        const lastPoint = selectionDrag.points[selectionDrag.points.length - 1];
        if (selectionDrag.mode === 'box') {
          selectionDrag.points = [selectionDrag.start, info.screen];
        } else if (screenDistance(lastPoint, info.screen) >= 2) {
          selectionDrag.points.push(info.screen);
        }
        selectionDrag.active = selectionDrag.active || screenDistance(selectionDrag.start, info.screen) > selectionDragThresholdPx;
        drawSelectionDrag(selectionDrag);
        updateSelectionPreview(selectionDrag);
        onViewportPointerMoveRef.current?.(info);
        return;
      }
      if (!selectionEnabledRef.current) {
        if (hoveredTarget) updateHoverTarget(null);
        onViewportPointerMoveRef.current?.(info);
        return;
      }
      const next = info.target;
      const nextHoverAnchor = next?.kind === 'member' && info.hoverPoint
        ? pointFromOverlay(info.hoverPoint)
        : null;
      const nextSource = info.targetSource ?? null;
      const hoverAnchorMoved = Boolean(
        nextHoverAnchor && (!hoveredMemberAnchor || hoveredMemberAnchor.distanceTo(nextHoverAnchor) > 1e-4)
      );
      if (!targetEquals(hoveredTarget, next) || hoverAnchorMoved || hoveredTargetSource !== nextSource) updateHoverTarget(next, nextHoverAnchor, nextSource);
      onViewportPointerMoveRef.current?.(info);
    }

    function handlePointerLeave() {
      if (selectionDrag) return;
      if (!hoveredTarget) return;
      updateHoverTarget(null);
    }

    function handlePointerCancel(event: PointerEvent) {
      controls.zoomSpeed = viewportZoomSpeedForGesture('none');
      cancelSelectionDrag(event);
      updateHoverTarget(null);
    }

    function handleSelectionModifier(event: KeyboardEvent) {
      if (event.key === 'Escape' && selectionDrag) {
        event.preventDefault();
        event.stopImmediatePropagation();
        cancelSelectionDrag();
      }
    }

    renderer.domElement.addEventListener('pointerdown', handlePointerDown, true);
    renderer.domElement.addEventListener('pointermove', handlePointerMove, true);
    renderer.domElement.addEventListener('pointerup', handlePointerUp, true);
    renderer.domElement.addEventListener('pointerleave', handlePointerLeave);
    renderer.domElement.addEventListener('pointercancel', handlePointerCancel);
    window.addEventListener('keydown', handleSelectionModifier);
    window.addEventListener('keyup', handleSelectionModifier);

    function updateFocusedTargets(nextTargets: AgentTarget[]) {
      currentFocusedTargets = nextTargets;
      const nextFocusedMembers = new Set(nextTargets.filter((target) => target.kind === 'member').map((target) => target.id));
      const positions = memberPositions(nextFocusedMembers);
      if (selectedMemberBatch) updateLineSegments(selectedMemberBatch, positions);
      const selectedNodeIds = nodeIdsForTargets(nextTargets);
      const selectedPoints = nodePoints(selectedNodeIds);
      updateNodePointGeometry(selectedNodeFillPoints, selectedPoints);
      updateSymbolInteractionStates();
      updateHoverTarget(hoveredTarget, hoveredMemberAnchor, hoveredTargetSource);
      scheduleRender();
    }

    function updateSymbolInteractionStates() {
      const interaction = viewportInteractionPalette();
      symbolSprites.forEach((item) => {
        if (!item.target || !item.halo) return;
        const selected = targetIsFocused(item.target);
        const hovered = targetEquals(item.target, hoveredTarget);
        item.focused = selected || hovered;
        item.halo.sprite.visible = selected || hovered;
        item.halo.material.color.set(selected ? interaction.selectedAccent : interaction.hoverAccent);
        item.halo.material.opacity = selected ? interaction.selectedOpacity : interaction.hoverOpacity;
      });
      loadInteractionStrokes.forEach((stroke) => {
        const target = { kind: 'load' as const, id: stroke.loadId };
        const selected = targetIsFocused(target);
        const hovered = targetEquals(target, hoveredTarget);
        stroke.line.material = selected ? focusedLoadMat : loadMat;
        stroke.line.renderOrder = selected || hovered ? 49 : stroke.baseRenderOrder;
        if (stroke.halo) {
          stroke.halo.material = selected ? focusedLoadHaloMat : hovered ? hoverLoadHaloMat : loadHaloMat;
          stroke.halo.renderOrder = stroke.line.renderOrder - 1;
        }
      });
    }

    function updateNavigationProfile(
      nextProfileId: ViewportNavigationProfileId,
      nextCustomSettings: ViewportCustomNavigationSettings,
    ) {
      const profileChanged = nextProfileId !== currentNavigationProfileId;
      const customSettingsChanged = Object.keys(nextCustomSettings).some((button) => (
        nextCustomSettings[button as keyof ViewportCustomNavigationSettings]
        !== currentCustomNavigationSettings[button as keyof ViewportCustomNavigationSettings]
      ));
      if (!profileChanged && !customSettingsChanged) return;
      cancelActiveCameraGesture();
      currentNavigationProfileId = nextProfileId;
      currentCustomNavigationSettings = nextCustomSettings;
    }

    function updateFitInsets(next: Required<ViewportFitInsets>) {
      currentFitInsetLeft = next.left;
      currentFitInsetRight = next.right;
      currentFitInsetTop = next.top;
      currentFitInsetBottom = next.bottom;
      refreshViewGizmoIfNeeded();
      resize();
    }

    function updateLabelVisibility(next: ViewportLabelVisibility) {
      currentLabelVisibility = { ...next };
      resize();
      scheduleRender();
    }

    const viewportStats = {
      renderer: 'three',
      nodeCount: scene.nodes.length,
      memberCount: scene.members.length,
      supportCount: scene.supports?.length ?? 0,
      loadCount: scene.loads?.length ?? 0,
      releaseCount: scene.releases?.length ?? 0,
      displayMemberCount: displayMembers.length,
      memberSegmentCount: memberHitSegments.length,
      memberBatchObjectCount: memberBatchObjects.length,
      memberLineObjectCount: memberObjects.length,
      nodePointCloudCount: nodeObjects.length,
      labelsEnabled,
      labelSpriteCount: memberLabelSprites.length + nodeLabelSprites.length + supportLabelSprites.length + loadLabelSprites.length,
      symbolSpriteCount: symbolSprites.length,
      arrowHeadObjectCount: loadArrowSegments.length,
      threeObjectCount: s.children.length,
      canvasPixelRatio: renderer.getPixelRatio(),
      rendererInfo: {
        geometries: renderer.info.memory.geometries,
        textures: renderer.info.memory.textures,
        calls: renderer.info.render.calls,
        triangles: renderer.info.render.triangles,
        points: renderer.info.render.points,
        lines: renderer.info.render.lines,
      },
    };
    (window as any).__FRAIA_VIEWPORT_STATS__ = viewportStats;
    sceneApiRef.current = { updateFocusedTargets, updateFitInsets, updateEditOverlay, updateLabelVisibility, updateNavigationProfile };
    updateFocusedTargets(focusedTargets);
    updateEditOverlay(editOverlay);
    resize();

    function toScreen(p: THREE.Vector3) {
      const r = renderer.domElement.getBoundingClientRect();
      const v = p.clone().project(camera);
      return { x: (v.x * 0.5 + 0.5) * r.width, y: (-v.y * 0.5 + 0.5) * r.height, z: v.z };
    }

    function screenPointToWorld(point: { x: number; y: number }, projectedZ: number) {
      const r = renderer.domElement.getBoundingClientRect();
      return new THREE.Vector3(
        point.x / Math.max(r.width, 1) * 2 - 1,
        -(point.y / Math.max(r.height, 1)) * 2 + 1,
        projectedZ,
      ).unproject(camera);
    }

    type ScreenBounds = { left: number; right: number; top: number; bottom: number };

    function visibleScreenBounds(width: number, height: number): ScreenBounds {
      const left = Math.min(Math.max(currentFitInsetLeft, 0), width);
      const right = Math.max(left, Math.min(width - currentFitInsetRight, width));
      const top = Math.min(Math.max(currentFitInsetTop, 0), height);
      const bottom = Math.max(top, Math.min(height - currentFitInsetBottom, height));
      return { left, right, top, bottom };
    }

    function pointIsOnScreen(point: { x: number; y: number; z?: number }, bounds: ScreenBounds) {
      const zVisible = typeof point.z !== 'number' || (point.z >= -1 && point.z <= 1);
      return Number.isFinite(point.x) && Number.isFinite(point.y) && zVisible
        && point.x >= bounds.left && point.x <= bounds.right
        && point.y >= bounds.top && point.y <= bounds.bottom;
    }
    function screenRect(center: { x: number; y: number }, width: number, height: number, padding = 4) {
      return {
        left: center.x - width / 2 - padding,
        right: center.x + width / 2 + padding,
        top: center.y - height / 2 - padding,
        bottom: center.y + height / 2 + padding,
      };
    }

    function rectsOverlap(a: ReturnType<typeof screenRect>, b: ReturnType<typeof screenRect>) {
      return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
    }

    function rectOverlapArea(a: ReturnType<typeof screenRect>, b: ReturnType<typeof screenRect>) {
      const width = Math.max(0, Math.min(a.right, b.right) - Math.max(a.left, b.left));
      const height = Math.max(0, Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top));
      return width * height;
    }

    function rectGap(a: ReturnType<typeof screenRect>, b: ReturnType<typeof screenRect>) {
      if (rectsOverlap(a, b)) return 0;
      const dx = Math.max(b.left - a.right, a.left - b.right, 0);
      const dy = Math.max(b.top - a.bottom, a.top - b.bottom, 0);
      return Math.hypot(dx, dy);
    }

    function viewportOverflowArea(rect: ReturnType<typeof screenRect>, bounds: ScreenBounds) {
      const overflowLeft = Math.max(0, bounds.left - rect.left);
      const overflowRight = Math.max(0, rect.right - bounds.right);
      const overflowTop = Math.max(0, bounds.top - rect.top);
      const overflowBottom = Math.max(0, rect.bottom - bounds.bottom);
      return (
        overflowLeft * Math.max(0, rect.bottom - rect.top) +
        overflowRight * Math.max(0, rect.bottom - rect.top) +
        overflowTop * Math.max(0, rect.right - rect.left) +
        overflowBottom * Math.max(0, rect.right - rect.left)
      );
    }

    function clampLabelCenter(center: { x: number; y: number }, label: { widthPx: number; heightPx: number }, bounds: ScreenBounds) {
      const padding = 6;
      const halfWidth = label.widthPx / 2;
      const halfHeight = label.heightPx / 2;
      const minX = bounds.left + halfWidth + padding;
      const maxX = Math.max(minX, bounds.right - halfWidth - padding);
      const minY = bounds.top + halfHeight + padding;
      const maxY = Math.max(minY, bounds.bottom - halfHeight - padding);
      return {
        x: Math.min(maxX, Math.max(minX, center.x)),
        y: Math.min(maxY, Math.max(minY, center.y)),
      };
    }

    function structuralObstacleRects() {
      const rects: ReturnType<typeof screenRect>[] = [];
      scene.nodes.forEach((node) => {
        rects.push(screenRect(toScreen(new THREE.Vector3(node.x, node.y, node.z)), 20, 20, 4));
      });
      scene.members.forEach((member) => {
        const start = nodesById.get(member.start);
        const end = nodesById.get(member.end);
        if (!start || !end) return;
        const a = toScreen(new THREE.Vector3(start.x, start.y, start.z));
        const b = toScreen(new THREE.Vector3(end.x, end.y, end.z));
        const length = Math.hypot(b.x - a.x, b.y - a.y);
        const samples = Math.max(4, Math.ceil(length / 10));
        for (let index = 0; index <= samples; index += 1) {
          const t = index / samples;
          rects.push(screenRect(
            { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t },
            18,
            18,
            4,
          ));
        }
      });
      return rects;
    }

    function modelObstacleRects() {
      const rects = structuralObstacleRects();
      symbolSprites.forEach((item) => {
        const center = toScreen(item.anchor);
        const offset = item.offset ?? { x: 0, y: 0 };
        rects.push(screenRect({ x: center.x + offset.x, y: center.y + offset.y }, item.widthPx, item.heightPx, item.tone === 'load' ? 8 : 6));
      });
      rects.push(...visibleVectorObstacleRects());
      if (editSnapGlyph) {
        rects.push(screenRect(toScreen(editSnapGlyph.anchor), editSnapGlyph.widthPx, editSnapGlyph.heightPx, 8));
      }
      (Object.keys(editProjectionDimensionLabels) as InferenceAxis[]).forEach((axis) => {
        const label = editProjectionDimensionLabels[axis];
        if (!label?.sprite.visible) return;
        rects.push(screenRect(toScreen(label.sprite.position), label.widthPx, label.heightPx, 6));
      });
      return rects;
    }

    function visibleVectorObstacleRects() {
      const rects: ReturnType<typeof screenRect>[] = [];
      loadArrowSegments.forEach((segment) => {
        if (!segment.visual.shaft.visible) return;
        rects.push(...screenSegmentObstacleRects(segment.start, segment.end, loadArrowSymbol.strokeWidth + 14, 4));
      });
      activeProjectionVectorSegments.forEach((segment) => {
        rects.push(...screenSegmentObstacleRects(segment.start, segment.end, PROJECTION_ARROW_SHAFT_WIDTH_PX + 18, 4));
      });
      return rects;
    }

    function screenSegmentObstacleRects(start: THREE.Vector3, end: THREE.Vector3, sizePx = 18, padding = 4) {
      const rects: ReturnType<typeof screenRect>[] = [];
      const a = toScreen(start);
      const b = toScreen(end);
      if (!Number.isFinite(a.x) || !Number.isFinite(a.y) || !Number.isFinite(b.x) || !Number.isFinite(b.y)) return rects;
      const length = Math.hypot(b.x - a.x, b.y - a.y);
      const samples = Math.max(2, Math.ceil(length / 16));
      for (let index = 0; index <= samples; index += 1) {
        const t = index / samples;
        rects.push(screenRect(
          { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t },
          sizePx,
          sizePx,
          padding,
        ));
      }
      return rects;
    }

    function updateLabelSprites() {
      const r = renderer.domElement.getBoundingClientRect();
      const visibleBounds = visibleScreenBounds(r.width, r.height);
      const worldUnitsPerPixel = viewSize / Math.max(r.height, 1) / Math.max(camera.zoom, 0.001);
      camera.updateMatrixWorld(true);
      const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
      const up = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
      const placedLabels: ReturnType<typeof screenRect>[] = [];
      loadLabelLeaders.forEach((leader) => {
        leader.line.visible = false;
        if (leader.halo) leader.halo.visible = false;
      });
      const compactLabelCollisionPaddingPx = 6;
      function compactLabelRect(center: { x: number; y: number }, label: LabelSprite) {
        return screenRect(center, label.widthPx, label.heightPx, compactLabelCollisionPaddingPx);
      }
      function labelOrthogonalCandidates(label: LabelSprite) {
        const nodeTarget = label.kind === 'node'
          ? label.ownerTargets.find((target) => target.kind === 'node')
          : undefined;
        const nodeAnchorRadius = nodeTarget && proposedSupportNodeIds.has(nodeTarget.id)
          ? 10 * SUPPORT_SYMBOL_SCALE
          : NODE_POINT_SIZE_PX / 2;
        const gap = label.kind === 'node'
          ? nodeAnchorRadius + 5
          : labelAnchorGapPx + label.anchorClearancePx;
        const diagonalX = label.widthPx / 2 + gap;
        const diagonalY = label.heightPx / 2 + gap;
        const candidates = [
          { direction: 'below' as const, x: 0, y: label.heightPx / 2 + gap },
          { direction: 'above' as const, x: 0, y: -(label.heightPx / 2 + gap) },
          { direction: 'right' as const, x: label.widthPx / 2 + gap, y: 0 },
          { direction: 'left' as const, x: -(label.widthPx / 2 + gap), y: 0 },
          { direction: 'below-left' as const, x: -diagonalX, y: diagonalY },
          { direction: 'below-right' as const, x: diagonalX, y: diagonalY },
          { direction: 'above-left' as const, x: -diagonalX, y: -diagonalY },
          { direction: 'above-right' as const, x: diagonalX, y: -diagonalY },
        ];
        if (label.kind === 'node') {
          const preferred = ['below', 'left', 'right', 'above'];
          return candidates
            .filter((candidate) => preferred.includes(candidate.direction))
            .map((candidate) => ({
              ...candidate,
              preference: preferred.indexOf(candidate.direction),
            }))
            .sort((a, b) => a.preference - b.preference);
        }
        const preferred = label.kind === 'member'
          ? ['above', 'below', 'right', 'left', 'above-left', 'above-right', 'below-left', 'below-right']
          : ['below', 'above', 'left', 'right', 'below-left', 'below-right', 'above-left', 'above-right'];
        return candidates
          .map((candidate) => ({
            ...candidate,
            preference: preferred.indexOf(candidate.direction),
          }))
          .sort((a, b) => a.preference - b.preference);
      }
      const labels = [
        ...nodeLabelSprites,
        ...supportLabelSprites,
        ...memberLabelSprites,
        ...loadLabelSprites,
      ].sort((a, b) => {
        const hoverOrder = Number(Boolean(a.hoverOnly)) - Number(Boolean(b.hoverOnly));
        if (hoverOrder) return hoverOrder;
        const placementOrder = (label: LabelSprite) => label.placement === 'anchored' ? 0 : 1;
        return placementOrder(a) - placementOrder(b) || a.priority - b.priority;
      });
      const labelTransitionDurationMs = 150;

      function targetMatches(a: AgentTarget, b: AgentTarget | null) {
        return Boolean(b && a.kind === b.kind && a.id === b.id);
      }

      function labelMatchesHover(label: LabelSprite) {
        return (label.hoverTargets ?? label.ownerTargets).some((target) => targetMatches(target, hoveredTarget));
      }

      function labelMatchesFocusedTarget(label: LabelSprite) {
        return label.ownerTargets.some(targetIsFocused);
      }

      function oppositeProjectedDirection(anchor: THREE.Vector3, direction: THREE.Vector3) {
        const origin = toScreen(anchor);
        const tip = toScreen(anchor.clone().add(direction));
        const opposite = { x: origin.x - tip.x, y: origin.y - tip.y };
        const length = Math.hypot(opposite.x, opposite.y);
        return length > 1e-6
          ? { x: opposite.x / length, y: opposite.y / length }
          : undefined;
      }

      const loadLabelLeaderLengthPx = 14;
      const loadLabelLeaderGapPx = 0;

      function loadLabelLeaderGeometry(track: LoadLabelTrack, fraction: number) {
        const start = track.start.clone().lerp(track.end, fraction);
        const startScreen = toScreen(start);
        const away = oppositeProjectedDirection(start, track.direction) ?? { x: 0, y: -1 };
        const endScreen = {
          x: startScreen.x + away.x * loadLabelLeaderLengthPx,
          y: startScreen.y + away.y * loadLabelLeaderLengthPx,
        };
        return {
          away,
          start,
          startScreen,
          end: screenPointToWorld(endScreen, startScreen.z),
          endScreen,
        };
      }

      function loadLabelCalloutCenter(
        geometry: ReturnType<typeof loadLabelLeaderGeometry>,
        label: LabelSprite,
      ) {
        const labelRadius = (
          Math.abs(geometry.away.x) * label.widthPx
          + Math.abs(geometry.away.y) * label.heightPx
        ) / 2;
        return {
          x: geometry.endScreen.x + geometry.away.x * (loadLabelLeaderGapPx + labelRadius),
          y: geometry.endScreen.y + geometry.away.y * (loadLabelLeaderGapPx + labelRadius),
        };
      }

      function showLoadLabelLeader(track: LoadLabelTrack, fraction: number) {
        if (!track.leader) return;
        const geometry = loadLabelLeaderGeometry(track, fraction);
        updateLoadLabelLeaderGeometry(track.leader, geometry.start, geometry.end);
      }

      function setLabelVisualState(label: LabelSprite, state: 'base' | 'hover' | 'selected') {
        const texture = label.stateTextures?.[state] ?? label.texture;
        label.material.opacity = state === 'base' ? currentVisualProfile.baseLabelOpacity : 1;
        if (label.material.map !== texture) {
          label.material.map = texture;
          label.material.needsUpdate = true;
        }
      }

      function labelPrimaryTargetKey(label: LabelSprite) {
        const target = labelTarget(label);
        return target ? targetKey(target) : '';
      }

      function compactCounterpartFor(label: LabelSprite) {
        if (!label.hoverOnly) return undefined;
        const key = labelPrimaryTargetKey(label);
        if (!key) return undefined;
        return labels.find((candidate) => (
          candidate !== label
          && !candidate.hoverOnly
          && candidate.kind === label.kind
          && labelPrimaryTargetKey(candidate) === key
        ));
      }

      function expandedCounterpartFor(label: LabelSprite) {
        if (label.hoverOnly) return undefined;
        const key = labelPrimaryTargetKey(label);
        if (!key) return undefined;
        return labels.find((candidate) => (
          candidate !== label
          && Boolean(candidate.hoverOnly)
          && candidate.kind === label.kind
          && labelPrimaryTargetKey(candidate) === key
        ));
      }

      function easeOutCubic(value: number) {
        const t = Math.min(1, Math.max(0, value));
        return 1 - Math.pow(1 - t, 3);
      }

      function applyLabelPose(label: LabelSprite, targetPosition: THREE.Vector3, targetScale: THREE.Vector3, state: LabelVisualState) {
        const now = performance.now();
        if (!label.wasVisible && label.hoverOnly) {
          const counterpart = compactCounterpartFor(label);
          if (counterpart?.sprite.visible) {
            label.transition = {
              startedAt: now,
              fromPosition: counterpart.sprite.position.clone(),
              fromScale: counterpart.sprite.scale.clone(),
            };
          }
        }
        label.wasVisible = true;
        if (!label.transition) {
          setLabelVisualState(label, state);
          label.sprite.position.copy(targetPosition);
          label.sprite.scale.copy(targetScale);
          return;
        }
        const progress = easeOutCubic((now - label.transition.startedAt) / labelTransitionDurationMs);
        setLabelVisualState(label, state);
        label.sprite.position.copy(label.transition.fromPosition.clone().lerp(targetPosition, progress));
        label.sprite.scale.copy(label.transition.fromScale.clone().lerp(targetScale, progress));
        if (progress >= 1) {
          label.transition = undefined;
          setLabelVisualState(label, state);
        } else {
          scheduleRender();
        }
      }

      function labelMatchesActiveEditMemberLabel(label: LabelSprite) {
        if (label.kind !== 'member') return false;
        const activeMemberIds = [
          currentEditOverlay?.memberSnapLabel?.memberId,
          currentEditOverlay?.memberStartLabel?.kind === 'member' ? currentEditOverlay.memberStartLabel.id : undefined,
        ].filter(Boolean);
        if (!activeMemberIds.length) return false;
        return label.ownerTargets.some((target) => (
          target.kind === 'member' && activeMemberIds.includes(target.id)
        ));
      }

      function labelVisibilityState(label: LabelSprite) {
        const permanentlyVisible = currentLabelVisibility[label.kind];
        const hoverVisible = labelMatchesHover(label);
        const swapCompactForExpanded = !label.hoverOnly && hoverVisible && Boolean(expandedCounterpartFor(label));
        if (swapCompactForExpanded) {
          return { permanentlyVisible, hoverVisible, visible: false };
        }
        if (label.placement === 'pinned') {
          return { permanentlyVisible, hoverVisible: false, visible: permanentlyVisible };
        }
        if (label.hoverOnly) {
          const visible = hoverVisible && !labelMatchesActiveEditMemberLabel(label);
          return { permanentlyVisible: false, hoverVisible: visible, visible };
        }
        return { permanentlyVisible, hoverVisible, visible: permanentlyVisible || hoverVisible };
      }

      function shouldStabilizeLabelPlacement(
        label: LabelSprite,
        visibility: ReturnType<typeof labelVisibilityState>,
      ) {
        return Boolean(label.hoverOnly) && visibility.hoverVisible;
      }

      const temporaryExpansionActive = labels.some((label) => (
        Boolean(label.hoverOnly) && labelMatchesHover(label)
      ));

      labels.forEach((label) => {
        const targetScale = new THREE.Vector3(label.widthPx * worldUnitsPerPixel, label.heightPx * worldUnitsPerPixel, 1);
        const visibility = labelVisibilityState(label);
        const selected = labelMatchesFocusedTarget(label);
        const hovered = labelMatchesHover(label);
        const visualState = hovered ? 'hover' : 'base';
        if (temporaryExpansionActive && !label.hoverOnly && visibility.visible && label.wasVisible) {
          const center = toScreen(label.sprite.position);
          label.sprite.visible = pointIsOnScreen(center, visibleBounds);
          if (!label.sprite.visible) return;
          if (label.placement === 'load-line' && label.loadTrack) {
            showLoadLabelLeader(label.loadTrack, label.loadTrack.labelFraction ?? 0.5);
          }
          placedLabels.push(compactLabelRect(center, label));
          applyLabelPose(label, label.sprite.position, targetScale, visualState);
          return;
        }
        if (!visibility.visible || (!selected && !hovered && currentVisualProfile.baseLabelOpacity <= 0.02)) {
          label.sprite.visible = false;
          label.wasVisible = false;
          label.transition = undefined;
          return;
        }
        const labelAnchor = label.anchor;
        const rawAnchorScreen = toScreen(labelAnchor);
        label.sprite.visible = pointIsOnScreen(rawAnchorScreen, visibleBounds);
        if (!label.sprite.visible) return;
        const anchorScreen = rawAnchorScreen;
        if (label.hoverOnly) {
          const compact = compactCounterpartFor(label);
          if (compact) {
            const compactCenter = toScreen(compact.sprite.position);
            if (label.kind === 'member' && compact.kind === 'member') {
              applyLabelPose(label, screenPointToWorld(compactCenter, compactCenter.z), targetScale, visualState);
              return;
            }
            if (label.kind === 'node' && compact.kind === 'node') {
              const cardinalDirections: LabelPlacementDirection[] = ['below', 'above', 'right', 'left'];
              const compactOffset = {
                x: compactCenter.x - anchorScreen.x,
                y: compactCenter.y - anchorScreen.y,
              };
              const inferredDirection: LabelPlacementDirection = Math.abs(compactOffset.x) > Math.abs(compactOffset.y)
                ? compactOffset.x < 0 ? 'left' : 'right'
                : compactOffset.y < 0 ? 'above' : 'below';
              const expansionDirection = compact.placementDirection && cardinalDirections.includes(compact.placementDirection)
                ? compact.placementDirection
                : inferredDirection;
              const expansionVectors: Record<'below' | 'above' | 'right' | 'left', { x: number; y: number }> = {
                below: { x: 0, y: 1 },
                above: { x: 0, y: -1 },
                right: { x: 1, y: 0 },
                left: { x: -1, y: 0 },
              };
              const center = expandedLabelCenterAlongDirection(
                compactCenter,
                { width: compact.widthPx, height: compact.heightPx },
                { width: label.widthPx, height: label.heightPx },
                expansionVectors[expansionDirection as keyof typeof expansionVectors],
              );
              applyLabelPose(label, screenPointToWorld(center, compactCenter.z), targetScale, visualState);
              return;
            }
            if (label.kind === 'load' && label.placement === 'load-line' && label.loadTrack) {
              const expansionDirection = oppositeProjectedDirection(label.anchor, label.loadTrack.direction);
              if (expansionDirection) {
                const center = expandedLabelCenterAlongDirection(
                  compactCenter,
                  { width: compact.widthPx, height: compact.heightPx },
                  { width: label.widthPx, height: label.heightPx },
                  expansionDirection,
                );
                showLoadLabelLeader(label.loadTrack, label.loadTrack.labelFraction ?? 0.5);
                applyLabelPose(label, screenPointToWorld(center, compactCenter.z), targetScale, visualState);
                return;
              }
            }
            const horizontalGap = (compact.widthPx + label.widthPx) / 2 + 8;
            const verticalGap = (compact.heightPx + label.heightPx) / 2 + 8;
            const candidates = [
              { x: 0, y: 0 },
              { x: horizontalGap, y: 0 },
              { x: -horizontalGap, y: 0 },
              { x: 0, y: verticalGap },
              { x: 0, y: -verticalGap },
              { x: horizontalGap, y: verticalGap },
              { x: -horizontalGap, y: verticalGap },
              { x: horizontalGap, y: -verticalGap },
              { x: -horizontalGap, y: -verticalGap },
            ];
            let bestCenter: { x: number; y: number } = compactCenter;
            let bestScore = Number.POSITIVE_INFINITY;
            candidates.forEach((offset, index) => {
              const unclampedCenter = { x: compactCenter.x + offset.x, y: compactCenter.y + offset.y };
              const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
              const rect = screenRect(center, label.widthPx, label.heightPx);
              const overlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
              const overflowArea = viewportOverflowArea(rect, visibleBounds);
              const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
              const score = overlapArea * 20000 + overflowArea * 50000 + clampDistance * 100 + index;
              if (score < bestScore) {
                bestScore = score;
                bestCenter = center;
              }
            });
            applyLabelPose(label, screenPointToWorld(bestCenter, compactCenter.z), targetScale, visualState);
            return;
          }
        }
        if (label.placement === 'load-line' && label.loadTrack) {
          const trackStart = toScreen(label.loadTrack.start);
          const trackEnd = toScreen(label.loadTrack.end);
          const trackLengthPx = Math.hypot(trackEnd.x - trackStart.x, trackEnd.y - trackStart.y);
          const fractions = viewportLoadLeaderFractions(Math.min(39, Math.max(5, Math.ceil(trackLengthPx / 14))));
          const structuralObstacles = structuralObstacleRects();
          let bestCenter: { x: number; y: number } = anchorScreen;
          let bestRect: ReturnType<typeof screenRect> | undefined;
          let bestFraction = 0.5;
          let bestDepth = trackStart.z;
          let bestScore = Number.POSITIVE_INFINITY;
          let bestBlockingOverlapArea = Number.POSITIVE_INFINITY;
          let bestOverflowArea = Number.POSITIVE_INFINITY;
          fractions.forEach((fraction, fractionIndex) => {
            const geometry = loadLabelLeaderGeometry(label.loadTrack!, fraction);
            const center = loadLabelCalloutCenter(geometry, label);
            const rect = compactLabelRect(center, label);
            const labelOverlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const structuralOverlapArea = structuralObstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const overflowArea = viewportOverflowArea(rect, visibleBounds);
            const score =
              (labelOverlapArea + structuralOverlapArea) * 20000
              + overflowArea * 50000
              + fractionIndex;
            if (score < bestScore) {
              bestScore = score;
              bestBlockingOverlapArea = labelOverlapArea + structuralOverlapArea;
              bestOverflowArea = overflowArea;
              bestCenter = center;
              bestRect = rect;
              bestFraction = fraction;
              bestDepth = geometry.startScreen.z;
            }
          });
          if ((bestBlockingOverlapArea > 0 || bestOverflowArea > 0) && !hovered && !selected) {
            label.sprite.visible = false;
            label.wasVisible = false;
            label.transition = undefined;
            return;
          }
          label.loadTrack.labelFraction = bestFraction;
          showLoadLabelLeader(label.loadTrack, bestFraction);
          placedLabels.push(bestRect ?? compactLabelRect(bestCenter, label));
          applyLabelPose(label, screenPointToWorld(bestCenter, bestDepth), targetScale, visualState);
          return;
        }
        if (label.placement === 'load-point') {
          const candidates = [
            { x: 0, y: 0 },
            ...labelOrthogonalCandidates(label),
          ];
          let bestCenter: { x: number; y: number } = anchorScreen;
          let bestRect: ReturnType<typeof screenRect> | undefined;
          let bestScore = Number.POSITIVE_INFINITY;
          let bestBlockingOverlapArea = Number.POSITIVE_INFINITY;
          const structuralObstacles = structuralObstacleRects();
          candidates.forEach((offset, index) => {
            const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
            const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
            const rect = compactLabelRect(center, label);
            const labelOverlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const structuralOverlapArea = structuralObstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const overflowArea = viewportOverflowArea(rect, visibleBounds);
            const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
            const score = (labelOverlapArea + structuralOverlapArea) * 20000 + overflowArea * 50000 + clampDistance * 100 + index;
            if (score < bestScore) {
              bestScore = score;
              bestBlockingOverlapArea = labelOverlapArea + structuralOverlapArea;
              bestCenter = center;
              bestRect = rect;
            }
          });
          if (bestBlockingOverlapArea > 0 && !hovered && !selected) {
            label.sprite.visible = false;
            label.wasVisible = false;
            label.transition = undefined;
            return;
          }
          placedLabels.push(bestRect ?? compactLabelRect(bestCenter, label));
          applyLabelPose(label, screenPointToWorld(bestCenter, anchorScreen.z), targetScale, visualState);
          return;
        }
        if (label.placement === 'pinned') {
          const rect = compactLabelRect(anchorScreen, label);
          const overlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          if (overlapArea > 0 && !visibility.hoverVisible) {
            label.sprite.visible = false;
            return;
          }
          placedLabels.push(rect);
          applyLabelPose(label, labelAnchor, targetScale, visualState);
          return;
        }
        if (label.placement === 'anchored') {
          let bestCenter: { x: number; y: number } = anchorScreen;
          let bestRect: ReturnType<typeof screenRect> | undefined;
          let bestScore = Number.POSITIVE_INFINITY;
          let bestOverlapArea = Number.POSITIVE_INFINITY;
          supportLabelOffsetCandidates(label.offset, label.widthPx, label.heightPx).forEach((offset, index) => {
            const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
            const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
            const rect = compactLabelRect(center, label);
            const overlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
            const preferredDistance = Math.hypot(offset.x - label.offset.x, offset.y - label.offset.y);
            const score = overlapArea * 20000 + clampDistance * 100 + preferredDistance + index;
            if (score < bestScore) {
              bestScore = score;
              bestOverlapArea = overlapArea;
              bestCenter = center;
              bestRect = rect;
            }
          });
          if (bestOverlapArea > 0 && !hovered && !selected) {
            label.sprite.visible = false;
            label.wasVisible = false;
            label.transition = undefined;
            return;
          }
          placedLabels.push(bestRect ?? compactLabelRect(bestCenter, label));
          const targetPosition = screenPointToWorld(bestCenter, anchorScreen.z);
          applyLabelPose(label, targetPosition, targetScale, visualState);
          return;
        }
        let best = label.offset;
        let bestRect: ReturnType<typeof screenRect> | undefined;
        let bestScore = Number.POSITIVE_INFINITY;
        let bestLabelOverlapArea = Number.POSITIVE_INFINITY;
        let bestDirection: LabelPlacementDirection | undefined;
        let retainedPlacement: { offset: { x: number; y: number }; rect: ReturnType<typeof screenRect>; score: number; labelOverlapArea: number; direction: LabelPlacementDirection } | undefined;
        const stabilizePlacement = shouldStabilizeLabelPlacement(label, visibility);
        const modelObstacles = modelObstacleRects().filter((rect) => !(
          label.kind === 'node'
          && anchorScreen.x >= rect.left
          && anchorScreen.x <= rect.right
          && anchorScreen.y >= rect.top
          && anchorScreen.y <= rect.bottom
        ));
        for (const offset of labelOrthogonalCandidates(label)) {
          const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
          const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
          const effectiveOffset = { ...offset, x: center.x - anchorScreen.x, y: center.y - anchorScreen.y };
          const rect = compactLabelRect(center, label);
          const labelOverlapArea = placedLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          const modelOverlapArea = modelObstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          const overflowArea = viewportOverflowArea(rect, visibleBounds);
          const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
          const overlapPenalty = 10000;
          const overflowPenalty = 50000;
          const score =
            (labelOverlapArea + modelOverlapArea) * overlapPenalty +
            overflowArea * overflowPenalty +
            clampDistance * 2 +
            offset.preference;
          if (stabilizePlacement && offset.direction === label.placementDirection) {
            retainedPlacement = { offset: effectiveOffset, rect, score, labelOverlapArea, direction: offset.direction };
          }
          if (score < bestScore) {
            bestScore = score;
            bestLabelOverlapArea = labelOverlapArea;
            best = effectiveOffset;
            bestRect = rect;
            bestDirection = offset.direction;
          }
        }
        if (
          retainedPlacement &&
          retainedPlacement.direction !== bestDirection &&
          retainedPlacement.score <= bestScore + hoverMemberPlacementHysteresis
        ) {
          best = retainedPlacement.offset;
          bestRect = retainedPlacement.rect;
          bestLabelOverlapArea = retainedPlacement.labelOverlapArea;
          bestDirection = retainedPlacement.direction;
        }
        label.placementDirection = bestDirection;
        if (bestLabelOverlapArea > 0 && !hovered && !selected) {
          label.sprite.visible = false;
          label.wasVisible = false;
          label.transition = undefined;
          return;
        }
        if (bestRect) {
          placedLabels.push(bestRect);
        } else {
          placedLabels.push(compactLabelRect({ x: anchorScreen.x + best.x, y: anchorScreen.y + best.y }, label));
        }
        const targetPosition = labelAnchor.clone()
          .addScaledVector(right, best.x * worldUnitsPerPixel)
          .addScaledVector(up, -best.y * worldUnitsPerPixel);
        applyLabelPose(label, targetPosition, targetScale, visualState);
      });
    }

    function projectedInferenceAxis(anchor: THREE.Vector3, axis: InferenceAxis, sign = 1) {
      const origin = toScreen(anchor);
      const axisPoint = toScreen(anchor.clone().addScaledVector(inferenceAxisVector(axis), sign));
      const dx = axisPoint.x - origin.x;
      const dy = axisPoint.y - origin.y;
      const length = Math.hypot(dx, dy);
      if (length < 1.5) return null;
      const along = { x: dx / length, y: dy / length };
      return {
        along,
        across: { x: -along.y, y: along.x },
        pxPerWorldUnit: length,
      };
    }

    function projectedDirectionScale(anchor: THREE.Vector3, direction: THREE.Vector3) {
      const origin = toScreen(anchor);
      const directionPoint = toScreen(anchor.clone().add(direction));
      const length = Math.hypot(directionPoint.x - origin.x, directionPoint.y - origin.y);
      return length > 1e-6 ? length : 1;
    }

    function updateInferenceAxisLines(bounds: ScreenBounds) {
      const axisPositions: Record<InferenceAxis, number[]> = { x: [], y: [], z: [] };
      const targetScreenLengthPx = 76;
      editInferenceAxisCues.forEach((cue) => {
        const anchorScreen = toScreen(cue.anchor);
        if (!pointIsOnScreen(anchorScreen, bounds)) return;
        const projection = projectedInferenceAxis(cue.anchor, cue.axis, cue.sign);
        if (!projection) return;
        const worldLength = Math.min(5, Math.max(0.35, targetScreenLengthPx / projection.pxPerWorldUnit));
        const end = cue.anchor.clone().addScaledVector(inferenceAxisVector(cue.axis), worldLength * cue.sign);
        axisPositions[cue.axis].push(cue.anchor.x, cue.anchor.y, cue.anchor.z, end.x, end.y, end.z);
      });
      (Object.keys(editInferenceAxisLines) as InferenceAxis[]).forEach((axis) => {
        updateLineSegments(editInferenceAxisHaloLines[axis], axisPositions[axis]);
        updateLineSegments(editInferenceAxisLines[axis], axisPositions[axis]);
      });
    }

    function updateProjectionGuideLines(bounds: ScreenBounds) {
      const axisPositions: Record<InferenceAxis, number[]> = { x: [], y: [], z: [] };
      const activePlanes = new Set<ProjectionPlane>();
      activeProjectionVectorSegments = [];
      updateLineSegments(editProjectionAngleHaloLine, []);
      updateLineSegments(editProjectionAngleLine, []);
      if (editProjectionAngleLabel) editProjectionAngleLabel.sprite.visible = false;
      Object.values(editProjectionArrowHeads).forEach((mesh) => {
        mesh.visible = false;
      });
      Object.values(editProjectionArrowHeadHalos).forEach((mesh) => {
        mesh.visible = false;
      });
      (Object.keys(editProjectionDimensionLabels) as InferenceAxis[]).forEach((axis) => {
        const label = editProjectionDimensionLabels[axis];
        if (label) label.sprite.visible = false;
      });
      const guide = editProjectionGuides[0];
      if (guide) {
        const screenStart = toScreen(guide.start);
        const screenEnd = toScreen(guide.projectedEnd);
        const screenLength = Math.hypot(screenEnd.x - screenStart.x, screenEnd.y - screenStart.y);
        const visible = pointIsOnScreen(screenStart, bounds) || pointIsOnScreen(screenEnd, bounds);
        if (visible && screenLength >= 24) {
          let chainPoint = guide.start.clone();
          const components = PROJECTION_PLANE_AXES[guide.plane]
            .map((axis) => {
              const axisVector = inferenceAxisVector(axis);
              const component = guide.realEnd.clone().sub(guide.start).dot(axisVector);
              return { axis, axisVector, component };
            })
            .filter(({ component }) => Math.abs(component) > 1e-6);
          const visualComponents: Array<{
            axis: InferenceAxis;
            sign: number;
            length: number;
            visualStart: THREE.Vector3;
            visualTip: THREE.Vector3;
            axisDirection: THREE.Vector3;
            headLength: number;
            headRadius: number;
            haloExtra: number;
          }> = [];
          for (const [componentIndex, componentEntry] of components.entries()) {
            const { axis, axisVector, component } = componentEntry;
            const sign = component >= 0 ? 1 : -1;
            const axisDirection = axisVector.clone().multiplyScalar(sign);
            const length = Math.abs(component);
            const segmentStart = chainPoint.clone();
            const axisTip = segmentStart.clone().addScaledVector(axisDirection, length);
            const visualStart = segmentStart;
            let visualTip = axisTip;
            let visualLength = visualStart.distanceTo(visualTip);
            if (visualLength <= 0.05) {
              chainPoint = axisTip;
              break;
            }
            const projection = projectedInferenceAxis(visualStart, axis, sign);
            if (!projection) break;
            if (componentIndex === components.length - 1) {
              const endpointGap = PROJECTION_FINAL_ENDPOINT_GAP_PX / projection.pxPerWorldUnit;
              const clampedGap = Math.min(endpointGap, Math.max(0, visualLength - 0.08));
              if (clampedGap > 0) {
                visualTip = visualTip.clone().addScaledVector(axisDirection, -clampedGap);
                visualLength = visualStart.distanceTo(visualTip);
              }
            }
            const segmentScreenStart = toScreen(visualStart);
            const segmentScreenEnd = toScreen(visualTip);
            const segmentScreenLength = Math.hypot(segmentScreenEnd.x - segmentScreenStart.x, segmentScreenEnd.y - segmentScreenStart.y);
            if (segmentScreenLength < PROJECTION_MIN_COMPONENT_SCREEN_LENGTH_PX) break;
            const headLength = Math.min(PROJECTION_ARROW_HEAD_LENGTH_PX / projection.pxPerWorldUnit, visualLength);
            const headRadius = PROJECTION_ARROW_HEAD_RADIUS_PX / projection.pxPerWorldUnit;
            const haloExtra = LAYER_HALO_EXTRA_PX / projection.pxPerWorldUnit;
            visualComponents.push({
              axis,
              sign,
              length,
              visualStart,
              visualTip,
              axisDirection,
              headLength,
              headRadius,
              haloExtra,
            });
            chainPoint = axisTip;
          }
          if (components.length === 2 && visualComponents.length === 2) {
            activePlanes.add(guide.plane);
            visualComponents.forEach(({ axis, sign, length, visualStart, visualTip, axisDirection, headLength, headRadius, haloExtra }) => {
              axisPositions[axis].push(visualStart.x, visualStart.y, visualStart.z, visualTip.x, visualTip.y, visualTip.z);
              activeProjectionVectorSegments.push({ start: visualStart, end: visualTip, axis });
              const segmentScreenStart = toScreen(visualStart);
              const segmentScreenEnd = toScreen(visualTip);
              const segmentScreenLength = Math.hypot(segmentScreenEnd.x - segmentScreenStart.x, segmentScreenEnd.y - segmentScreenStart.y);
              if (segmentScreenLength >= 34) {
                const midpoint = visualStart.clone().lerp(visualTip, 0.5);
                setProjectionDimensionLabel(axis, midpoint, sign, `${formatLengthScalar(length)} ${axis.toUpperCase()}`);
              }
              const arrowHead = editProjectionArrowHeads[axis];
              arrowHead.position.copy(visualTip).addScaledVector(axisDirection, -headLength * 0.5);
              arrowHead.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), axisDirection);
              arrowHead.scale.set(headRadius / 0.32, headLength, headRadius / 0.32);
              arrowHead.visible = true;
              const arrowHeadHalo = editProjectionArrowHeadHalos[axis];
              const haloLength = headLength + haloExtra;
              const haloRadius = headRadius + haloExtra * 0.5;
              arrowHeadHalo.position.copy(visualTip).addScaledVector(axisDirection, -haloLength * 0.5);
              arrowHeadHalo.quaternion.copy(arrowHead.quaternion);
              arrowHeadHalo.scale.set(haloRadius / 0.32, haloLength, haloRadius / 0.32);
              arrowHeadHalo.visible = true;
            });
            updateProjectionAngleCue(guide, components[0]);
          }
        }
      }
      (Object.keys(editProjectionAxisLines) as InferenceAxis[]).forEach((axis) => {
        updateLineSegments(editProjectionAxisHaloLines[axis], axisPositions[axis]);
        updateLineSegments(editProjectionAxisLines[axis], axisPositions[axis]);
      });
      return activePlanes;
    }

    function updateProjectionAngleCue(
      guide: (typeof editProjectionGuides)[number],
      firstComponent: { axis: InferenceAxis; axisVector: THREE.Vector3; component: number } | undefined,
    ) {
      if (!firstComponent || !guide.angle) return;
      if (guide.angle.angleDeg <= 2 || guide.angle.angleDeg >= 88) return;
      const memberVector = guide.projectedEnd.clone().sub(guide.start);
      const memberLength = memberVector.length();
      if (memberLength <= 1e-6) return;
      const memberDirection = memberVector.clone().normalize();
      const sign = firstComponent.component >= 0 ? 1 : -1;
      const firstDirection = firstComponent.axisVector.clone().multiplyScalar(sign);
      const dotProduct = Math.min(1, Math.max(-1, memberDirection.dot(firstDirection)));
      const angleRad = Math.acos(dotProduct);
      if (angleRad <= 0.03 || angleRad >= Math.PI / 2 - 0.03) return;
      const perpendicular = memberDirection.clone().addScaledVector(firstDirection, -dotProduct);
      if (perpendicular.lengthSq() <= 1e-9) return;
      perpendicular.normalize();
      const firstLegLength = Math.abs(firstComponent.component);
      const shorterLegLength = Math.min(firstLegLength, memberLength);
      const maxInscribedRadius = shorterLegLength * 0.62;
      const firstPxPerWorldUnit = projectedDirectionScale(guide.start, firstDirection);
      const memberPxPerWorldUnit = projectedDirectionScale(guide.start, memberDirection);
      const pxPerWorldUnit = Math.max(1, Math.min(firstPxPerWorldUnit, memberPxPerWorldUnit));
      const angleT = THREE.MathUtils.clamp((guide.angle.angleDeg - 8) / 74, 0, 1);
      const radiusFraction = THREE.MathUtils.lerp(0.75, 0.25, angleT);
      const targetRadius = shorterLegLength * radiusFraction;
      const arcMinRadius = 18 / pxPerWorldUnit;
      const radius = Math.min(targetRadius, maxInscribedRadius);
      const shouldDrawArc = radius >= arcMinRadius && radius * pxPerWorldUnit >= 18;
      const positions: number[] = [];
      if (shouldDrawArc) {
        const steps = 16;
        let previous = guide.start.clone().addScaledVector(firstDirection, radius);
        for (let index = 1; index <= steps; index += 1) {
          const t = (angleRad * index) / steps;
          const next = guide.start.clone()
            .addScaledVector(firstDirection, Math.cos(t) * radius)
            .addScaledVector(perpendicular, Math.sin(t) * radius);
          positions.push(previous.x, previous.y, previous.z, next.x, next.y, next.z);
          previous = next;
        }
      }
      updateLineSegments(editProjectionAngleHaloLine, []);
      updateLineSegments(editProjectionAngleLine, positions);
      const labelRadiusPx = Math.min(targetRadius * pxPerWorldUnit, 18);
      const labelRadius = shouldDrawArc
        ? radius
        : Math.min(Math.max(0.04, labelRadiusPx / pxPerWorldUnit), Math.max(0.02, maxInscribedRadius * 0.72));
      const labelAngle = angleRad * 0.5;
      const labelAnchor = guide.start.clone()
        .addScaledVector(firstDirection, Math.cos(labelAngle) * labelRadius)
        .addScaledVector(perpendicular, Math.sin(labelAngle) * labelRadius);
      setProjectionAngleLabel(labelAnchor, `${guide.angle.angleDeg}°`);
    }

    function screenSegmentIntersection(
      a: { x: number; y: number },
      b: { x: number; y: number },
      c: { x: number; y: number },
      d: { x: number; y: number },
    ) {
      const abx = b.x - a.x;
      const aby = b.y - a.y;
      const cdx = d.x - c.x;
      const cdy = d.y - c.y;
      const denom = abx * cdy - aby * cdx;
      const abLength = Math.hypot(abx, aby);
      const cdLength = Math.hypot(cdx, cdy);
      if (Math.abs(denom) <= 1e-6 || abLength <= 1 || cdLength <= 1) return null;
      const acx = c.x - a.x;
      const acy = c.y - a.y;
      const t = (acx * cdy - acy * cdx) / denom;
      const u = (acx * aby - acy * abx) / denom;
      if (t < 0 || t > 1 || u < 0 || u > 1) return null;
      const lineDot = Math.min(1, Math.max(0, Math.abs((abx * cdx + aby * cdy) / (abLength * cdLength))));
      const angleDeg = (Math.acos(lineDot) * 180) / Math.PI;
      return { t, u, angleDeg, previewScreenLength: abLength };
    }

    function distanceToScreenLine(point: { x: number; y: number }, lineStart: { x: number; y: number }, lineEnd: { x: number; y: number }) {
      const dx = lineEnd.x - lineStart.x;
      const dy = lineEnd.y - lineStart.y;
      const length = Math.hypot(dx, dy);
      if (length <= 1e-6) return Number.POSITIVE_INFINITY;
      return Math.abs((point.x - lineStart.x) * dy - (point.y - lineStart.y) * dx) / length;
    }

    function screenColinearOverlap(
      memberStart: { x: number; y: number },
      memberEnd: { x: number; y: number },
      vectorStart: { x: number; y: number },
      vectorEnd: { x: number; y: number },
    ) {
      const vectorDx = vectorEnd.x - vectorStart.x;
      const vectorDy = vectorEnd.y - vectorStart.y;
      const memberDx = memberEnd.x - memberStart.x;
      const memberDy = memberEnd.y - memberStart.y;
      const vectorLength = Math.hypot(vectorDx, vectorDy);
      const memberLength = Math.hypot(memberDx, memberDy);
      if (vectorLength <= 1 || memberLength <= 1) return null;
      const alignment = Math.abs((vectorDx * memberDx + vectorDy * memberDy) / (vectorLength * memberLength));
      if (alignment < 0.992) return null;
      const maxLineDistance = Math.max(
        distanceToScreenLine(memberStart, vectorStart, vectorEnd),
        distanceToScreenLine(memberEnd, vectorStart, vectorEnd),
        distanceToScreenLine(vectorStart, memberStart, memberEnd),
        distanceToScreenLine(vectorEnd, memberStart, memberEnd),
      );
      if (maxLineDistance > 5) return null;
      const vectorLengthSquared = vectorLength * vectorLength;
      const memberStartU = ((memberStart.x - vectorStart.x) * vectorDx + (memberStart.y - vectorStart.y) * vectorDy) / vectorLengthSquared;
      const memberEndU = ((memberEnd.x - vectorStart.x) * vectorDx + (memberEnd.y - vectorStart.y) * vectorDy) / vectorLengthSquared;
      const padU = Math.min(0.12, 9 / vectorLength);
      const startU = Math.max(0, Math.min(memberStartU, memberEndU) - padU);
      const endU = Math.min(1, Math.max(memberStartU, memberEndU) + padU);
      if ((endU - startU) * vectorLength < 8) return null;
      return { startU, endU };
    }

    function updatePreviewHaloMasks() {
      const positions: number[] = [];
      activePreviewSplitMaskSegments.forEach((mask) => {
        positions.push(mask.start.x, mask.start.y, mask.start.z, mask.end.x, mask.end.y, mask.end.z);
      });
      updateLineSegments(editPreviewHaloLine, positions);
    }

    function pushSegmentPosition(positions: number[], start: THREE.Vector3, end: THREE.Vector3) {
      positions.push(start.x, start.y, start.z, end.x, end.y, end.z);
    }

    function cameraDepthAt(start: THREE.Vector3, end: THREE.Vector3, t: number) {
      const cameraDirection = camera.getWorldDirection(new THREE.Vector3()).normalize();
      return start.clone().lerp(end, t).sub(camera.position).dot(cameraDirection);
    }

    function updateDepthAwareProjectionCrossings() {
      const projectionForegroundPositions: Record<InferenceAxis, number[]> = { x: [], y: [], z: [] };
      const previewForegroundPositions: number[] = [];
      activeProjectionVectorSegments.forEach((vector) => {
        const vectorScreenStart = toScreen(vector.start);
        const vectorScreenEnd = toScreen(vector.end);
        const vectorScreenLength = Math.hypot(vectorScreenEnd.x - vectorScreenStart.x, vectorScreenEnd.y - vectorScreenStart.y);
        if (vectorScreenLength <= 1) return;
        memberVisualSegments.forEach((member) => {
          const memberScreenStart = toScreen(member.start);
          const memberScreenEnd = toScreen(member.end);
          const colinearOverlap = screenColinearOverlap(memberScreenStart, memberScreenEnd, vectorScreenStart, vectorScreenEnd);
          if (colinearOverlap && !member.preview) {
            pushSegmentPosition(
              projectionForegroundPositions[vector.axis],
              vector.start.clone().lerp(vector.end, colinearOverlap.startU),
              vector.start.clone().lerp(vector.end, colinearOverlap.endU),
            );
            return;
          }
          const intersection = screenSegmentIntersection(memberScreenStart, memberScreenEnd, vectorScreenStart, vectorScreenEnd);
          if (!intersection || intersection.angleDeg < 18) return;
          const memberDepth = cameraDepthAt(member.start, member.end, intersection.t);
          const vectorDepth = cameraDepthAt(vector.start, vector.end, intersection.u);
          const vectorWins = vectorDepth <= memberDepth + PROJECTION_DEPTH_TIE_EPSILON;
          const halfPx = intersection.angleDeg < 35 ? 7 : 13;
          if (vectorWins && !member.preview) {
            const halfU = halfPx / Math.max(vectorScreenLength, 1);
            const startU = Math.max(0, intersection.u - halfU);
            const endU = Math.min(1, intersection.u + halfU);
            if (endU - startU <= 1e-4) return;
            pushSegmentPosition(
              projectionForegroundPositions[vector.axis],
              vector.start.clone().lerp(vector.end, startU),
              vector.start.clone().lerp(vector.end, endU),
            );
          } else if (!vectorWins && member.preview) {
            const halfT = halfPx / Math.max(intersection.previewScreenLength, 1);
            const startT = Math.max(0, intersection.t - halfT);
            const endT = Math.min(1, intersection.t + halfT);
            if (endT - startT <= 1e-4) return;
            pushSegmentPosition(
              previewForegroundPositions,
              member.start.clone().lerp(member.end, startT),
              member.start.clone().lerp(member.end, endT),
            );
          }
        });
      });
      (Object.keys(editProjectionForegroundAxisLines) as InferenceAxis[]).forEach((axis) => {
        // Foreground projection segments sit over members; a second wide halo cuts visible gaps into the same shaft.
        updateLineSegments(editProjectionForegroundAxisHaloLines[axis], []);
        updateLineSegments(editProjectionForegroundAxisLines[axis], projectionForegroundPositions[axis]);
      });
      updateLineSegments(editPreviewForegroundHaloLine, previewForegroundPositions);
      updateLineSegments(editPreviewForegroundLine, previewForegroundPositions);
    }

    function inferenceLabelOffsetCandidates(label: { anchor: THREE.Vector3; axis?: InferenceAxis; sign?: number; offset: { x: number; y: number } }) {
      if (label.axis) {
        const projection = projectedInferenceAxis(label.anchor, label.axis, label.sign ?? 1);
        if (projection) {
          const { along, across } = projection;
          return [
            { x: along.x * 58 + across.x * 18, y: along.y * 58 + across.y * 18, preference: 0 },
            { x: along.x * 58 - across.x * 18, y: along.y * 58 - across.y * 18, preference: 1 },
            { x: along.x * 76 + across.x * 28, y: along.y * 76 + across.y * 28, preference: 2 },
            { x: along.x * 76 - across.x * 28, y: along.y * 76 - across.y * 28, preference: 3 },
            { x: along.x * 42 + across.x * 34, y: along.y * 42 + across.y * 34, preference: 4 },
            { x: along.x * 42 - across.x * 34, y: along.y * 42 - across.y * 34, preference: 5 },
          ];
        }
      }
      const fallback = label.offset;
      return [
        { ...fallback, preference: 0 },
        { x: -fallback.x, y: fallback.y, preference: 1 },
        { x: fallback.x, y: -fallback.y, preference: 2 },
        { x: -fallback.x, y: -fallback.y, preference: 3 },
      ];
    }

    function updateSymbolSprites() {
      const r = renderer.domElement.getBoundingClientRect();
      const visibleBounds = visibleScreenBounds(r.width, r.height);
      const worldUnitsPerPixel = viewSize / Math.max(r.height, 1) / Math.max(camera.zoom, 0.001);
      camera.updateMatrixWorld(true);
      const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
      const up = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
      updateProjectionGuideLines(visibleBounds);
      updatePreviewHaloMasks();
      updateDepthAwareProjectionCrossings();
      updateInferenceAxisLines(visibleBounds);
      if (editSnapGlyph) {
        editSnapGlyph.sprite.position.copy(editSnapGlyph.anchor);
        editSnapGlyph.sprite.scale.set(editSnapGlyph.widthPx * worldUnitsPerPixel, editSnapGlyph.heightPx * worldUnitsPerPixel, 1);
      }
      if (editSnapLabel) {
        editSnapLabel.sprite.position.copy(editSnapLabel.anchor)
          .addScaledVector(right, 24 * worldUnitsPerPixel)
          .addScaledVector(up, -24 * worldUnitsPerPixel);
        editSnapLabel.sprite.scale.set(editSnapLabel.widthPx * worldUnitsPerPixel, editSnapLabel.heightPx * worldUnitsPerPixel, 1);
      }
      (Object.keys(editProjectionDimensionLabels) as InferenceAxis[]).forEach((axis) => {
        const label = editProjectionDimensionLabels[axis];
        if (!label) return;
        label.sprite.scale.set(label.widthPx * worldUnitsPerPixel, label.heightPx * worldUnitsPerPixel, 1);
        const anchorScreen = toScreen(label.anchor);
        label.sprite.visible = label.sprite.visible && pointIsOnScreen(anchorScreen, visibleBounds);
        if (!label.sprite.visible) return;
        label.sprite.position.copy(label.anchor);
      });
      if (editProjectionAngleLabel) {
        editProjectionAngleLabel.sprite.scale.set(editProjectionAngleLabel.widthPx * worldUnitsPerPixel, editProjectionAngleLabel.heightPx * worldUnitsPerPixel, 1);
        const anchorScreen = toScreen(editProjectionAngleLabel.anchor);
        editProjectionAngleLabel.sprite.visible = editProjectionAngleLabel.sprite.visible && pointIsOnScreen(anchorScreen, visibleBounds);
        if (editProjectionAngleLabel.sprite.visible) {
          editProjectionAngleLabel.sprite.position.copy(editProjectionAngleLabel.anchor);
        }
      }
      if (editCoordinateLabel) {
        editCoordinateLabel.sprite.scale.set(editCoordinateLabel.widthPx * worldUnitsPerPixel, editCoordinateLabel.heightPx * worldUnitsPerPixel, 1);
        const anchorScreen = toScreen(editCoordinateLabel.anchor);
        editCoordinateLabel.sprite.visible = pointIsOnScreen(anchorScreen, visibleBounds);
        if (editCoordinateLabel.sprite.visible) {
          const coordinateObstacles: ReturnType<typeof screenRect>[] = [];
          activeProjectionVectorSegments.forEach((segment) => {
            coordinateObstacles.push(...screenSegmentObstacleRects(segment.start, segment.end, 18, 4));
          });
          if (currentEditOverlay?.previewLine) {
            coordinateObstacles.push(...screenSegmentObstacleRects(
              pointFromOverlay(currentEditOverlay.previewLine.start),
              pointFromOverlay(currentEditOverlay.previewLine.end),
              20,
              4,
            ));
          }
          if (editSnapGlyph) {
            coordinateObstacles.push(screenRect(toScreen(editSnapGlyph.anchor), editSnapGlyph.widthPx, editSnapGlyph.heightPx, 8));
          }
          coordinateObstacles.push(screenRect(anchorScreen, NODE_POINT_SIZE_PX, NODE_POINT_SIZE_PX, 8));
          (Object.keys(editProjectionDimensionLabels) as InferenceAxis[]).forEach((axis) => {
            const label = editProjectionDimensionLabels[axis];
            if (!label?.sprite.visible) return;
            coordinateObstacles.push(screenRect(toScreen(label.sprite.position), label.widthPx, label.heightPx, 5));
          });
          if (editProjectionAngleLabel?.sprite.visible) {
            coordinateObstacles.push(screenRect(toScreen(editProjectionAngleLabel.sprite.position), editProjectionAngleLabel.widthPx, editProjectionAngleLabel.heightPx, 5));
          }
          const verticalGap = editCoordinateLabel.heightPx / 2 + NODE_POINT_SIZE_PX / 2 + 12;
          const horizontalGap = editCoordinateLabel.widthPx / 2 + NODE_POINT_SIZE_PX / 2 + 12;
          const candidates = [
            { x: 0, y: -verticalGap, preference: 0 },
            { x: 0, y: verticalGap, preference: 1 },
            { x: horizontalGap, y: 0, preference: 2 },
            { x: -horizontalGap, y: 0, preference: 3 },
          ];
          let best = candidates[0];
          let bestScore = Number.POSITIVE_INFINITY;
          for (const offset of candidates) {
            const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
            const center = clampLabelCenter(unclampedCenter, editCoordinateLabel, visibleBounds);
            const effectiveOffset = { ...offset, x: center.x - anchorScreen.x, y: center.y - anchorScreen.y };
            const rect = screenRect(center, editCoordinateLabel.widthPx, editCoordinateLabel.heightPx, 3);
            const obstacleOverlapArea = coordinateObstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
            const overflowArea = viewportOverflowArea(rect, visibleBounds);
            const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
            const distancePenalty = Math.hypot(effectiveOffset.x, effectiveOffset.y) * 0.04;
            const score =
              obstacleOverlapArea * 16000 +
              overflowArea * 50000 +
              clampDistance * 12 +
              distancePenalty +
              offset.preference;
            if (score < bestScore) {
              bestScore = score;
              best = effectiveOffset;
            }
          }
          editCoordinateLabel.sprite.position.copy(editCoordinateLabel.anchor)
            .addScaledVector(right, best.x * worldUnitsPerPixel)
            .addScaledVector(up, -best.y * worldUnitsPerPixel);
        }
      }
      const placedEditPrimitiveLabels: ReturnType<typeof screenRect>[] = [];
      editPrimitiveLabels.forEach((label) => {
        label.sprite.scale.set(label.widthPx * worldUnitsPerPixel, label.heightPx * worldUnitsPerPixel, 1);
        const anchorScreen = toScreen(label.anchor);
        label.sprite.visible = pointIsOnScreen(anchorScreen, visibleBounds);
        if (!label.sprite.visible) return;
        const gap = label.anchorClearancePx + labelAnchorGapPx + 5;
        const preferred = [
          label.offset,
          { x: 0, y: -(label.heightPx / 2 + gap) },
          { x: 0, y: label.heightPx / 2 + gap },
          { x: label.widthPx / 2 + gap, y: 0 },
          { x: -(label.widthPx / 2 + gap), y: 0 },
        ];
        const obstacles = [
          ...modelObstacleRects(),
          ...placedEditPrimitiveLabels,
          screenRect(anchorScreen, NODE_POINT_SIZE_PX, NODE_POINT_SIZE_PX, 8),
        ];
        if (currentEditOverlay?.previewLine) {
          obstacles.push(...screenSegmentObstacleRects(
            pointFromOverlay(currentEditOverlay.previewLine.start),
            pointFromOverlay(currentEditOverlay.previewLine.end),
            18,
            4,
          ));
        }
        let best = preferred[0];
        let bestScore = Number.POSITIVE_INFINITY;
        preferred.forEach((offset, index) => {
          const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
          const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
          const effectiveOffset = { x: center.x - anchorScreen.x, y: center.y - anchorScreen.y };
          const rect = screenRect(center, label.widthPx, label.heightPx, 3);
          const overlapArea = obstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          const overflowArea = viewportOverflowArea(rect, visibleBounds);
          const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
          const score = overlapArea * 12000 + overflowArea * 50000 + clampDistance * 4 + index;
          if (score < bestScore) {
            bestScore = score;
            best = effectiveOffset;
          }
        });
        label.sprite.position.copy(label.anchor)
          .addScaledVector(right, best.x * worldUnitsPerPixel)
          .addScaledVector(up, -best.y * worldUnitsPerPixel);
        placedEditPrimitiveLabels.push(screenRect(toScreen(label.sprite.position), label.widthPx, label.heightPx, 3));
      });
      const placedInferenceLabels: ReturnType<typeof screenRect>[] = [];
      const inferenceObstacles = modelObstacleRects();
      editInferenceLabels.forEach((label) => {
        label.sprite.scale.set(label.widthPx * worldUnitsPerPixel, label.heightPx * worldUnitsPerPixel, 1);
        const anchorScreen = toScreen(label.anchor);
        label.sprite.visible = pointIsOnScreen(anchorScreen, visibleBounds);
        if (!label.sprite.visible) return;
        let best = label.offset;
        let bestRect: ReturnType<typeof screenRect> | undefined;
        let bestScore = Number.POSITIVE_INFINITY;
        for (const offset of inferenceLabelOffsetCandidates(label)) {
          const unclampedCenter = { x: anchorScreen.x + offset.x, y: anchorScreen.y + offset.y };
          const center = clampLabelCenter(unclampedCenter, label, visibleBounds);
          const effectiveOffset = { ...offset, x: center.x - anchorScreen.x, y: center.y - anchorScreen.y };
          const rect = screenRect(center, label.widthPx, label.heightPx, 3);
          const labelOverlapArea = placedInferenceLabels.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          const modelOverlapArea = inferenceObstacles.reduce((sum, existing) => sum + rectOverlapArea(rect, existing), 0);
          const overflowArea = viewportOverflowArea(rect, visibleBounds);
          const clampDistance = Math.hypot(center.x - unclampedCenter.x, center.y - unclampedCenter.y);
          const score =
            labelOverlapArea * 20000 +
            modelOverlapArea * 1200 +
            overflowArea * 50000 +
            clampDistance * 2 +
            offset.preference;
          if (score < bestScore) {
            bestScore = score;
            best = effectiveOffset;
            bestRect = rect;
          }
        }
        if (bestRect) placedInferenceLabels.push(bestRect);
        label.sprite.position.copy(label.anchor)
          .addScaledVector(right, best.x * worldUnitsPerPixel)
          .addScaledVector(up, -best.y * worldUnitsPerPixel);
      });
      for (const item of symbolSprites) {
        const rawOffset = item.offset ?? { x: 0, y: 0 };
        const overviewScale = item.focused ? 1 : currentVisualProfile.symbolScale;
        const offset = { x: rawOffset.x * overviewScale, y: rawOffset.y * overviewScale };
        const symbolScale = (item.tone === 'support' ? SUPPORT_SYMBOL_SCALE : 1) * overviewScale;
        let screenOffset = { x: offset.x, y: offset.y };
        if (item.direction) {
          const p = toScreen(item.anchor);
          const tail = toScreen(item.anchor.clone().addScaledVector(item.direction, -1));
          const dx = p.x - tail.x;
          const dy = p.y - tail.y;
          const length = Math.hypot(dx, dy) || 1;
          const along = { x: dx / length, y: dy / length };
          const across = { x: along.y, y: -along.x };
          screenOffset = {
            x: across.x * offset.x + along.x * offset.y,
            y: across.y * offset.x + along.y * offset.y,
          };
        }
        item.sprite.position.copy(item.anchor)
          .addScaledVector(right, screenOffset.x * worldUnitsPerPixel)
          .addScaledVector(up, -screenOffset.y * worldUnitsPerPixel);
        item.sprite.scale.set(item.widthPx * symbolScale * worldUnitsPerPixel, item.heightPx * symbolScale * worldUnitsPerPixel, 1);
        item.material.opacity = item.proposed ? currentVisualProfile.detail : 1;
        if (item.halo) {
          item.halo.sprite.position.copy(item.sprite.position);
          item.halo.sprite.scale.set(
            (item.widthPx * symbolScale + 4) * worldUnitsPerPixel,
            (item.heightPx * symbolScale + 4) * worldUnitsPerPixel,
            1,
          );
        }
        if (item.direction) {
          const p = toScreen(item.anchor);
          const tail = toScreen(item.anchor.clone().addScaledVector(item.direction, -1));
          const angle = Math.atan2(p.y - tail.y, p.x - tail.x) - Math.PI / 2;
          item.material.rotation = angle;
          if (item.halo) item.halo.material.rotation = angle;
        } else {
          item.material.rotation = 0;
        }
      }
    }
    scheduleRender();

    return () => {
      saveCameraState();
      cancelSmoothFitAnimation();
      if (renderFrameId) cancelAnimationFrame(renderFrameId);
      if (sceneApiRef.current?.updateFocusedTargets === updateFocusedTargets) {
        sceneApiRef.current = null;
      }
      if ((window as any).__FRAIA_VIEWPORT_STATS__ === viewportStats) {
        delete (window as any).__FRAIA_VIEWPORT_STATS__;
      }
      themeQuery.removeEventListener('change', handleThemeChange);
      window.removeEventListener('fraia:themechange', handleThemeChange);
      renderer.domElement.removeEventListener('pointerdown', handlePointerDown, true);
      renderer.domElement.removeEventListener('pointermove', handlePointerMove, true);
      renderer.domElement.removeEventListener('pointerup', handlePointerUp, true);
      renderer.domElement.removeEventListener('pointerleave', handlePointerLeave);
      renderer.domElement.removeEventListener('pointercancel', handlePointerCancel);
      window.removeEventListener('keydown', handleSelectionModifier);
      window.removeEventListener('keyup', handleSelectionModifier);
      document.removeEventListener('pointermove', handleStrandChordPointerMove, true);
      document.removeEventListener('pointerup', handleStrandChordPointerUp, true);
      document.removeEventListener('pointercancel', handleStrandChordPointerUp, true);
      if (resizeRenderLoopId) cancelAnimationFrame(resizeRenderLoopId);
      ro.disconnect();
      controls.removeEventListener('start', markCameraInteractionStart);
      controls.removeEventListener('change', rememberCamera);
      viewGizmo.removeEventListener('start', handleViewGizmoStart);
      viewGizmo.removeEventListener('change', handleViewGizmoChange);
      viewGizmo.removeEventListener('end', handleViewGizmoEnd);
      unbindViewGizmoGlobalHover();
      viewGizmo.dispose();
      controls.dispose();
      renderer.dispose();
      selectionCanvas.remove();
      memberObjects.forEach((o) => o.geometry.dispose());
      memberBatchObjects.forEach((o) => o.geometry.dispose());
      const disposeLabel = (label: LabelSprite) => {
        const textures = label.stateTextures
          ? Object.values(label.stateTextures)
          : [label.texture];
        [...new Set(textures)].forEach((texture) => texture.dispose());
        label.material.dispose();
      };
      memberLabelSprites.forEach(disposeLabel);
      nodeLabelSprites.forEach(disposeLabel);
      loadLabelSprites.forEach(disposeLabel);
      supportLabelSprites.forEach(disposeLabel);
      symbolSprites.forEach((symbol) => {
        symbol.texture.dispose();
        symbol.material.dispose();
        symbol.halo?.material.dispose();
      });
      nodeObjects.forEach((o) => o.geometry.dispose());
      baseMemberHaloMat.dispose();
      memberBatchMat.dispose();
      previewMemberHaloMat.dispose();
      previewMemberBatchMat.dispose();
      selectedMemberBatchMat.dispose();
      hoverMemberMat.dispose();
      loadMat.dispose();
      focusedLoadMat.dispose();
      loadHaloMat.dispose();
      focusedLoadHaloMat.dispose();
      hoverLoadHaloMat.dispose();
      releaseHaloMat.dispose();
      allReleaseMats.forEach((material) => material.dispose());
      nodeMat.dispose();
      proposedSupportNodeMat.dispose();
      focusedNodeMat.dispose();
      selectedNodeFillMat.dispose();
      hoverNodeFillMat.dispose();
      previewNodeFillMat.dispose();
      nodeTexture.dispose();
      editGridLine.geometry.dispose();
      editPreviewHaloLine.geometry.dispose();
      editPreviewLine.geometry.dispose();
      editPreviewSplitLine.geometry.dispose();
      editPreviewForegroundHaloLine.geometry.dispose();
      editPreviewForegroundLine.geometry.dispose();
      editPreviewNodeGeometry.dispose();
      Object.values(editGuideLines).forEach((line) => line.geometry.dispose());
      Object.values(editGuideHaloLines).forEach((line) => line.geometry.dispose());
      Object.values(editInferenceAxisLines).forEach((line) => line.geometry.dispose());
      Object.values(editInferenceAxisHaloLines).forEach((line) => line.geometry.dispose());
      Object.values(editProjectionAxisLines).forEach((line) => line.geometry.dispose());
      Object.values(editProjectionAxisHaloLines).forEach((line) => line.geometry.dispose());
      Object.values(editProjectionForegroundAxisLines).forEach((line) => line.geometry.dispose());
      Object.values(editProjectionForegroundAxisHaloLines).forEach((line) => line.geometry.dispose());
      editProjectionAngleLine.geometry.dispose();
      editProjectionAngleHaloLine.geometry.dispose();
      editProjectionArrowGeometry.dispose();
      editGridMat.dispose();
      editPreviewHaloMat.dispose();
      editPreviewMat.dispose();
      editPreviewSplitMat.dispose();
      Object.values(editGuideMats).forEach((material) => material.dispose());
      editGuideHaloMat.dispose();
      Object.values(editInferenceAxisMats).forEach((material) => material.dispose());
      editInferenceAxisHaloMat.dispose();
      Object.values(editProjectionAxisMats).forEach((material) => material.dispose());
      editProjectionAxisHaloMat.dispose();
      editProjectionAngleMat.dispose();
      editProjectionAngleHaloMat.dispose();
      editProjectionArrowHaloMat.dispose();
      Object.values(editProjectionArrowMats).forEach((material) => material.dispose());
      (Object.keys(editProjectionDimensionLabels) as InferenceAxis[]).forEach((axis) => {
        const label = editProjectionDimensionLabels[axis];
        if (!label) return;
        label.texture.dispose();
        label.material.dispose();
      });
      if (editCoordinateLabel) {
        editCoordinateLabel.texture.dispose();
        editCoordinateLabel.material.dispose();
      }
      if (editProjectionAngleLabel) {
        editProjectionAngleLabel.texture.dispose();
        editProjectionAngleLabel.material.dispose();
      }
      if (editSnapGlyph) {
        editSnapGlyph.texture.dispose();
        editSnapGlyph.material.dispose();
      }
      if (editSnapLabel) {
        editSnapLabel.texture.dispose();
        editSnapLabel.material.dispose();
      }
      clearEditPrimitiveLabels();
      clearEditInferenceLabels();
      if (renderer.domElement.parentElement === el) el.removeChild(renderer.domElement);
    };
  }, [scene]);

  return (
    <div className="relative h-full w-full">
      <div ref={ref} className="h-full w-full" />
      {(scene.releases?.length ?? 0) > 0 && (
        <Alert
          className="absolute w-fit max-w-[min(32rem,calc(100%-1.5rem))]"
          style={{
            left: `${Math.max(12, fitInsetLeft + 12)}px`,
            bottom: `${Math.max(12, fitInsetBottom + 12)}px`,
          }}
          title="End releases use member-local axes. Red, green, and blue are local X, Y, and Z. Positive local axis ticks indicate translational releases; negative local axis ticks indicate rotational releases."
        >
          <AlertDescription>End releases: X/Y/Z colours use local axes; + = translation, - = rotation</AlertDescription>
        </Alert>
      )}
    </div>
  );
}
