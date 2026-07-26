import type { BackendDesignScheme, BaseCleanupProposal, EngineeringScheme, RenderLoad, RenderMember, RenderNode, RenderRelease, RenderScene, RenderSupport, SceneMember, SceneNode, SchemeWorkspaceState, WorkbenchState } from './types';
import { unitProfileFrom } from './units';

function p(n: SceneNode): [number, number, number] {
  const q: any = n.position;
  if (Array.isArray(q)) return [q[0] ?? 0, q[1] ?? 0, q[2] ?? 0];
  if (q) return [q.x ?? 0, q.y ?? 0, q.z ?? 0];
  return [n.x ?? 0, n.y ?? 0, n.z ?? 0];
}
function start(m: SceneMember) { return m.startNode ?? m.start_node ?? m.i ?? m.node_i ?? m.start ?? ''; }
function end(m: SceneMember) { return m.endNode ?? m.end_node ?? m.j ?? m.node_j ?? m.end ?? ''; }
function normalizeSectionCoordination(m: SceneMember) {
  const explicit = m.sectionCoordination ?? m.section_coordination;
  if (explicit?.kind) {
    return {
      kind: explicit.kind.toLowerCase(),
      groupLabel: explicit.groupLabel ?? explicit.group_label,
    };
  }
  const label = m.familyGroupLabel ?? m.family_group_label;
  if (!label) return undefined;
  if (label.trim().toLowerCase() === 'unique') return { kind: 'independent' };
  return { kind: 'shared', groupLabel: label };
}
function normalizeSizeCoordination(m: SceneMember) {
  const explicit = m.sizeCoordination ?? m.size_coordination;
  if (explicit?.kind) {
    return {
      kind: explicit.kind.toLowerCase(),
      groupLabel: explicit.groupLabel ?? explicit.group_label,
    };
  }
  const label = m.sizeGroupLabel ?? m.size_group_label;
  if (!label) return undefined;
  if (['size independent', 'unique'].includes(label.trim().toLowerCase())) return { kind: 'independent' };
  return { kind: 'shared', groupLabel: label };
}
function memberSectionKey(member: { familyGroupLabel?: string; sectionCoordination?: { kind?: string; groupLabel?: string } }) {
  if (member.sectionCoordination?.kind === 'shared') return member.sectionCoordination.groupLabel ?? member.familyGroupLabel;
  return undefined;
}
function memberSizeKey(member: { familyGroupLabel?: string; sizeGroupLabel?: string; sectionCoordination?: { kind?: string; groupLabel?: string }; sizeCoordination?: { kind?: string; groupLabel?: string } }) {
  if (member.sizeCoordination?.kind !== 'shared') return undefined;
  const size = member.sizeCoordination.groupLabel ?? member.sizeGroupLabel;
  if (!size) return undefined;
  return `${memberSectionKey(member) ?? ''}|${size}`;
}
function applySingletonCoordinationFallbacks<T extends { familyGroupLabel?: string; sizeGroupLabel?: string; sectionCoordination?: { kind?: string; groupLabel?: string }; sizeCoordination?: { kind?: string; groupLabel?: string } }>(members: T[]): T[] {
  const sectionCounts = new Map<string, number>();
  const sizeCounts = new Map<string, number>();
  members.forEach((member) => {
    const sectionKey = memberSectionKey(member);
    if (sectionKey) sectionCounts.set(sectionKey, (sectionCounts.get(sectionKey) ?? 0) + 1);
    const sizeKey = memberSizeKey(member);
    if (sizeKey) sizeCounts.set(sizeKey, (sizeCounts.get(sizeKey) ?? 0) + 1);
  });
  return members.map((member) => {
    const sectionKey = memberSectionKey(member);
    const sizeKey = memberSizeKey(member);
    return {
      ...member,
      sectionCoordination: sectionKey && (sectionCounts.get(sectionKey) ?? 0) <= 1 ? { kind: 'independent' } : member.sectionCoordination,
      sizeCoordination: sizeKey && (sizeCounts.get(sizeKey) ?? 0) <= 1 ? { kind: 'independent' } : member.sizeCoordination,
    };
  });
}
function supportNode(s: RenderSupport) { return s.targetNode ?? s.target_node ?? ''; }
function loadTarget(load: RenderLoad) {
  const label = load.targetLabel ?? load.target_label ?? '';
  if (label.startsWith('member ')) return label.slice(7);
  return load.targetMember ?? load.target_member ?? load.memberId ?? load.member_id ?? '';
}
function loadNodeTarget(load: RenderLoad) {
  const label = load.targetLabel ?? load.target_label ?? '';
  if (label.startsWith('node ')) return label.slice(5);
  return load.targetNode ?? load.target_node ?? '';
}

function quantityValue(value: RenderLoad['magnitude'] | null | undefined) {
  if (typeof value === 'number') return value;
  if (value && typeof value === 'object' && typeof value.value === 'number') return value.value;
  return undefined;
}

function fallbackScene(): RenderScene {
  return {
    nodes: [],
    members: [],
    supports: [],
    loads: [],
    releases: [],
  };
}

export function normalizeSupports(raw?: { supports?: RenderSupport[] }): RenderSupport[] {
  return (raw?.supports ?? [])
    .map((support) => ({ ...support, targetNode: supportNode(support), supportGroupLabel: support.supportGroupLabel ?? support.support_group_label }))
    .filter((support) => Boolean(support.id && support.targetNode));
}

export function normalizeLoads(raw?: { loads?: RenderLoad[] }): RenderLoad[] {
  return (raw?.loads ?? [])
    .map((load) => ({
      ...load,
      magnitude: quantityValue(load.magnitude),
      directionX: load.directionX ?? load.direction_x ?? 0,
      directionY: load.directionY ?? load.direction_y ?? -1,
      directionZ: load.directionZ ?? load.direction_z ?? 0,
      targetMember: loadTarget(load),
      targetNode: loadNodeTarget(load),
      semanticLabel: load.semanticLabel ?? load.semantic_label,
    }))
    .filter((load) => Boolean(load.id));
}

export function normalizeReleases(raw?: { releases?: RenderRelease[] }): RenderRelease[] {
  return (raw?.releases ?? [])
    .map((release) => ({
      ...release,
      memberId: release.memberId ?? release.member_id ?? '',
      end: String(release.end ?? 'end').toLowerCase(),
    }))
    .filter((release) => Boolean(release.id && release.memberId));
}

function briefVisualIntent(state: WorkbenchState | null) {
  const brief = state?.baseModelBrief ?? state?.base_model_brief;
  return brief?.visualIntent ?? brief?.visual_intent ?? null;
}

function loadIntentTargetKind(target: { kind?: string } | undefined) {
  return target?.kind ?? '';
}

function loadIntentMemberId(target: { memberId?: string | null; member_id?: string | null } | undefined) {
  return target?.memberId ?? target?.member_id ?? '';
}

function loadIntentNodeId(target: { nodeId?: string | null; node_id?: string | null } | undefined) {
  return target?.nodeId ?? target?.node_id ?? '';
}

function directionVector(scene: RenderScene, direction: { kind?: string; fromNode?: string | null; from_node?: string | null; toNode?: string | null; to_node?: string | null; x?: number | null; y?: number | null; z?: number | null } | null | undefined) {
  if (!direction) return null;
  if (direction.kind === 'vector') {
    const x = direction.x ?? 0;
    const y = direction.y ?? 0;
    const z = direction.z ?? 0;
    const length = Math.hypot(x, y, z);
    if (!length) return null;
    return { directionX: x / length, directionY: y / length, directionZ: z / length };
  }
  if (direction.kind === 'toward_node') {
    const fromId = direction.fromNode ?? direction.from_node ?? '';
    const toId = direction.toNode ?? direction.to_node ?? '';
    const nodesById = new Map(scene.nodes.map((node) => [node.id, node]));
    const from = nodesById.get(fromId);
    const to = nodesById.get(toId);
    if (!from || !to || from.id === to.id) return null;
    const x = to.x - from.x;
    const y = to.y - from.y;
    const z = to.z - from.z;
    const length = Math.hypot(x, y, z);
    if (!length) return null;
    return { directionX: x / length, directionY: y / length, directionZ: z / length };
  }
  return null;
}

function addBriefVisualAssumptions(scene: RenderScene, state: WorkbenchState | null, options: { supportLocations?: boolean; loads?: boolean } = {}): RenderScene {
  const visualIntent = briefVisualIntent(state);
  if (!visualIntent) return scene;
  const includeSupportLocations = options.supportLocations ?? true;
  const includeLoads = options.loads ?? true;
  const supports = [...scene.supports];
  const loads = [...scene.loads];

  const nodesById = new Map(scene.nodes.map((node) => [node.id, node]));
  const membersById = new Map(scene.members.map((member) => [member.id, member]));

  if (includeSupportLocations) {
    for (const support of visualIntent.supportLocations ?? visualIntent.support_locations ?? []) {
      const targetNode = support.targetNode ?? support.target_node ?? '';
      if (!targetNode || !nodesById.has(targetNode)) continue;
      if (!supports.some((existing) => existing.id === `brief-visual-support-${support.id}` || supportNode(existing) === targetNode)) {
        supports.push({
          id: `brief-visual-support-${support.id}`,
          targetNode,
          supportGroupLabel: support.label ?? 'Location only',
          ux: false,
          uy: false,
          uz: false,
          rx: false,
          ry: false,
          rz: false,
        });
      }
    }
  }

  if (includeLoads) {
    for (const load of visualIntent.loads ?? []) {
      if (!load.id || loads.some((existing) => existing.id === `brief-visual-load-${load.id}`)) continue;
      const target = load.target;
      const targetKind = loadIntentTargetKind(target);
      if (load.kind === 'self_weight') {
        const targetMembers = targetKind === 'all_members'
          ? scene.members
          : targetKind === 'member'
            ? [membersById.get(loadIntentMemberId(target))].filter(Boolean) as RenderScene['members']
            : [];
        targetMembers.forEach((member) => {
          loads.push({
            id: `brief-visual-load-${load.id}-${member.id}`,
            kind: 'uniform_line',
            targetMember: member.id,
            semanticLabel: 'self_weight',
            directionX: 0,
            directionY: -1,
            directionZ: 0,
          });
        });
        continue;
      }
      if (load.kind === 'point') {
        const vector = directionVector(scene, load.direction);
        const targetNodeId = loadIntentNodeId(target);
        const targetNode = targetKind === 'node' ? nodesById.get(targetNodeId) : undefined;
        const magnitude = typeof load.magnitude === 'number'
          ? load.magnitude
          : load.magnitude?.value ?? load.magnitudeN ?? load.magnitude_n ?? legacyKnToN(load.magnitudeKn ?? load.magnitude_kn);
        if (!vector || !targetNode || magnitude == null) continue;
        loads.push({
          id: `brief-visual-load-${load.id}`,
          kind: 'point',
          magnitude,
          targetNode: targetNode.id,
          x: targetNode.x,
          y: targetNode.y,
          z: targetNode.z,
          directionX: vector.directionX,
          directionY: vector.directionY,
          directionZ: vector.directionZ,
        });
        continue;
      }
      if (load.kind === 'uniform_line') {
        const vector = directionVector(scene, load.direction);
        const targetMemberId = loadIntentMemberId(target);
        const targetMember = targetKind === 'member' ? membersById.get(targetMemberId) : undefined;
        const magnitude = typeof load.magnitude === 'number'
          ? load.magnitude
          : load.magnitude?.value ?? load.magnitudeNPerM ?? load.magnitude_n_per_m;
        if (!vector || !targetMember || magnitude == null) continue;
        loads.push({
          id: `brief-visual-load-${load.id}`,
          kind: 'uniform_line',
          magnitude,
          targetMember: targetMember.id,
          directionX: vector.directionX,
          directionY: vector.directionY,
          directionZ: vector.directionZ,
        });
      }
    }
  }

  return { ...scene, supports, loads };
}

function legacyKnToN(value: number | null | undefined) {
  return value == null ? undefined : value * 1000;
}

function nodeById(scene: RenderScene) {
  return new Map(scene.nodes.map((node) => [node.id, node]));
}

function memberIntentText(scheme: BackendDesignScheme) {
  const intent = scheme.intent;
  return [
    scheme.id,
    scheme.label,
    scheme.strategy,
    scheme.summary,
    scheme.differentiation,
    scheme.supportStrategy ?? scheme.support_strategy,
    scheme.standardisationStrategy ?? scheme.standardisation_strategy,
    scheme.connectionStrategy ?? scheme.connection_strategy,
    intent?.id,
    intent?.label,
    intent?.hypothesis,
    intent?.explorationBand ?? intent?.exploration_band,
    intent?.objectiveTags?.join(' ') ?? intent?.objective_tags?.join(' '),
    intent?.standardisationStrategy ?? intent?.standardisation_strategy,
    intent?.connectionStrategy ?? intent?.connection_strategy,
    intent?.supportStrategy ?? intent?.support_strategy,
    intent?.sectionFamilyPolicy ?? intent?.section_family_policy,
    intent?.coordinationGroupPolicy ?? intent?.coordination_group_policy,
    intent?.assumptions?.join(' '),
  ].filter(Boolean).join(' ').toLowerCase();
}

function addNodeIfMissing(scene: RenderScene, node: RenderNode) {
  if (!scene.nodes.some((existing) => existing.id === node.id)) scene.nodes.push(node);
}

function addMemberIfMissing(scene: RenderScene, member: RenderMember) {
  if (!scene.members.some((existing) => existing.id === member.id)) scene.members.push(member);
}

function pointAlong(from: RenderNode, to: RenderNode, distance: number): RenderNode {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const dz = to.z - from.z;
  const length = Math.hypot(dx, dy, dz) || 1;
  const t = Math.min(0.35, Math.max(0.08, distance / length));
  return {
    id: '',
    x: from.x + dx * t,
    y: from.y + dy * t,
    z: from.z + dz * t,
  };
}

function addLocalStiffeningPreview(scene: RenderScene, scheme: BackendDesignScheme) {
  const nodes = nodeById(scene);
  const incident = new Map<string, SceneMember[]>();
  scene.members.forEach((member) => {
    const a = start(member);
    const b = end(member);
    if (!a || !b) return;
    incident.set(a, [...(incident.get(a) ?? []), member]);
    incident.set(b, [...(incident.get(b) ?? []), member]);
  });
  let previewIndex = 1;
  for (const [jointId, members] of incident) {
    if (members.length < 2) continue;
    const joint = nodes.get(jointId);
    if (!joint) continue;
    const ordered = [...members].slice(0, 2);
    const previewNodes = ordered.map((member, index) => {
      const otherId = start(member) === jointId ? end(member) : start(member);
      const other = nodes.get(otherId);
      if (!other) return null;
      const point = pointAlong(joint, other, 1.6);
      point.id = `${scheme.id}::stiffener-node-${previewIndex}-${index + 1}`;
      point.source = 'scheme';
      addNodeIfMissing(scene, point);
      nodes.set(point.id, point);
      return point;
    });
    if (!previewNodes[0] || !previewNodes[1]) continue;
    addMemberIfMissing(scene, {
      id: `${scheme.id}::local-stiffener-${previewIndex}`,
      role: 'brace',
      start: previewNodes[0].id,
      end: previewNodes[1].id,
      source: 'scheme',
      schemeNote: 'approval required',
    });
    previewIndex += 1;
  }
}

function addAlternateLoadPathPreview(scene: RenderScene, scheme: BackendDesignScheme) {
  const nodes = nodeById(scene);
  const supportNodes = scene.supports.map((support) => supportNode(support)).filter(Boolean);
  const membersByPair = new Set(scene.members.map((member) => [start(member), end(member)].sort().join('|')));
  const candidates = scene.nodes
    .filter((node) => !supportNodes.includes(node.id))
    .sort((a, b) => (b.y ?? 0) - (a.y ?? 0));
  supportNodes.slice(0, 2).forEach((supportNodeId, index) => {
    const support = nodes.get(supportNodeId);
    if (!support) return;
    const target = candidates.find((candidate) => {
      const pair = [supportNodeId, candidate.id].sort().join('|');
      return pair && !membersByPair.has(pair);
    });
    if (!target) return;
    addMemberIfMissing(scene, {
      id: `${scheme.id}::alternate-load-path-${index + 1}`,
      role: 'brace',
      start: supportNodeId,
      end: target.id,
      source: 'scheme',
      schemeNote: 'approval required',
    });
  });
}

function addDesignOptionPreviewGeometry(scene: RenderScene, scheme: BackendDesignScheme) {
  const text = memberIntentText(scheme);
  const next: RenderScene = {
    ...scene,
    nodes: [...scene.nodes],
    members: [...scene.members],
    supports: [...scene.supports],
    loads: [...scene.loads],
    releases: [...(scene.releases ?? [])],
  };
  if (text.includes('stiffen') || text.includes('haunch')) addLocalStiffeningPreview(next, scheme);
  if (text.includes('alternate load path') || text.includes('brace') || text.includes('bracing')) addAlternateLoadPathPreview(next, scheme);
  return next;
}

export function normalizeScene(state: WorkbenchState | null): RenderScene {
  return normalizeRenderScene(state?.scene, state, { addBriefAssumptions: false });
}

export function normalizeBaseSceneWithBriefAssumptions(state: WorkbenchState | null): RenderScene {
  return normalizeRenderScene(state?.scene, state, { addBriefAssumptions: true });
}

function normalizeRenderScene(raw: WorkbenchState['scene'] | BackendDesignScheme['scene'] | undefined, state: WorkbenchState | null, options: { addBriefAssumptions?: boolean } = {}): RenderScene {
  const addAssumptions = options.addBriefAssumptions ?? false;
  const nodes = (raw?.nodes ?? []).map((n) => { const [x, y, z] = p(n); return { id: n.id, x, y, z }; }).filter((n) => Boolean(n.id));
  const members = applySingletonCoordinationFallbacks((raw?.members ?? [])
    .map((m) => ({
      id: m.id,
      role: m.role ?? 'member',
      start: start(m),
      end: end(m),
      source: 'base' as const,
      allowedSectionFamilies: m.allowedSectionFamilies ?? m.allowed_section_families ?? [],
      coordinationGroupId: m.coordinationGroupId ?? m.coordination_group_id,
      coordinationGroupLabel: m.coordinationGroupLabel ?? m.coordination_group_label,
      familyGroupLabel: m.familyGroupLabel ?? m.family_group_label,
      sectionCoordination: normalizeSectionCoordination(m),
      sizeGroupLabel: m.sizeGroupLabel ?? m.size_group_label,
      sizeCoordination: normalizeSizeCoordination(m),
      schemeNote: m.schemeNote ?? m.scheme_note,
    }))
    .filter((m) => m.start && m.end));
  if (nodes.length && members.length) {
    const scene = {
      nodes,
      members,
      supports: normalizeSupports(raw),
      loads: normalizeLoads(raw),
      releases: normalizeReleases(raw),
      unitProfile: unitProfileFrom(raw?.unitProfile ?? raw?.unit_profile ?? state?.unitProfile ?? state?.unit_profile),
    };
    return addAssumptions ? addBriefVisualAssumptions(scene, state, { supportLocations: true, loads: true }) : scene;
  }
  return fallbackScene();
}

export function deriveBaseCleanupProposal(scene: RenderScene): BaseCleanupProposal {
  const issues: BaseCleanupProposal['issues'] = [];
  for (let i = 0; i < scene.nodes.length; i += 1) {
    for (let j = i + 1; j < scene.nodes.length; j += 1) {
      const a = scene.nodes[i];
      const b = scene.nodes[j];
      const distance = Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
      if (distance > 0 && distance < 0.05) {
        issues.push({ id: `near-node-${a.id}-${b.id}`, severity: 'warning' as const, title: 'Near-duplicate nodes', summary: `${a.id} and ${b.id} are ${distance.toFixed(3)} m apart.`, proposedCorrection: 'Review whether these nodes should be merged before design-option generation.', confidence: 'medium' as const, affectedTargets: [{ kind: 'node', id: a.id }, { kind: 'node', id: b.id }] });
      }
    }
  }
  if (!issues.length) {
    issues.push({ id: 'base-cleanup-clear', severity: 'info', title: 'Base sketch looks coherent', summary: 'No near-duplicate nodes were detected in this first-pass cleanup check.', proposedCorrection: 'Use the Base Model review flow to settle roles, load assumptions, support assumptions, and qualitative constraints before generating design options.', confidence: 'high', affectedTargets: [] });
  }
  return { id: 'base-cleanup', status: 'suggested', issues };
}

function designSchemesFromState(state: WorkbenchState | null): BackendDesignScheme[] {
  const direct = state?.designSchemes;
  if (Array.isArray(direct) && direct.length) return direct;
  const snake = state?.design_schemes;
  if (Array.isArray(snake) && snake.length) return snake;
  return [];
}

function backendSchemeToEngineeringScheme(scheme: BackendDesignScheme, baseScene: RenderScene, state: WorkbenchState | null): EngineeringScheme {
  const choices = scheme.groupChoices ?? scheme.group_choices ?? [];
  const intent = scheme.intent ?? null;
  const objectiveTags = intent?.objectiveTags ?? intent?.objective_tags ?? [];
  const intentAssumptions = intent?.assumptions ?? [];
  const provenance = intent?.provenance ?? [];
  const lifecycleStatus = scheme.lifecycleStatus ?? scheme.lifecycle_status ?? intent?.lifecycleStatus ?? intent?.lifecycle_status ?? 'active';
  const supersededBy = scheme.supersededBy ?? scheme.superseded_by ?? intent?.supersededBy ?? intent?.superseded_by ?? null;
  const supersededReason = scheme.supersededReason ?? scheme.superseded_reason ?? intent?.supersededReason ?? intent?.superseded_reason ?? null;
  const revisionOf = scheme.revisionOf ?? scheme.revision_of ?? intent?.revisionOf ?? intent?.revision_of ?? null;
  const standardisationStrategy = scheme.standardisationStrategy ?? scheme.standardisation_strategy ?? intent?.standardisationStrategy ?? intent?.standardisation_strategy;
  const connectionStrategy = scheme.connectionStrategy ?? scheme.connection_strategy ?? intent?.connectionStrategy ?? intent?.connection_strategy;
  const assumptions = choices.flatMap((choice) => {
    const group = choice.coordinationGroupId ?? choice.coordination_group_id ?? 'member group';
    const families = choice.allowedSectionFamilies ?? choice.allowed_section_families ?? [];
    const notes = choice.notes ?? [];
    return [
      families.length ? `${group}: allowed section families ${families.join(', ')}.` : `${group}: section family constraints not set yet.`,
      ...notes,
    ].filter(Boolean);
  });
  const schemeScene = scheme.scene
    ? addDesignOptionPreviewGeometry(
        addBriefVisualAssumptions(normalizeRenderScene(scheme.scene, state, { addBriefAssumptions: false }), state, { supportLocations: true, loads: true }),
        scheme,
      )
    : baseScene;
  const supportStrategy = scheme.supportStrategy ?? scheme.support_strategy ?? 'Review design-option support assumptions in chat';
  return {
    id: scheme.id,
    name: scheme.label ?? scheme.id,
    status: lifecycleStatus === 'superseded' || lifecycleStatus === 'rejected' ? lifecycleStatus : 'concept',
    recommendation: lifecycleStatus === 'superseded'
      ? `Superseded by ${supersededBy ?? 'a replacement option'}.`
      : lifecycleStatus === 'rejected'
        ? 'Rejected as a comparison option.'
        : 'Review this design option as a comparison artefact.',
    summary: scheme.summary ?? 'Design option generated from the current Base Model state.',
    supersededBy,
    supersededReason,
    revisionOf,
    assumptions: [
      intent?.hypothesis ? `Hypothesis: ${intent.hypothesis}` : scheme.differentiation,
      objectiveTags.length ? `Objectives: ${objectiveTags.join(', ')}` : null,
      standardisationStrategy ? `Standardisation: ${standardisationStrategy}` : null,
      connectionStrategy ? `Connections/details: ${connectionStrategy}` : null,
      ...intentAssumptions,
      ...provenance.map((item) => `Why this is worth exploring: ${item}`),
      ...assumptions,
    ].filter(Boolean) as string[],
    operations: [],
    tradeoffs: [
      {
        label: 'Strategy',
        pros: scheme.pros ?? [],
        cons: scheme.cons ?? [],
        compromise: connectionStrategy ?? standardisationStrategy ?? scheme.strategy ?? scheme.summary ?? 'Review the option assumptions, support style, load path, and allowed section-family constraints.',
      },
    ],
    intent,
    approximateMassKg: scheme.approximateMassKg ?? scheme.approximate_mass_kg ?? null,
    analysisSummary: scheme.analysisSummary ?? scheme.analysis_summary ?? null,
    groupChoices: scheme.groupChoices ?? scheme.group_choices ?? [],
    diagnostics: scheme.diagnostics ?? [],
    scene: schemeScene,
    comparison: {
      supportStrategy,
      bracingStrategy: 'Review design-option assumptions',
      loadStrategy: 'Review design-option load assumptions in chat',
      connectionImplication: connectionStrategy ?? scheme.strategy ?? 'Review required',
      readiness: lifecycleStatus === 'superseded' ? 'Superseded' : lifecycleStatus === 'rejected' ? 'Rejected' : 'Concept option',
    },
  };
}

export function buildSchemeWorkspace(state: WorkbenchState | null): SchemeWorkspaceState {
  const baseScene = normalizeBaseSceneWithBriefAssumptions(state);
  return { baseScene, cleanup: deriveBaseCleanupProposal(baseScene), schemes: designSchemesFromState(state).map((scheme) => backendSchemeToEngineeringScheme(scheme, baseScene, state)) };
}
